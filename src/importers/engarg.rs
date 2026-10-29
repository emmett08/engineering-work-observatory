//! EngArg execution reports, completion audits and resumable checkpoints.
use super::{ImportRecord, digest, read_bounded, required_str, valid_id};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

/// An operator assigns identities to an observed execution, never infers them
/// from its output. All representations of one execution share chain/session.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EngargManifest {
    pub schema: String,
    pub provenance_chain_id: String,
    pub session_id: String,
    pub attempt_id: String,
    pub source_revision: String,
    pub artifact_kind: String,
    #[serde(default)]
    pub call_offset: u64,
    /// Explicit source-position -> logical ordinal mapping. Required for
    /// multi-call audits/checkpoints because completion order can differ from
    /// the dependency order of a final report.
    #[serde(default)]
    pub call_ordinals: Option<Vec<u64>>,
    #[serde(default)]
    pub parent_attempt_id: Option<String>,
    #[serde(default)]
    pub parent_checkpoint_sha256: Option<String>,
}

fn is_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

impl EngargManifest {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == "observatory.engarg-import/1",
            "unsupported EngArg import manifest schema"
        );
        for id in [
            &self.provenance_chain_id,
            &self.session_id,
            &self.attempt_id,
        ] {
            ensure!(
                valid_id(id),
                "manifest execution identities must be bounded ASCII identifiers"
            );
        }
        ensure!(
            self.source_revision.len() == 40
                && self.source_revision.bytes().all(|b| b.is_ascii_hexdigit()),
            "source_revision must be a full 40-character Git SHA"
        );
        ensure!(
            ["report", "audit", "checkpoint"].contains(&self.artifact_kind.as_str()),
            "artifact_kind must be report, audit, or checkpoint"
        );
        if let Some(parent) = &self.parent_attempt_id {
            ensure!(
                valid_id(parent) && parent != &self.attempt_id,
                "invalid parent_attempt_id"
            );
        }
        if let Some(parent) = &self.parent_checkpoint_sha256 {
            ensure!(is_digest(parent), "invalid parent checkpoint digest");
        }
        Ok(())
    }

    fn ordinals(&self, count: usize) -> Result<Vec<u64>> {
        let ordinals = if let Some(ordinals) = &self.call_ordinals {
            ensure!(
                ordinals.len() == count,
                "call_ordinals length must equal number of completed calls"
            );
            ordinals.clone()
        } else {
            ensure!(
                self.artifact_kind == "report" || count <= 1,
                "multi-call audits/checkpoints require call_ordinals: completion order is not necessarily report order"
            );
            (0..count)
                .map(|i| {
                    self.call_offset
                        .checked_add(i as u64)
                        .context("call ordinal overflow")
                })
                .collect::<Result<_>>()?
        };
        ensure!(
            ordinals.iter().collect::<BTreeSet<_>>().len() == ordinals.len(),
            "call_ordinals must be unique within an artifact"
        );
        Ok(ordinals)
    }
}

fn record(
    manifest: &EngargManifest,
    artifact_hash: &str,
    position: usize,
    mut data: Value,
) -> ImportRecord {
    let object = data.as_object_mut().expect("adapter data object");
    object.extend(
        json!({
            "source_type":"engarg", "source_revision":manifest.source_revision,
            "purpose":"product_execution", "provenance_chain_id":manifest.provenance_chain_id,
            "session_id":manifest.session_id, "attempt_id":manifest.attempt_id,
            "artifact_kind":manifest.artifact_kind, "artifact_sha256":artifact_hash,
            "source_position":position, "parent_attempt_id":manifest.parent_attempt_id,
            "parent_checkpoint_sha256":manifest.parent_checkpoint_sha256
        })
        .as_object()
        .unwrap()
        .clone(),
    );
    let id = digest(serde_json::to_vec(&data).expect("serializable imported JSON"));
    ImportRecord {
        id: format!("engarg:{id}"),
        kind: "source_record".into(),
        data,
    }
}

fn token(value: &Value, field: &str) -> Result<Value> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(n) if n.as_u64().is_some() => Ok(n.clone()),
        _ => bail!("{field} must be null or an unsigned integer"),
    }
}

