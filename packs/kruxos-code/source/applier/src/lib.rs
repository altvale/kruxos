//! `kruxos-code-applier` — the pure, deterministic edit applier behind the
//! `code.edit` / `code.multi_edit` capabilities of the kruxos-code pack.
//!
//! This crate is intentionally PURE: it touches no filesystem, no scope/policy
//! engine, no network, no TLS/openssl. It takes an already-read file buffer (a
//! `&str`) plus the model's edit request (as `serde_json::Value`) and returns
//! `(Option<new_buffer>, outcome_json)`. The same crate is compiled into the
//! WASM pack guest AND linked natively into the off-box evaluation harness, so
//! the edit OUTCOME is byte-identical on both — that equivalence is the whole
//! point of factoring it out.
//!
//! ## Outcome taxonomy (the `outcome` discriminator, snake_case)
//! Every result carries a stable top-level `outcome` string:
//!
//! - `applied` — exactly the intended change was made.
//! - `ambiguous_match` — `old_string` matched more than one site (and
//!   `replace_all` was not set). Nothing was written.
//! - `no_match` — `old_string` matched zero sites. Nothing was written.
//! - `parse_error` — the request itself was malformed (empty old_string,
//!   old == new, a non-object edits[] element, over the edit ceiling, or a
//!   too-generic replace_all). Nothing written.
//! - `stale_file` — an `expected_sha` was supplied and did NOT match the
//!   file's current hash. Nothing written.
//!
//! `applied` is the ONLY outcome that produces a `Some(new_buffer)` to write,
//! and the ONLY outcome that carries a `path` key. Non-applied outcomes omit
//! every path-shaped key so a caller never emits a phantom file-change event.
//!
//! Recovery strings carry NO file content — only counts and 1-based line
//! numbers — so a near-miss can never launder file bytes into a model-visible
//! result.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Ceilings (the handler clamps; the registry never does).
// ---------------------------------------------------------------------------

/// Maximum editable file size — aligned to the host's `max_fs_bytes` so a file
/// the host would refuse to read/write is also refused here. Exposed so the
/// guest can stat-gate before reading a file into wasm memory.
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

/// Maximum length of a single `old_string` / `new_string` (bytes).
pub const MAX_MATCH_LEN: usize = 1024 * 1024;

/// Maximum number of edits in one `code.multi_edit` batch.
pub const MAX_EDITS_PER_MULTI_EDIT: usize = 50;

/// Maximum number of sites a single `replace_all` edit may rewrite.
pub const MAX_REPLACEMENTS: usize = 1000;

/// Minimum `old_string` length (in chars) required when `replace_all` is set —
/// stops a one-character `old_string` from silently rewriting a whole file.
pub const MIN_REPLACE_ALL_OLD_STRING_LEN: usize = 3;

/// How many match line numbers an `ambiguous_match` recovery may list before it
/// is truncated (a `truncated` flag is then set). Keeps a pathological
/// ambiguous match well under any tool-result byte budget.
pub const MATCH_LINES_CAP: usize = 20;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Options threaded into [`apply`].
#[derive(Debug, Clone, Default)]
pub struct ApplyOpts {
    /// `true` for `code.multi_edit` (batch, all-or-nothing, per-edit aggregates);
    /// `false` for `code.edit` (a single edit).
    pub multi: bool,
    /// The (already scope-validated) path, echoed into the `applied` outcome.
    /// Purely data — the applier never touches the filesystem.
    pub path: String,
    /// Optional unchanged-since-read pin. When `Some`, the raw buffer is hashed
    /// and compared; a mismatch yields a `stale_file` outcome and zero writes.
    pub expected_sha: Option<String>,
}

