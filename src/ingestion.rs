//! Retain source responses and ingest a collection under its explicit completeness policy.
use crate::{
    collectors::Collection,
    model::Evidence,
    store::{Store, digest},
};
use anyhow::{Result, bail};
use chrono::Utc;
use serde_json::json;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionRequirements {
    Full,
    Throughput,
}

pub struct CollectionReceipt {
    pub new_records: usize,
    pub manifest_sha256: String,
}

/// Retain partial evidence before reporting failure; never promote omitted required sources.
pub fn persist_collection(
    store: &mut Store,
    collection: Collection,
    source: &str,
    requirements: CollectionRequirements,
) -> Result<CollectionReceipt> {
    let mut raw_objects = Vec::new();
    let mut response_metadata = Vec::new();
    for observation in &collection.observations {
        let hash = store.retain_bytes(&observation.body)?;
        raw_objects.push(hash.clone());
        let mut value = serde_json::to_value(observation)?;
        if let Some(map) = value.as_object_mut() {
            map.remove("body");
            map.insert("body_sha256".into(), json!(hash));
        }
        response_metadata.push(value);
    }
    let manifest = json!({"schema":"observatory.collection/1","source":source,"raw_objects":raw_objects,"responses":response_metadata,"coverage":collection.coverage});
    let hash = store.retain_bytes(&serde_json::to_vec(&manifest)?)?;
    let mut records: Vec<Evidence> = collection
        .records
        .into_iter()
        .map(|r| Evidence {
            id: r.id,
            kind: r.kind,
            data: r.data,
        })
        .collect();
    for coverage in &collection.coverage {
        let data = json!({"record_type":"collection_coverage","coverage":coverage});
        records.push(Evidence {
            id: format!("coverage:{}", digest(&serde_json::to_vec(&data)?)),
            kind: "source_record".into(),
            data,
        });
    }
    let count = store.insert_batch(&records, source, Utc::now(), &hash)?;
    let incomplete: Vec<_> = collection
        .coverage
        .iter()
        .filter(|c| {
            c.state != "complete"
                && c.state != "not_applicable"
                && !(c.state == "not_collected"
                    && (requirements == CollectionRequirements::Throughput
                        || ["github.workflow_logs", "github.history"].contains(&c.source.as_str())))
        })
        .collect();
    let missing_required: Vec<_> = if requirements == CollectionRequirements::Throughput {
        ["github.repository", "github.pull_request"]
            .into_iter()
            .filter(|required| {
                let scopes: Vec<_> = collection
                    .coverage
                    .iter()
                    .filter(|c| c.source == *required)
                    .collect();
                scopes.len() != 1 || scopes[0].state != "complete"
            })
            .collect()
    } else {
        Vec::new()
    };
    if !incomplete.is_empty() || !missing_required.is_empty() {
        bail!(
            "collection retained with incomplete coverage: {}; missing required sources: {}",
            serde_json::to_string(&incomplete)?,
            serde_json::to_string(&missing_required)?
        );
    }
    Ok(CollectionReceipt {
        new_records: count,
        manifest_sha256: hash,
    })
}
