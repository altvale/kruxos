//! Unit tests for the pure applier. Every assertion is on an OBSERVABLE: the
//! returned `new_buffer` bytes and the outcome JSON shape. Each test fails
//! without the corresponding behaviour (no tautologies).

use super::*;
use serde_json::json;

fn single(old: &str, new: &str) -> Value {
    json!([{ "old_string": old, "new_string": new }])
}

fn single_ra(old: &str, new: &str, replace_all: bool) -> Value {
    json!([{ "old_string": old, "new_string": new, "replace_all": replace_all }])
}

fn opts_single() -> ApplyOpts {
    ApplyOpts { multi: false, path: "/w/f.rs".to_string(), expected_sha: None }
}

fn opts_multi() -> ApplyOpts {
    ApplyOpts { multi: true, path: "/w/f.rs".to_string(), expected_sha: None }
}

fn outcome(v: &Value) -> &str {
    v.get("outcome").and_then(Value::as_str).unwrap_or("<none>")
}

// ----------------------------- exact tier --------------------------------

#[test]
fn exact_unique_applies() {
    let buf = "fn a() {}\nfn b() {}\n";
    let (nb, out) = apply(buf, &single("fn b() {}", "fn c() {}"), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["match_tier"], "exact");
    assert_eq!(out["replacements"], 1);
    assert_eq!(out["path"], "/w/f.rs");
    let nb = nb.expect("applied yields a new buffer");
    assert_eq!(nb, "fn a() {}\nfn c() {}\n");
    // checksum is over the FINAL bytes and matches the shared hasher.
    assert_eq!(out["checksum"], sha256_hex(nb.as_bytes()));
    assert_eq!(out["bytes_before"], buf.len());
    assert_eq!(out["bytes_after"], nb.len());
}

#[test]
fn empty_new_string_is_a_delete_and_survives() {
    // An empty new_string is a meaningful DELETE — must apply, not be coerced away.
    let buf = "keep\nDROP ME\nkeep2\n";
    let (nb, out) = apply(buf, &single("DROP ME\n", ""), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(nb.unwrap(), "keep\nkeep2\n");
}

#[test]
fn no_match_returns_recovery_no_write() {
    let buf = "alpha\nbeta\n";
    let (nb, out) = apply(buf, &single("gamma", "delta"), &opts_single());
    assert_eq!(outcome(&out), "no_match");
    assert!(nb.is_none());
    assert!(out["recovery"].as_str().unwrap().contains("filesystem.read"));
    // No path key on a non-applied outcome.
    assert!(out.get("path").is_none());
}

#[test]
fn ambiguous_match_no_write_line_numbers_only() {
    let buf = "x = 1\ny = 2\nx = 1\n";
    let (nb, out) = apply(buf, &single("x = 1", "x = 9"), &opts_single());
    assert_eq!(outcome(&out), "ambiguous_match");
    assert!(nb.is_none());
    assert_eq!(out["match_count"], 2);
    let lines = out["match_lines"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], 1);
    assert_eq!(lines[1], 3);
    assert_eq!(out["truncated"], false);
    // No laundering: the file content "x = 1" appears in old_string but the
    // recovery must NOT echo any buffer line content beyond what the model sent.
    let rec = out["recovery"].as_str().unwrap();
    assert!(rec.contains("matched 2 sites"));
    assert!(rec.contains("lines 1, 3"));
}

#[test]
fn ambiguous_match_lines_are_capped() {
    let mut buf = String::new();
    for _ in 0..30 {
        buf.push_str("dup\n");
    }
    let (_, out) = apply(&buf, &single("dup", "x"), &opts_single());
    assert_eq!(outcome(&out), "ambiguous_match");
    assert_eq!(out["match_count"], 30);
    assert_eq!(out["match_lines"].as_array().unwrap().len(), MATCH_LINES_CAP);
    assert_eq!(out["truncated"], true);
}

// --------------------------- replace_all ---------------------------------

#[test]
fn replace_all_changes_every_occurrence() {
    let buf = "foo foo foo";
    let (nb, out) = apply(buf, &single_ra("foo", "bar", true), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["replacements"], 3);
    assert_eq!(nb.unwrap(), "bar bar bar");
}