fn classification(source: &Value) -> Result<Value> {
    let mut labels = Vec::new();
    for field in ["execution_mode", "evidence_class"] {
        match source.get(field) {
            None | Some(Value::Null) => labels.push(None),
            Some(Value::String(label)) if valid_id(label) => labels.push(Some(label.as_str())),
            _ => bail!("{field} must be a bounded execution classification label"),
        }
    }
    let explicit_synthetic = labels.iter().flatten().any(|label| {
        let parts = label
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let marker = |part: &str| {
            ["synthetic", "fixture", "demo"]
                .iter()
                .any(|marker| part.eq_ignore_ascii_case(marker))
        };
        let negated = parts.windows(2).any(|pair| {
            ["non", "not"]
                .iter()
                .any(|word| pair[0].eq_ignore_ascii_case(word))
                && marker(pair[1])
        });
        !negated && parts.iter().any(|part| marker(part))
    }) || source.get("synthetic").and_then(Value::as_bool) == Some(true);
    let synthetic = if explicit_synthetic {
        Some(true)
    } else {
        source.get("synthetic").and_then(Value::as_bool)
    };
    Ok(
        json!({"execution_mode":labels[0],"source_evidence_class":labels[1],
        "evidence_class":if explicit_synthetic {"synthetic"} else {labels[1].unwrap_or("unspecified")},
        "synthetic":synthetic}),
    )
}

fn classified(mut data: Value, classification: &Value) -> Value {
    data.as_object_mut().expect("adapter object").extend(
        classification
            .as_object()
            .expect("classification object")
            .clone(),
    );
    data
}

fn classified_records(mut records: Vec<ImportRecord>, source: &Value) -> Result<Vec<ImportRecord>> {
    let evidence_class = classification(source)?;
    for record in &mut records {
        record.data = classified(std::mem::take(&mut record.data), &evidence_class);
        record.id = format!("engarg:{}", digest(serde_json::to_vec(&record.data)?));
    }
    Ok(records)
}

fn provider_metadata(
    config: &Value,
    role: &str,
    response: &Value,
    unresolved: bool,
) -> Result<Value> {
    let selected = config
        .get("role_models")
        .and_then(|models| models.get(role))
        .or_else(|| config.get("default_model"))
        .and_then(Value::as_str);
    let assistant = config.get("assistant").filter(|v| !v.is_null());
    let kind = if let Some(assistant) = assistant {
        let name = required_str(assistant, "provider")?;
        ensure!(
            ["cursor", "augment", "command"].contains(&name),
            "unsupported assistant provider"
        );
        if let Some(hash) = assistant
            .get("configuration_sha256")
            .filter(|v| !v.is_null())
        {
            ensure!(
                hash.as_str().is_some_and(is_digest),
                "invalid assistant configuration digest"
            );
        }
        name
    } else if config.get("api").and_then(Value::as_str).is_some() {
        "http"
    } else {
        "unknown"
    };
    let label = required_str(response, "model")?;
    let configured_label =
        label.starts_with("assistant:cursor:") || label.starts_with("assistant:augment:");
    let basis = if unresolved {
        "historical_provider_unresolved"
    } else if configured_label {
        "configured_selection"
    } else if kind == "cursor" || kind == "command" {
        "adapter_reported"
    } else if kind == "http" {
        "provider_reported_or_configured_fallback"
    } else {
        "unresolved"
    };
    // Historical HTTP decoder can fall back to the requested model. The saved
    // Completion does not say whether that occurred, so do not certify it actual.
    let observed = if !unresolved && !configured_label && (kind == "cursor" || kind == "command") {
        Some(label)
    } else {
        None
    };
    Ok(json!({
        "kind":if unresolved { "unknown" } else { kind },
        "configured_model":if unresolved { None } else { selected },
        "observed_model":observed, "source_model_label":label,
        "model_identity_basis":basis,
        "configuration_sha256":if unresolved { Value::Null } else { assistant.and_then(|a| a.get("configuration_sha256")).cloned().unwrap_or(Value::Null) },
        "requested_reasoning_effort":if unresolved { Value::Null } else { config.get("requested_reasoning_effort").cloned().unwrap_or(Value::Null) },
        "returned_reasoning_effort":response.get("reasoning_effort").cloned().unwrap_or(Value::Null),
        "returned_service_tier":response.get("service_tier").cloned().unwrap_or(Value::Null)
    }))
}

struct CallLocation<'a> {
    manifest: &'a EngargManifest,
    artifact_hash: &'a str,
    schema: &'a str,
    position: usize,
    ordinal: u64,
    lineage: &'a Value,
}

