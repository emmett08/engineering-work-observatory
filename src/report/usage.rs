//! Summarise declared product calls without allocating them to development effort.
use crate::snapshot::Snapshot;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn merge_known(target: &mut Value, source: &Value) {
    if target.is_null() {
        *target = source.clone();
        return;
    }
    if let (Some(left), Some(right)) = (target.as_object_mut(), source.as_object()) {
        for (key, value) in right {
            merge_known(left.entry(key.clone()).or_insert(Value::Null), value);
        }
    }
}

fn token_summary(calls: &[&Value], field: &str) -> Value {
    let values = calls
        .iter()
        .filter_map(|c| {
            c.pointer(&format!("/usage/{field}"))
                .and_then(Value::as_u64)
        })
        .collect::<Vec<_>>();
    let sum = values
        .iter()
        .try_fold(0u64, |acc, value| acc.checked_add(*value));
    json!({"known_call_count":values.len(),"unknown_call_count":calls.len()-values.len(),
        "sum_known_tokens":sum,"complete":values.len()==calls.len(),"overflow":sum.is_none()})
}

pub fn source_summary(snapshot: &Snapshot) -> Value {
    let mut kinds = BTreeMap::<String, usize>::new();
    let mut sources = BTreeMap::<String, usize>::new();
    let mut coverage = BTreeMap::<String, (chrono::DateTime<chrono::Utc>, Value)>::new();
    let mut calls = BTreeMap::<String, Value>::new();
    let mut synthetic = BTreeSet::new();
    let mut conflicts = BTreeSet::new();
    for retained in &snapshot.records {
        *kinds.entry(retained.record.kind.clone()).or_default() += 1;
        *sources.entry(retained.source.clone()).or_default() += 1;
        let data = &retained.record.data;
        if data.get("record_type").and_then(Value::as_str) == Some("collection_coverage") {
            let key = format!(
                "{}|{}|{}",
                retained.source,
                data.pointer("/coverage/source")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                data.pointer("/coverage/scope")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            );
            let mut receipt = data.clone();
            receipt["observed_at"] = json!(retained.observed_at);
            receipt["collection_source"] = json!(retained.source);
            if coverage
                .get(&key)
                .is_none_or(|(time, _)| *time < retained.observed_at)
            {
                coverage.insert(key, (retained.observed_at, receipt));
            }
        }
        if retained.record.kind != "source_record"
            || data.get("source_type").and_then(Value::as_str) != Some("engarg")
            || data.get("record_type").and_then(Value::as_str) != Some("model_call")
            || data.get("purpose").and_then(Value::as_str) != Some("product_execution")
        {
            continue;
        }
        let Some(id) = data.get("canonical_call_id").and_then(Value::as_str) else {
            continue;
        };
        if data.get("synthetic").and_then(Value::as_bool) == Some(true) {
            synthetic.insert(id.to_owned());
        }
        if let Some(prior) = calls.get_mut(id) {
            for field in [
                "/core_call_sha256",
                "/usage/input_tokens",
                "/usage/output_tokens",
                "/provider/observed_model",
            ] {
                if let (Some(a), Some(b)) = (prior.pointer(field), data.pointer(field))
                    && !a.is_null()
                    && !b.is_null()
                    && a != b
                {
                    conflicts.insert(id.to_owned());
                }
            }
            merge_known(prior, data);
        } else {
            calls.insert(id.to_owned(), data.clone());
        }
    }
    let product = calls
        .iter()
        .filter(|(id, _)| !synthetic.contains(*id))
        .map(|(_, call)| call)
        .collect::<Vec<_>>();
    let mut providers = BTreeMap::<String, usize>::new();
    for call in &product {
        *providers
            .entry(
                call.pointer("/provider/kind")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned(),
            )
            .or_default() += 1;
    }
    let coverage = coverage
        .into_values()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    json!({"record_counts":kinds,"source_counts":sources,"coverage":coverage,
        "logical_product_calls_observed":product.len(),"synthetic_logical_calls":synthetic.len(),
        "product_calls_by_provider":providers,"input_tokens":token_summary(&product,"input_tokens"),"output_tokens":token_summary(&product,"output_tokens"),
        "conflicting_call_ids":conflicts,"invoiced_cost":null,
        "note":"Product calls are not development effort. Known token totals are partial when any usage is unknown. Unmarked artifacts have unspecified evidence class. Native failures may have no completion record; logical calls do not count native internal inference requests. Explicitly synthetic calls are excluded from product usage totals."})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::Evidence,
        store::{RetainedRecord, Store},
    };
    #[test]
    fn summaries_deduplicate_enrich_and_keep_synthetic_separate() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let now = chrono::Utc::now();
        let mut snapshot = Snapshot::freeze(&store, now).unwrap();
        for (id, canonical, tokens, is_synthetic) in [
            ("a", "call1", Value::Null, false),
            ("b", "call1", json!(10), false),
            ("c", "call2", json!(30), true),
        ] {
            snapshot.records.push(RetainedRecord{record:Evidence{id:id.into(),kind:"source_record".into(),data:json!({"source_type":"engarg","record_type":"model_call","purpose":"product_execution","canonical_call_id":canonical,"core_call_sha256":"same","usage":{"input_tokens":tokens,"output_tokens":null},"synthetic":is_synthetic})},observed_at:now,source:"fixture".into(),raw_sha256:"test".into()});
        }
        let result = source_summary(&snapshot);
        assert_eq!(result["logical_product_calls_observed"], 1);
        assert_eq!(result["synthetic_logical_calls"], 1);
        assert_eq!(result["input_tokens"]["sum_known_tokens"], 10);
        assert_eq!(result["output_tokens"]["unknown_call_count"], 1);
        assert!(
            result["conflicting_call_ids"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}