#[test]
fn without_replace_all_multiple_is_ambiguous() {
    let buf = "foo foo";
    let (nb, out) = apply(buf, &single("foo", "bar"), &opts_single());
    assert_eq!(outcome(&out), "ambiguous_match");
    assert!(nb.is_none());
}

#[test]
fn replace_all_too_generic_is_parse_error() {
    // old_string shorter than the min replace_all length.
    let buf = "a a a a";
    let (nb, out) = apply(buf, &single_ra("a", "b", true), &opts_single());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
    assert!(out["recovery"].as_str().unwrap().contains("narrow"));
}

#[test]
fn replace_all_over_ceiling_is_parse_error() {
    let buf = "abcd".repeat(MAX_REPLACEMENTS + 5);
    let (nb, out) = apply(&buf, &single_ra("abcd", "wxyz", true), &opts_single());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
    assert!(out["recovery"].as_str().unwrap().contains("ceiling"));
}

// ---------------------------- parse_error --------------------------------

#[test]
fn empty_old_string_is_parse_error() {
    let buf = "anything";
    let (nb, out) = apply(buf, &single("", "x"), &opts_single());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

#[test]
fn old_equals_new_is_parse_error() {
    let buf = "anything here";
    let (nb, out) = apply(buf, &single("here", "here"), &opts_single());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

#[test]
fn non_object_edits_element_is_parse_error_not_panic() {
    let buf = "data";
    let edits = json!(["not-an-object"]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

#[test]
fn edits_not_array_is_parse_error() {
    let buf = "data";
    let edits = json!({ "old_string": "d", "new_string": "x" });
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

#[test]
fn over_edit_limit_is_parse_error() {
    let buf = "x";
    let mut arr = Vec::new();
    for i in 0..(MAX_EDITS_PER_MULTI_EDIT + 1) {
        arr.push(json!({ "old_string": format!("o{i}"), "new_string": format!("n{i}") }));
    }
    let (nb, out) = apply(buf, &Value::Array(arr), &opts_multi());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

#[test]
fn match_string_over_ceiling_is_parse_error() {
    let buf = "x";
    let big = "z".repeat(MAX_MATCH_LEN + 1);
    let (nb, out) = apply(buf, &single(&big, "y"), &opts_single());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
}

// --------------------------- whitespace tier -----------------------------

#[test]
fn tier2_whitespace_normalized_applies() {
    // Buffer is indented 8 spaces; the model's old_string uses 4. Tier-1 fails,
    // Tier-2 normalizes leading/trailing whitespace and matches uniquely.
    let buf = "fn main() {\n        let x = 1;\n        let y = 2;\n}\n";
    let old = "    let x = 1;\n    let y = 2;";
    let new = "    let x = 10;\n    let y = 20;";
    let (nb, out) = apply(buf, &single(old, new), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["match_tier"], "whitespace");
    let nb = nb.unwrap();
    // The matched block was replaced with new (model indentation); surrounding
    // bytes preserved.
    assert!(nb.starts_with("fn main() {\n"));
    assert!(nb.ends_with("}\n"));
    assert!(nb.contains("let x = 10;"));
}

#[test]
fn tier2_does_not_collapse_internal_whitespace_or_blank_lines() {
    // Internal double-space and a blank line carry meaning; Tier-2 must NOT
    // collapse them. old_string with collapsed internal spacing must NOT match.
    let buf = "a  b\n\nc\n";
    // old collapses the internal double space -> must NOT match (no_match).
    let (nb, out) = apply(buf, &single("a b", "Z"), &opts_single());
    assert_eq!(outcome(&out), "no_match");
    assert!(nb.is_none());
}

#[test]
fn tier2_ambiguity_is_not_silently_resolved() {
    // Two regions identical after leading/trailing-ws normalization -> ambiguous,
    // never a silent first-match apply.
    let buf = "    foo\n        foo\nbar\n";
    let (nb, out) = apply(buf, &single("foo", "FOO"), &opts_single());
    // Tier-1 exact: "foo" occurs twice exactly -> already ambiguous at Tier-1.
    assert_eq!(outcome(&out), "ambiguous_match");
    assert!(nb.is_none());
}

#[test]
fn tier2_ambiguous_via_normalization() {
    // Tier-1 finds zero (different indent on each), Tier-2 normalizes to two
    // matches -> ambiguous, not applied.
    let buf = "  alpha(1)\n    alpha(1)\n";
    let old = "alpha(1)"; // no exact match? "alpha(1)" IS an exact substring twice.
    // Make Tier-1 miss by giving old leading ws that doesn't match either line.
    let _ = old;
    let old2 = "\talpha(1)"; // tab-indented; no exact match in buffer (spaces).
    let (nb, out) = apply(buf, &single(old2, "X"), &opts_single());
    assert_eq!(outcome(&out), "ambiguous_match");
    assert_eq!(out["match_count"], 2);
    assert!(nb.is_none());
}

// ------------------------------- EOL -------------------------------------

#[test]
fn crlf_file_model_sends_lf_applies_and_keeps_crlf() {
    let buf = "line1\r\nline2\r\nline3\r\n";
    // Model emits old_string/new_string with \n.
    let (nb, out) = apply(buf, &single("line2", "LINE2"), &opts_single());
    assert_eq!(outcome(&out), "applied");
    let nb = nb.unwrap();
    assert_eq!(nb, "line1\r\nLINE2\r\nline3\r\n");
    // No lone LF was introduced: every '\n' is preceded by '\r'.
    let bytes = nb.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            assert!(i > 0 && bytes[i - 1] == b'\r', "lone LF introduced at byte {i}");
        }
    }
}

#[test]
fn crlf_multiline_old_string_with_lf_translates() {
    let buf = "a\r\nb\r\nc\r\n";
    let old = "a\nb"; // model uses LF
    let new = "A\nB";
    let (nb, out) = apply(buf, &single(old, new), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(nb.unwrap(), "A\r\nB\r\nc\r\n");
}

#[test]
fn mixed_eol_falls_through_to_raw_and_names_crlf_on_miss() {
    // One CRLF line, one LF line -> mixed. A raw exact match still works.
    let buf = "a\r\nb\nc\n";
    let (nb, out) = apply(buf, &single("b\nc", "B\nC"), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(nb.unwrap(), "a\r\nB\nC\n");

    // A miss on a mixed file names CRLF in the recovery.
    let (_, miss) = apply(buf, &single("zzz", "qqq"), &opts_single());
    assert_eq!(outcome(&miss), "no_match");
    assert!(miss["recovery"].as_str().unwrap().contains("MIXED"));
}

// ----------------------------- expected_sha ------------------------------

#[test]
fn expected_sha_match_applies() {
    let buf = "hello world\n";
    let sha = sha256_hex(buf.as_bytes());
    let opts = ApplyOpts {
        multi: false,
        path: "/w/f".to_string(),
        expected_sha: Some(sha),
    };
    let (nb, out) = apply(buf, &single("world", "kruxos"), &opts);
    assert_eq!(outcome(&out), "applied");
    assert_eq!(nb.unwrap(), "hello kruxos\n");
}

#[test]
fn expected_sha_mismatch_is_stale_file() {
    let buf = "hello world\n";
    let opts = ApplyOpts {
        multi: false,
        path: "/w/f".to_string(),
        expected_sha: Some("deadbeef".to_string()),
    };
    let (nb, out) = apply(buf, &single("world", "kruxos"), &opts);
    assert_eq!(outcome(&out), "stale_file");
    assert!(nb.is_none());
}

#[test]
fn sha256_hex_is_lowercase_hex_over_raw_bytes() {
    // Hash parity primitive: known vector for the empty input.
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

// ------------------------------ multi_edit -------------------------------

#[test]
fn multi_edit_sequential_over_evolving_buffer() {
    let buf = "one\ntwo\nthree\n";
    let edits = json!([
        { "old_string": "one", "new_string": "1" },
        { "old_string": "two", "new_string": "2" },
        { "old_string": "three", "new_string": "3" },
    ]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["edits_applied"], 3);
    assert_eq!(out["edits_total"], 3);
    assert_eq!(out["per_edit"].as_array().unwrap().len(), 3);
    assert_eq!(nb.unwrap(), "1\n2\n3\n");
}

#[test]
fn multi_edit_earlier_edit_makes_later_anchor_unique() {
    // The ALLOWED sequential pattern: edit A rewrites one of two ambiguous sites
    // (via unique surrounding context) so edit B's bare anchor becomes unique
    // against the EVOLVING buffer. Both apply; edit B does NOT match inside the
    // bytes edit A wrote (so the overlap guard is not triggered).
    let buf = "target\nNOISE\ntarget\n";
    let edits = json!([
        { "old_string": "NOISE\ntarget", "new_string": "NOISE\nDONE" },
        { "old_string": "target", "new_string": "FIRST" },
    ]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["edits_applied"], 2);
    assert_eq!(nb.unwrap(), "FIRST\nNOISE\nDONE\n");
}

#[test]
fn multi_edit_all_or_nothing_on_failure() {
    let buf = "one\ntwo\nthree\n";
    let edits = json!([
        { "old_string": "one", "new_string": "1" },
        { "old_string": "MISSING", "new_string": "x" }, // fails
        { "old_string": "three", "new_string": "3" },
    ]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "no_match");
    assert!(nb.is_none(), "ZERO edits applied — buffer unchanged");
    assert_eq!(out["failed_edit_index"], 1);
    let rec = out["recovery"].as_str().unwrap();
    assert!(rec.contains("Batch rejected at edit index 1"));
    assert!(rec.contains("resubmit the COMPLETE edits[] array"));
    assert_eq!(out["failed_edit"]["outcome"], "no_match");
}

#[test]
fn multi_edit_overlap_guard_rejects() {
    // edit 1 writes "XXXX" in place of "one"; edit 2's match falls inside that
    // written range -> parse_error (region overlap).
    let buf = "one\n";
    let edits = json!([
        { "old_string": "one", "new_string": "XYZ" },
        { "old_string": "XYZ", "new_string": "Q" }, // matches inside edit-1's output
    ]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "parse_error");
    assert!(nb.is_none());
    assert_eq!(out["failed_edit_index"], 1);
    assert!(out["failed_edit"]["recovery"]
        .as_str()
        .unwrap()
        .contains("written by an earlier edit"));
}

#[test]
fn multi_edit_one_commit_byte_exact() {
    // Two edits at different sites -> single coherent final buffer.
    let buf = "head\nmid\ntail\n";
    let edits = json!([
        { "old_string": "head", "new_string": "HEAD" },
        { "old_string": "tail", "new_string": "TAIL" },
    ]);
    let (nb, out) = apply(buf, &edits, &opts_multi());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(nb.unwrap(), "HEAD\nmid\nTAIL\n");
    assert_eq!(out["path"], "/w/f.rs");
}

// ------------------------- byte-preservation -----------------------------

#[test]
fn untouched_bytes_far_from_match_are_preserved() {
    // A large buffer with a unique match near the end; assert the prefix is intact.
    let prefix = "PREFIX_LINE\n".repeat(5000);
    let buf = format!("{prefix}UNIQUE_TARGET\n");
    let (nb, out) = apply(&buf, &single("UNIQUE_TARGET", "DONE"), &opts_single());
    assert_eq!(outcome(&out), "applied");
    let nb = nb.unwrap();
    assert!(nb.starts_with(&prefix));
    assert_eq!(nb, format!("{prefix}DONE\n"));
}

#[test]
fn lines_added_removed_reflect_change_region() {
    let buf = "a\nb\nc\n";
    // remove a 1-line region (with its newline span via the matched bytes),
    // insert a 2-line region.
    let (_, out) = apply(buf, &single("b\n", "B1\nB2\n"), &opts_single());
    assert_eq!(outcome(&out), "applied");
    assert_eq!(out["lines_removed"], 1);
    assert_eq!(out["lines_added"], 2);
}