fn call_record(
    location: CallLocation<'_>,
    call: &Value,
    config: &Value,
    unresolved: bool,
) -> Result<ImportRecord> {
    let CallLocation {
        manifest,
        artifact_hash,
        schema,
        position,
        ordinal,
        lineage,
    } = location;
    let role = required_str(call, "role")?;
    ensure!(valid_id(role), "invalid call role");
    let response = call
        .get("response")
        .filter(|v| v.is_object())
        .context("call lacks response object")?;
    let response_digest = required_str(call, "response_content_sha256")?;
    ensure!(
        is_digest(response_digest),
        "invalid response content digest"
    );
    let omitted = call
        .get("response_content_omitted")
        .and_then(Value::as_bool)
        .context("missing response omission flag")?;
    let content_bytes = call
        .get("response_content_bytes")
        .and_then(Value::as_u64)
        .context("invalid response byte count")?;
    if !omitted {
        let content = response
            .get("content")
            .and_then(Value::as_str)
            .context("missing completion content")?;
        ensure!(
            digest(content.as_bytes()) == response_digest,
            "completion content digest mismatch"
        );
        ensure!(
            content.len() as u64 == content_bytes,
            "completion content byte count mismatch"
        );
    }
    let elapsed = call
        .get("elapsed_ms")
        .and_then(Value::as_u64)
        .context("invalid call elapsed_ms")?;
    let canonical = digest(serde_json::to_vec(&json!([
        manifest.provenance_chain_id,
        manifest.session_id,
        ordinal
    ]))?);
    let input = token(response, "input_tokens")?;
    let output = token(response, "output_tokens")?;
    let prompt = call
        .get("prompt")
        .filter(|v| v.is_object())
        .context("call lacks prompt object")?;
    let prompt_hash = digest(serde_json::to_vec(prompt)?);
    let core_hash = digest(serde_json::to_vec(
        &json!({"role":role,"prompt_sha256":prompt_hash,"response_content_sha256":response_digest}),
    )?);
    let canonical_facts = json!({"role":role,"prompt_sha256":prompt_hash,"response_content_sha256":response_digest,
        "response_content_bytes":content_bytes,"response_content_omitted":omitted,"elapsed_ms":elapsed,
        "model":response.get("model"),"input_tokens":input,"output_tokens":output,
        "request_id":response.get("request_id").cloned().unwrap_or(Value::Null),
        "service_tier":response.get("service_tier").cloned().unwrap_or(Value::Null),
        "reasoning_effort":response.get("reasoning_effort").cloned().unwrap_or(Value::Null)});
    let data = json!({
        "record_type":"model_call", "source_schema":schema,
        "canonical_call_id":format!("engarg-call:{canonical}"), "logical_call_ordinal":ordinal,
        "call_record_sha256":digest(serde_json::to_vec(call)?),
        "core_call_sha256":core_hash,"prompt_sha256":prompt_hash,
        "canonical_fact_sha256":digest(serde_json::to_vec(&canonical_facts)?),
        "response_content_sha256":response_digest, "response_content_bytes":content_bytes,
        "response_content_omitted":omitted, "role":role, "elapsed_ms":elapsed,
        "provider":provider_metadata(config,role,response,unresolved)?,
        "usage":{"input_tokens":input,"output_tokens":output,
            "input_tokens_status":if input.is_null() {"unknown"} else {"observed"},
            "output_tokens_status":if output.is_null() {"unknown"} else {"observed"},
            "internal_inference_requests":null,"billed_cost":null},
        "lineage":lineage, "request_id_sha256":response.get("request_id").and_then(Value::as_str).map(digest),
        "source_record_sha256":digest(serde_json::to_vec(call)?)
    });
    Ok(record(manifest, artifact_hash, position, data))
}

