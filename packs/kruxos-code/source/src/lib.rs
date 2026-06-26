//! `kruxos-code` WASM pack guest — the `code.edit` / `code.multi_edit`
//! capabilities. Compiled against the frozen `pack` world (vendored at
//! `wit/kruxos-pack.wit`) and dispatched by the host runtime.
//!
//! The guest is deliberately thin: it does I/O only through the capability-gated
//! `host-fs` imports (so scope confinement, the secrets denylist, write-protect,
//! and the atomic temp+rename happen host-side, with zero pack code) and
//! delegates ALL edit logic — matching, EOL translation, the outcome taxonomy —
//! to the pure `kruxos-code-applier` crate.
//!
//! Outcome encoding: the four data outcomes {applied, ambiguous_match, no_match,
//! parse_error} (plus stale_file when an `expected_sha` is supplied) ride inside
//! the `Ok(json_string)` under a top-level `outcome` discriminator, so their
//! recovery survives the host's tool-result projection and reaches the model.
//! Only GENUINE host errors (PathOutOfScope / PathDenied / PathWriteProtected /
//! FileNotFound / ResourceExhausted), plus the guest-synthesized EncodingError,
//! return `Err(cap-error)` and propagate with `?`.

wit_bindgen::generate!({
    world: "pack",
    path: "wit/kruxos-pack.wit",
});

use exports::kruxos::pack::capabilities::Guest as Capabilities;
use kruxos::pack::host_fs;
use kruxos::pack::types::{CapError, RecoveryAction};
use kruxos_code_applier::{apply, ApplyOpts, MAX_FILE_BYTES};
use serde_json::{json, Value};

struct Pack;

fn parse_inputs(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}

fn cap_error(
    error_type: &str,
    description: &str,
    agent: &str,
    recovery: Vec<RecoveryAction>,
) -> CapError {
    CapError {
        error_type: error_type.to_string(),
        description: description.to_string(),
        agent_description: agent.to_string(),
        recovery_actions: recovery,
        retryable: false,
        retry_after_secs: None,
        context_json: None,
    }
}

/// Binary / non-UTF-8 target — the host returns raw bytes with no validation, so
/// the GUEST detects this and routes to `filesystem.write` (code.edit has no
/// encoding argument and cannot operate on non-text).
fn encoding_error() -> CapError {
    cap_error(
        "EncodingError",
        "Target file is not valid UTF-8 text",
        "This target is binary or non-UTF-8 text; code.edit operates on UTF-8 text only. Use filesystem.write to replace a binary file wholesale.",
        vec![RecoveryAction {
            action: "use_filesystem_write".to_string(),
            description: "Write the binary/non-text file wholesale instead of editing.".to_string(),
            capability: Some("filesystem.write".to_string()),
            inputs_json: None,
        }],
    )
}

/// File above the 16 MiB edit ceiling — synthesized from `host-fs.stat` so a huge
/// file is never read into the guest's bounded memory. The host's `read-file`
/// emits the same `ResourceExhausted` error_type if this guard is bypassed.
fn too_large_error() -> CapError {
    cap_error(
        "ResourceExhausted",
        "File exceeds the 16 MiB code.edit ceiling",
        "This file is larger than 16 MiB; code.edit cannot load it. Use filesystem.write for very large files.",
        vec![RecoveryAction {
            action: "use_filesystem_write".to_string(),
            description: "Use filesystem.write for files above the edit ceiling.".to_string(),
            capability: Some("filesystem.write".to_string()),
            inputs_json: None,
        }],
    )
}

/// Defensive guard for structurally-impossible input (the registry validates
/// required inputs before the guest is reached, so this is a belt-and-suspenders
/// net, not part of the normal contract).
fn invalid_input(msg: &str) -> CapError {
    cap_error("InvalidInput", msg, msg, vec![])
}

impl Capabilities for Pack {
    fn invoke(capability: String, inputs_json: String) -> Result<String, CapError> {
        // Dispatch on the suffix after the last '.', so both a bare op name and
        // the dotted registry name ("code.edit") resolve.
        let op = capability.rsplit('.').next().unwrap_or(capability.as_str());
        let v = parse_inputs(&inputs_json);

        let path = match str_field(&v, "path") {
            Some(p) if !p.is_empty() => p,
            _ => return Err(invalid_input("missing or empty 'path'")),
        };

        // Normalize the request into the applier's (edits-array, multi) shape.
        let (edits, multi) = match op {
            "edit" => {
                let edit = json!({
                    "old_string": v.get("old_string").cloned().unwrap_or(Value::Null),
                    "new_string": v.get("new_string").cloned().unwrap_or(Value::Null),
                    "replace_all": v.get("replace_all").cloned().unwrap_or(Value::Bool(false)),
                });
                (Value::Array(vec![edit]), false)
            }
            "multi_edit" => (v.get("edits").cloned().unwrap_or(Value::Null), true),
            other => return Err(invalid_input(&format!("unknown capability '{other}'"))),
        };

        let opts = ApplyOpts {
            multi,
            path: path.clone(),
            expected_sha: str_field(&v, "expected_sha"),
        };

        // host-fs.stat — size pre-check so a huge file is never read into wasm
        // memory (propagates FileNotFound / PathOutOfScope / PathDenied with `?`).
        let st = host_fs::stat(&path)?;
        if st.size as usize > MAX_FILE_BYTES {
            return Err(too_large_error());
        }

        // host-fs.read-file — the whole file (the host also caps at max_fs_bytes
        // and emits ResourceExhausted if this guard were bypassed).
        let bytes = host_fs::read_file(&path)?;
        let buffer = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return Err(encoding_error()),
        };

        // Pure, deterministic apply — identical native and in-wasm.
        let (new_buffer, outcome) = apply(&buffer, &edits, &opts);

        // Write ONCE, ONLY on the `applied` outcome (the sole `Some` result).
        // host-fs.write-file re-enters scope + write-protect + secrets denial and
        // commits atomically; a denial propagates with `?`.
        if let Some(nb) = new_buffer {
            host_fs::write_file(&path, nb.as_bytes())?;
        }

        Ok(outcome.to_string())
    }
}

export!(Pack);