/// Lowercase-hex SHA-256 over the EXACT raw bytes. This is the ONE shared
/// hasher: the same function computes the `applied` checksum, the
/// `expected_sha` comparison, and (in the host) the `filesystem.read` checksum,
/// so all three agree byte-for-byte.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Apply `edits` to `buffer`, returning `(maybe_new_buffer, outcome_json)`.
///
/// `edits` MUST be a JSON array of edit objects, each
/// `{ "old_string": String, "new_string": String, "replace_all"?: bool }`.
/// For `code.edit` the caller wraps the single edit in a one-element array and
/// sets `opts.multi = false`; for `code.multi_edit` it passes the `edits` array
/// and sets `opts.multi = true`.
///
/// On `applied` the returned `Option` is `Some(new_buffer)` (the post-edit,
/// post-EOL-translation bytes to write); for every other outcome it is `None`
/// and nothing should be written.
pub fn apply(buffer: &str, edits: &Value, opts: &ApplyOpts) -> (Option<String>, Value) {
    // 1) Parse + structurally validate the edit request (the model's malformed
    //    input class -> parse_error, zero work).
    let specs = match parse_edits(edits, opts.multi) {
        Ok(s) => s,
        Err(pf) => return (None, build_parse_error(opts.multi, pf.index, &pf.reason)),
    };

    // 2) Optional unchanged-since-read pin, over the RAW buffer bytes
    //    (pre-EOL-translate, pre-decode is moot — `buffer.as_bytes()` IS the
    //    raw file bytes the guest read).
    if let Some(expected) = &opts.expected_sha {
        let actual = sha256_hex(buffer.as_bytes());
        if &actual != expected {
            return (None, stale_file_outcome());
        }
    }

    // 3) Detect the file's dominant line ending once.
    let eol_mode = detect_eol(buffer);

    // 4) Apply edits SEQUENTIALLY over one evolving buffer; any non-applied
    //    edit rejects the whole request (single edit: itself; multi: the batch).
    let mut cur = buffer.to_string();
    let mut written: Vec<(usize, usize)> = Vec::new();
    let mut per_edit: Vec<Value> = Vec::new();
    let mut total_lines_added = 0usize;
    let mut total_lines_removed = 0usize;

    for (i, spec) in specs.iter().enumerate() {
        let old_t = translate_eol(&spec.old_string, eol_mode);
        let new_t = translate_eol(&spec.new_string, eol_mode);

        match match_one(&cur, &old_t, &new_t, spec.replace_all, eol_mode, &written) {
            EditStep::Applied {
                replacements,
                tier,
                repls,
                lines_added,
                lines_removed,
            } => {
                let (next, next_written) = apply_replacements(&cur, &written, &repls);
                cur = next;
                written = next_written;
                total_lines_added += lines_added;
                total_lines_removed += lines_removed;
                per_edit.push(json!({
                    "index": i,
                    "outcome": "applied",
                    "replacements": replacements,
                    "match_tier": tier,
                }));
            }
            EditStep::NoMatch => {
                return (
                    None,
                    build_no_match(opts.multi, i, eol_mode == EolMode::Mixed),
                );
            }
            EditStep::Ambiguous { count, lines, truncated } => {
                return (
                    None,
                    build_ambiguous(opts.multi, i, count, &lines, truncated),
                );
            }
            EditStep::ParseErr { reason } => {
                return (None, build_parse_error(opts.multi, Some(i), &reason));
            }
        }
    }

    // 5) Everything applied -> assemble the Applied outcome.
    let bytes_before = buffer.len();
    let bytes_after = cur.len();
    let checksum = sha256_hex(cur.as_bytes());

    let outcome = if opts.multi {
        let total = per_edit.len();
        let replacements_total: u64 = per_edit
            .iter()
            .map(|e| e.get("replacements").and_then(Value::as_u64).unwrap_or(0))
            .sum();
        json!({
            "outcome": "applied",
            "edits_applied": total,
            "edits_total": total,
            "replacements": replacements_total,
            "per_edit": per_edit,
            "bytes_before": bytes_before,
            "bytes_after": bytes_after,
            "lines_added": total_lines_added,
            "lines_removed": total_lines_removed,
            "checksum": checksum,
            "path": opts.path,
        })
    } else {
        // Single edit: exactly one per_edit entry.
        let (replacements, tier) = per_edit
            .first()
            .map(|e| {
                (
                    e.get("replacements").and_then(Value::as_u64).unwrap_or(1),
                    e.get("match_tier")
                        .and_then(Value::as_str)
                        .unwrap_or("exact")
                        .to_string(),
                )
            })
            .unwrap_or((1, "exact".to_string()));
        json!({
            "outcome": "applied",
            "replacements": replacements,
            "match_tier": tier,
            "bytes_before": bytes_before,
            "bytes_after": bytes_after,
            "lines_added": total_lines_added,
            "lines_removed": total_lines_removed,
            "checksum": checksum,
            "path": opts.path,
        })
    };

    (Some(cur), outcome)
}