fn recovery(root: &Value, manifest: &EngargManifest, count: usize) -> Result<(usize, Value)> {
    let Some(recovery) = root.get("authoring_recovery").filter(|v| !v.is_null()) else {
        if manifest.artifact_kind == "checkpoint"
            && let Some(parent) = root
                .pointer("/source/parent_checkpoint_sha256")
                .and_then(Value::as_str)
        {
            ensure!(is_digest(parent), "invalid checkpoint parent digest");
            let n = root
                .get("repair_allowances")
                .and_then(Value::as_array)
                .and_then(|a| a.last())
                .and_then(|a| a.get("recorded_calls"))
                .and_then(Value::as_u64)
                .context("resumed checkpoint lacks recorded_calls boundary")?;
            ensure!(
                n <= count as u64,
                "checkpoint parent prefix exceeds transcript"
            );
            if let Some(expected) = &manifest.parent_checkpoint_sha256 {
                ensure!(
                    expected == parent,
                    "manifest/checkpoint parent digest differs"
                );
            }
            return Ok((
                n as usize,
                json!({"parent_checkpoint_sha256":parent,"parent_prefix_calls":n,"linkage":"declared_source_lineage"}),
            ));
        }
        return Ok((0, Value::Null));
    };
    ensure!(
        recovery.get("schema").and_then(Value::as_str) == Some("engarg.authoring-recovery/1"),
        "unsupported authoring recovery schema"
    );
    let n = recovery
        .get("recorded_calls")
        .and_then(Value::as_u64)
        .context("missing recovery recorded_calls")?;
    let m = recovery
        .get("new_model_calls")
        .and_then(Value::as_u64)
        .context("missing recovery new_model_calls")?;
    ensure!(
        n.checked_add(m) == Some(count as u64),
        "recovery counts do not match transcript"
    );
    let order = recovery
        .get("previous_call_order")
        .and_then(Value::as_array)
        .context("missing previous_call_order")?;
    let permutation = order
        .iter()
        .map(|v| v.as_u64().context("invalid previous_call_order entry"))
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        permutation.len() == n as usize
            && permutation.iter().copied().collect::<BTreeSet<_>>() == (0..n).collect(),
        "previous_call_order is not a prefix permutation"
    );
    let parent = required_str(recovery, "parent_checkpoint_sha256")?;
    ensure!(is_digest(parent), "invalid parent checkpoint digest");
    if let Some(expected) = &manifest.parent_checkpoint_sha256 {
        ensure!(expected == parent, "manifest/report parent digest differs");
    }
    Ok((
        n as usize,
        json!({"parent_checkpoint_sha256":parent,"parent_prefix_calls":n,"new_model_calls":m,"previous_call_order":order,"linkage":"declared_source_lineage"}),
    ))
}

/// Import an EngArg report, audit JSONL, or authoring checkpoint. No commands in
/// the artifact are executed. Prompts, generated text and raw failure messages
/// stay in the private raw-artifact layer, outside transformed records.
pub fn engarg_import(path: &Path, manifest: &Path) -> Result<Vec<ImportRecord>> {
    let manifest: EngargManifest = serde_json::from_slice(&read_bounded(manifest)?)
        .context("invalid EngArg import manifest")?;
    manifest.validate()?;
    let bytes = read_bounded(path)?;
    let artifact_hash = digest(&bytes);
    let mut out = Vec::new();
    if manifest.artifact_kind == "audit" {
        let text = std::str::from_utf8(&bytes).context("audit is not UTF-8")?;
        let mut events = Vec::new();
        let mut incomplete = false;
        let lines: Vec<_> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(line) {
                Ok(value) => events.push(value),
                Err(_) if i + 1 == lines.len() && !text.ends_with('\n') => {
                    incomplete = true;
                }
                Err(_) => bail!("invalid audit JSON at line {}", i + 1),
            }
        }
        ensure!(!events.is_empty(), "audit contains no complete events");
        let count = events
            .iter()
            .filter(|v| v.get("event").and_then(Value::as_str) == Some("completion"))
            .count();
        let ordinals = manifest.ordinals(count)?;
        let mut config = Value::Null;
        let mut call_position = 0;
        let mut lineage = Value::Null;
        for (position, event) in events.iter().enumerate() {
            let name = required_str(event, "event")?;
            ensure!(
                [
                    "start",
                    "completion",
                    "failure",
                    "success",
                    "http_attempt",
                    "authoring_resume_start",
                    "resume_request",
                    "warning"
                ]
                .contains(&name),
                "unsupported EngArg audit event {name}"
            );
            if name == "start" {
                config = event.clone();
            }
            if name == "authoring_resume_start" {
                config = event
                    .get("provider_config")
                    .cloned()
                    .context("resume start lacks provider metadata")?;
                let parent = required_str(event, "parent_checkpoint_sha256")?;
                ensure!(is_digest(parent), "invalid resume parent checkpoint digest");
                if let Some(expected) = &manifest.parent_checkpoint_sha256 {
                    ensure!(expected == parent, "manifest/audit parent digest differs");
                }
                let n = event
                    .get("recorded_calls")
                    .and_then(Value::as_u64)
                    .context("resume start lacks recorded_calls")?;
                ensure!(
                    ordinals.iter().all(|i| *i >= n),
                    "resume audit ordinals overlap the parent prefix"
                );
                lineage = json!({"parent_checkpoint_sha256":parent,"parent_prefix_calls":n,"linkage":"declared_source_lineage"});
            }
            if name == "completion" {
                ensure!(
                    !config.is_null(),
                    "completion precedes audit start/provider metadata"
                );
                out.push(call_record(
                    CallLocation {
                        manifest: &manifest,
                        artifact_hash: &artifact_hash,
                        schema: "engarg.audit-jsonl/1",
                        position,
                        ordinal: ordinals[call_position],
                        lineage: &lineage,
                    },
                    event
                        .get("record")
                        .context("completion event lacks record")?,
                    &config,
                    false,
                )?);
                call_position += 1;
            } else {
                let mut data = json!({"record_type":"run_event","source_schema":"engarg.audit-jsonl/1","event":name,
                    "source_record_sha256":digest(serde_json::to_vec(event)?),"lineage":lineage,
                    "error_sha256":event.get("error").and_then(Value::as_str).map(digest),
                    "canonical_call_id":null});
                if name == "http_attempt" {
                    let source = event.get("record").context("http_attempt lacks record")?;
                    let safe = data.as_object_mut().unwrap();
                    for key in ["role", "attempt", "status", "kind", "wait_ms"] {
                        if let Some(value) = source.get(key) {
                            safe.insert(format!("http_{key}"), value.clone());
                        }
                    }
                    if let Some(failure) = source.get("response_failure") {
                        safe.insert("usage".into(),json!({"input_tokens":token(failure,"input_tokens")?,"output_tokens":token(failure,"output_tokens")?,"reasoning_tokens":token(failure,"reasoning_tokens")?,"billed_cost":null}));
                    }
                }
                out.push(record(&manifest, &artifact_hash, position, data));
            }
        }
        out.push(record(&manifest,&artifact_hash,events.len(),json!({"record_type":"artifact","source_schema":"engarg.audit-jsonl/1","canonical_call_id":null,"complete":!incomplete,"completed_logical_calls":count,"attempted_logical_calls":null})));
        let source_classification = events
            .iter()
            .find(|event| {
                matches!(
                    event.get("event").and_then(Value::as_str),
                    Some("start" | "authoring_resume_start")
                )
            })
            .unwrap_or(&Value::Null);
        return classified_records(out, source_classification);
    }
    let root: Value = serde_json::from_slice(&bytes).context("invalid EngArg JSON artifact")?;
    let schema = if manifest.artifact_kind == "checkpoint" {
        required_str(&root, "schema")?
    } else {
        required_str(&root, "schema_version")?
    };
    ensure!(
        (manifest.artifact_kind == "report"
            && ["engarg.run.v3", "engarg.demo.v2"].contains(&schema))
            || (manifest.artifact_kind == "checkpoint"
                && schema == "engarg.authoring-checkpoint/1"),
        "unsupported EngArg source schema {schema}"
    );
    if schema == "engarg.demo.v2" {
        ensure!(
            root.get("execution_mode").and_then(Value::as_str) == Some("offline_synthetic_fixture"),
            "demo schema lacks synthetic execution mode"
        );
        return classified_records(
            vec![record(
                &manifest,
                &artifact_hash,
                0,
                json!({"record_type":"artifact","source_schema":schema,"canonical_call_id":null,"synthetic":true,"completed_logical_calls":0,"usage":null}),
            )],
            &root,
        );
    }
    let calls = root
        .get("transcript")
        .and_then(Value::as_array)
        .context("artifact lacks transcript array")?;
    let ordinals = manifest.ordinals(calls.len())?;
    let (parent_count, lineage) = recovery(&root, &manifest, calls.len())?;
    if parent_count > 0 && manifest.artifact_kind == "report" {
        ensure!(
            ordinals.iter().copied().eq(0..calls.len() as u64),
            "resumed report must map its full ordered transcript from ordinal zero"
        );
    }
    for (i, call) in calls.iter().enumerate() {
        let config =
            if parent_count > 0 && i >= parent_count && root.get("authoring_recovery").is_some() {
                &root["authoring_recovery"]["provider_config"]
            } else {
                &root["provider_config"]
            };
        // A compact parent can contain several prior provider stages. Without
        // importing/validating that ancestry, provider attribution stays unknown.
        let unresolved = parent_count > 0 && ordinals[i] < parent_count as u64;
        out.push(call_record(
            CallLocation {
                manifest: &manifest,
                artifact_hash: &artifact_hash,
                schema,
                position: i,
                ordinal: ordinals[i],
                lineage: &lineage,
            },
            call,
            config,
            unresolved,
        )?);
    }
    out.push(record(
        &manifest,
        &artifact_hash,
        calls.len(),
        json!({"record_type":"artifact","source_schema":schema,"canonical_call_id":null,
        "completed_logical_calls":calls.len(),"attempted_logical_calls":null,"lineage":lineage,
        "failed":root.get("failure").is_some_and(|v|!v.is_null()),"complete":true}),
    ));
    classified_records(out, &root)
}