// ---------------------------------------------------------------------------
// Edit parsing + structural validation
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct EditSpec {
    old_string: String,
    new_string: String,
    replace_all: bool,
}

struct ParseFail {
    index: Option<usize>,
    reason: String,
}

fn parse_edits(edits: &Value, multi: bool) -> Result<Vec<EditSpec>, ParseFail> {
    let arr = edits.as_array().ok_or_else(|| ParseFail {
        index: None,
        reason: "edits must be a JSON array".to_string(),
    })?;

    if multi {
        if arr.is_empty() {
            return Err(ParseFail {
                index: None,
                reason: "edits[] is empty — provide at least one edit".to_string(),
            });
        }
        if arr.len() > MAX_EDITS_PER_MULTI_EDIT {
            return Err(ParseFail {
                index: None,
                reason: format!(
                    "too many edits ({}) — the ceiling is {MAX_EDITS_PER_MULTI_EDIT} per batch; split the work",
                    arr.len()
                ),
            });
        }
    }

    let mut specs = Vec::with_capacity(arr.len());
    for (i, el) in arr.iter().enumerate() {
        let obj = el.as_object().ok_or_else(|| ParseFail {
            index: Some(i),
            reason: "malformed edits[] element (not a JSON object)".to_string(),
        })?;
        let old_string = obj
            .get("old_string")
            .and_then(Value::as_str)
            .ok_or_else(|| ParseFail {
                index: Some(i),
                reason: "edit is missing a string old_string".to_string(),
            })?;
        let new_string = obj
            .get("new_string")
            .and_then(Value::as_str)
            .ok_or_else(|| ParseFail {
                index: Some(i),
                reason: "edit is missing a string new_string".to_string(),
            })?;
        let replace_all = obj
            .get("replace_all")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if old_string.is_empty() {
            return Err(ParseFail {
                index: Some(i),
                reason: "old_string is empty".to_string(),
            });
        }
        if old_string == new_string {
            return Err(ParseFail {
                index: Some(i),
                reason: "old_string == new_string (no-op edit)".to_string(),
            });
        }
        if old_string.len() > MAX_MATCH_LEN || new_string.len() > MAX_MATCH_LEN {
            return Err(ParseFail {
                index: Some(i),
                reason: format!("a match string exceeds the {MAX_MATCH_LEN}-byte ceiling"),
            });
        }
        if replace_all && old_string.chars().count() < MIN_REPLACE_ALL_OLD_STRING_LEN {
            return Err(ParseFail {
                index: Some(i),
                reason: format!(
                    "replace_all old_string is too generic (< {MIN_REPLACE_ALL_OLD_STRING_LEN} chars) — narrow it"
                ),
            });
        }

        specs.push(EditSpec {
            old_string: old_string.to_string(),
            new_string: new_string.to_string(),
            replace_all,
        });
    }
    Ok(specs)
}

// ---------------------------------------------------------------------------
// EOL detection + translation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EolMode {
    Lf,
    Crlf,
    Mixed,
}

fn detect_eol(buffer: &str) -> EolMode {
    let crlf = buffer.matches("\r\n").count();
    let total_lf = buffer.matches('\n').count();
    let lone_lf = total_lf.saturating_sub(crlf);
    if crlf > 0 && lone_lf > 0 {
        EolMode::Mixed
    } else if crlf > 0 {
        EolMode::Crlf
    } else {
        EolMode::Lf
    }
}

/// Translate a model-supplied string to the file's dominant EOL. For a
/// uniformly-CRLF file the model's `\n` becomes `\r\n` (idempotent: any `\r\n`
/// the model already sent is first collapsed so we never produce `\r\r\n`).
/// LF and mixed-EOL files are left untouched (mixed falls through to raw
/// exact-match).
fn translate_eol(s: &str, mode: EolMode) -> String {
    match mode {
        EolMode::Crlf => s.replace("\r\n", "\n").replace('\n', "\r\n"),
        EolMode::Lf | EolMode::Mixed => s.to_string(),
    }
}

fn eol_str(mode: EolMode) -> &'static str {
    match mode {
        EolMode::Crlf => "\r\n",
        // Mixed never reaches the line-window tier; default to "\n".
        EolMode::Lf | EolMode::Mixed => "\n",
    }
}

// ---------------------------------------------------------------------------
// Matching — Tier-1 exact, then Tier-2 per-line leading/trailing-ws normalized
// ---------------------------------------------------------------------------

enum EditStep {
    Applied {
        replacements: usize,
        tier: &'static str,
        /// (start, end, replacement) spans in the CURRENT buffer, sorted ascending,
        /// non-overlapping.
        repls: Vec<(usize, usize, String)>,
        lines_added: usize,
        lines_removed: usize,
    },
    NoMatch,
    Ambiguous {
        count: usize,
        lines: Vec<usize>,
        truncated: bool,
    },
    ParseErr {
        reason: String,
    },
}

fn match_one(
    cur: &str,
    old_t: &str,
    new_t: &str,
    replace_all: bool,
    eol_mode: EolMode,
    prior_written: &[(usize, usize)],
) -> EditStep {
    // ---- TIER 1: exact byte match, uniqueness-enforced ----
    let exact = find_all(cur, old_t);
    if !exact.is_empty() {
        return resolve_matches(
            cur,
            new_t,
            replace_all,
            prior_written,
            exact.iter().map(|&off| (off, off + old_t.len())).collect(),
            "exact",
        );
    }

    // ---- TIER 2: per-line leading/trailing-whitespace-normalized ----
    // Skipped for mixed-EOL files (raw exact-match only — CRLF is named in the
    // no_match recovery there).
    if eol_mode != EolMode::Mixed {
        if let Some(spans) = tier2_candidates(cur, old_t, new_t, eol_mode) {
            if !spans.is_empty() {
                return resolve_matches(cur, new_t, replace_all, prior_written, spans, "whitespace");
            }
        }
    }

    EditStep::NoMatch
}

/// Common resolution of a set of candidate (start, end) match spans into an
/// outcome, honoring `replace_all`, the overlap guard, and the replacement
/// ceiling. For Tier-2, candidate spans carry a pre-computed replacement; for
/// Tier-1 the replacement is always `new_t`.
fn resolve_matches(
    cur: &str,
    new_t: &str,
    replace_all: bool,
    prior_written: &[(usize, usize)],
    candidates: Vec<(usize, usize)>,
    tier: &'static str,
) -> EditStep {
    // De-overlap candidates greedily (only relevant for replace_all over
    // periodic patterns); preserves ascending order.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for (s, e) in candidates {
        if spans.last().map(|&(_, pe)| s >= pe).unwrap_or(true) {
            spans.push((s, e));
        }
    }
    let count = spans.len();

    if count > 1 && !replace_all {
        let mut lines: Vec<usize> = spans.iter().map(|&(s, _)| line_of(cur, s)).collect();
        let truncated = lines.len() > MATCH_LINES_CAP;
        lines.truncate(MATCH_LINES_CAP);
        return EditStep::Ambiguous { count, lines, truncated };
    }

    if replace_all && count > MAX_REPLACEMENTS {
        return EditStep::ParseErr {
            reason: format!(
                "replace_all matched {count} sites, over the {MAX_REPLACEMENTS} ceiling — narrow old_string"
            ),
        };
    }

    // Overlap guard: a match falling inside a byte range an earlier edit wrote.
    for &(s, e) in &spans {
        for &(ws, we) in prior_written {
            if s < we && ws < e {
                return EditStep::ParseErr {
                    reason: "matched inside text written by an earlier edit in this batch — reorder or add surrounding context".to_string(),
                };
            }
        }
    }

    let repls: Vec<(usize, usize, String)> =
        spans.iter().map(|&(s, e)| (s, e, new_t.to_string())).collect();
    let lines_removed: usize = repls
        .iter()
        .map(|(s, e, _)| cur[*s..*e].matches('\n').count())
        .sum();
    let lines_added: usize = repls.iter().map(|(_, _, r)| r.matches('\n').count()).sum();

    EditStep::Applied {
        replacements: count,
        tier,
        repls,
        lines_added,
        lines_removed,
    }
}

/// All non-overlapping byte offsets at which `needle` occurs in `hay`.
/// `needle` is guaranteed non-empty by upstream validation.
fn find_all(hay: &str, needle: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = hay.get(start..).and_then(|h| h.find(needle)) {
        let abs = start + pos;
        out.push(abs);
        start = abs + needle.len();
    }
    out
}

/// 1-based line number of the given byte offset.
fn line_of(buffer: &str, off: usize) -> usize {
    buffer.get(..off).map(|s| s.matches('\n').count()).unwrap_or(0) + 1
}

// --- Tier-2 line-window matching ---

struct Seg<'a> {
    content: &'a str,
    start: usize,
}

/// Split `s` into segments on `eol`, tracking each segment's start byte offset.
/// Mirrors `str::split` semantics: a trailing `eol` yields a final empty
/// segment ("a\nb\n" -> ["a","b",""]).
fn split_segments<'a>(s: &'a str, eol: &str) -> Vec<Seg<'a>> {
    let mut segs = Vec::new();
    let mut start = 0;
    let mut idx = 0;
    loop {
        match s.get(idx..).and_then(|rest| rest.find(eol)) {
            Some(pos) => {
                let abs = idx + pos;
                segs.push(Seg { content: &s[start..abs], start });
                idx = abs + eol.len();
                start = idx;
            }
            None => {
                segs.push(Seg { content: &s[start..], start });
                break;
            }
        }
    }
    segs
}

/// Find every buffer region whose per-line leading/trailing-whitespace-trimmed
/// content equals the trimmed `old_t` content. Returns the (start, end) byte
/// spans to replace. Returns `None` only when the structure is unusable.
fn tier2_candidates(
    cur: &str,
    old_t: &str,
    _new_t: &str,
    eol_mode: EolMode,
) -> Option<Vec<(usize, usize)>> {
    let eol = eol_str(eol_mode);
    let old_segs = split_segments(old_t, eol);
    let old_has_trailing = old_t.ends_with(eol);
    let w = if old_has_trailing {
        old_segs.len().saturating_sub(1)
    } else {
        old_segs.len()
    };
    if w == 0 {
        return None;
    }
    let buf_segs = split_segments(cur, eol);
    if w > buf_segs.len() {
        return Some(Vec::new());
    }
    // If trimming old_t to whitespace-only-equal would match the raw bytes
    // exactly, Tier-1 already handled it; Tier-2 is reached only when Tier-1
    // found zero, so we proceed.
    let mut spans = Vec::new();
    let last_buf_idx = buf_segs.len() - 1;
    for j in 0..=(buf_segs.len() - w) {
        let all_match = (0..w).all(|k| buf_segs[j + k].content.trim() == old_segs[k].content.trim());
        if !all_match {
            continue;
        }
        let content_start = buf_segs[j].start;
        let last = &buf_segs[j + w - 1];
        let content_end = last.start + last.content.len();
        let span_end = if old_has_trailing && (j + w - 1) < last_buf_idx {
            // The matched block was followed by an EOL in the buffer; consume it
            // too so the replacement supplants whole lines.
            content_end + eol.len()
        } else {
            content_end
        };
        spans.push((content_start, span_end));
    }
    Some(spans)
}

// ---------------------------------------------------------------------------
// Replacement application + written-range bookkeeping
// ---------------------------------------------------------------------------

/// Apply `repls` (sorted ascending, non-overlapping) to `buffer`, returning the
/// new buffer and the updated set of written byte ranges (in NEW-buffer
/// coordinates): the prior ranges shifted by the cumulative length delta, plus
/// the newly-written ranges.
fn apply_replacements(
    buffer: &str,
    prior_written: &[(usize, usize)],
    repls: &[(usize, usize, String)],
) -> (String, Vec<(usize, usize)>) {
    let mut out = String::with_capacity(buffer.len());
    let mut new_written = Vec::new();
    let mut cursor = 0usize;
    for (s, e, r) in repls {
        if let Some(gap) = buffer.get(cursor..*s) {
            out.push_str(gap);
        }
        let new_start = out.len();
        out.push_str(r);
        new_written.push((new_start, out.len()));
        cursor = *e;
    }
    if let Some(tail) = buffer.get(cursor..) {
        out.push_str(tail);
    }

    // Shift prior ranges into NEW coordinates. Prior ranges never overlap the
    // repls (the overlap guard rejected such edits), so each prior range shifts
    // uniformly by the delta of all repls ending at or before its start.
    let mut shifted = Vec::with_capacity(prior_written.len() + new_written.len());
    for &(a, b) in prior_written {
        let mut delta: isize = 0;
        for (s, e, r) in repls {
            if *e <= a {
                delta += r.len() as isize - (*e as isize - *s as isize);
            }
        }
        let na = (a as isize + delta).max(0) as usize;
        let nb = (b as isize + delta).max(0) as usize;
        shifted.push((na, nb));
    }
    shifted.extend(new_written);
    shifted.sort_unstable();
    (out, shifted)
}

// ---------------------------------------------------------------------------
// Outcome builders (recovery strings carry NO file content)
// ---------------------------------------------------------------------------

const BATCH_ROLLBACK_HINT: &str = "all-or-nothing — ZERO edits applied, file unchanged";

fn build_no_match(multi: bool, index: usize, mixed_eol: bool) -> Value {
    let mut recovery = String::from(
        "old_string not found (0 occurrences). Whitespace/indentation/line-endings may differ, \
         the file may have changed since you read it, or you already applied this edit \
         (old_string is now gone). Call filesystem.read, copy old_string verbatim, then retry.",
    );
    if mixed_eol {
        recovery.push_str(
            " This file has MIXED CRLF/LF line endings, so it is matched byte-for-byte — \
             include the exact \\r\\n bytes where present.",
        );
    }
    if multi {
        batch_wrap(index, "no_match", json!({ "recovery": recovery }))
    } else {
        json!({ "outcome": "no_match", "recovery": recovery })
    }
}

fn build_ambiguous(
    multi: bool,
    index: usize,
    count: usize,
    lines: &[usize],
    truncated: bool,
) -> Value {
    let line_list = lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let recovery = format!(
        "old_string matched {count} sites (lines {line_list}{}). Add unique surrounding context \
         to match exactly one, OR set replace_all=true to change every occurrence.",
        if truncated { ", …" } else { "" }
    );
    let detail = json!({
        "recovery": recovery,
        "match_count": count,
        "match_lines": lines,
        "truncated": truncated,
    });
    if multi {
        batch_wrap(index, "ambiguous_match", detail)
    } else {
        let mut v = detail;
        if let Some(obj) = v.as_object_mut() {
            obj.insert("outcome".to_string(), json!("ambiguous_match"));
        }
        v
    }
}

fn build_parse_error(multi: bool, index: Option<usize>, reason: &str) -> Value {
    let recovery = format!("Malformed edit: {reason}. Fix the arguments and retry — do not retry unchanged.");
    match (multi, index) {
        (true, Some(i)) => batch_wrap(i, "parse_error", json!({ "recovery": recovery })),
        // A batch-level parse error with no specific index (e.g. edits[] not an
        // array, empty, or over the count ceiling).
        (true, None) => json!({
            "outcome": "parse_error",
            "recovery": format!("{recovery} ({BATCH_ROLLBACK_HINT})"),
        }),
        (false, _) => json!({ "outcome": "parse_error", "recovery": recovery }),
    }
}

fn stale_file_outcome() -> Value {
    json!({
        "outcome": "stale_file",
        "recovery": "The file changed since you read it (expected_sha mismatch). \
                     Call filesystem.read to get the current contents, then retry the edit.",
    })
}

/// Wrap a single failing edit's `detail` (which already carries its specific
/// `recovery` + any metadata) into a batch-reject outcome: the top-level
/// `outcome` is the failure class (so the metric buckets it correctly), the
/// top-level `recovery` is the all-or-nothing rollback instruction naming the
/// index, and `failed_edit` carries the specific per-edit detail.
fn batch_wrap(index: usize, class: &str, detail: Value) -> Value {
    let mut failed = detail;
    if let Some(obj) = failed.as_object_mut() {
        obj.insert("index".to_string(), json!(index));
        obj.insert("outcome".to_string(), json!(class));
    }
    json!({
        "outcome": class,
        "failed_edit_index": index,
        "recovery": format!(
            "Batch rejected at edit index {index} ({BATCH_ROLLBACK_HINT}). Fix edit {index}, \
             then resubmit the COMPLETE edits[] array — do not drop the edits that matched."
        ),
        "failed_edit": failed,
    })
}

#[cfg(test)]
mod tests;
