//! Retain source responses and ingest a collection under its explicit completeness policy.
use crate::{
    collectors::{Collection, Coverage},
    importers::validate_execution_records,
    model::{Evidence, validate_records},
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

fn execution_interpretation(source: Option<&str>) -> bool {
    matches!(
        source,
        Some(
            "observatory.tool_receipt"
                | "observatory.execution_bundle"
                | "github.execution_reconciliation"
                | "github.execution_job_reconciliation"
        )
    )
}

/// Preserve independent GitHub observations when an execution delivery conflicts
/// with a previously retained receipt. Quarantine every execution interpretation
/// in that collection rather than choosing one conflicting representation.
/// Raw archives, inventory, runs and jobs remain retained and verifiable; the
/// incomplete coverage makes the command fail after those observations persist.
pub fn persist_execution_collection(
    store: &mut Store,
    mut collection: Collection,
    source: &str,
) -> Result<CollectionReceipt> {
    let mut combined = store.records(None)?;
    combined.extend(
        collection
            .records
            .iter()
            .filter(|record| execution_interpretation(record.data["source"].as_str()))
            .map(|record| Evidence {
                id: record.id.clone(),
                kind: record.kind.clone(),
                data: record.data.clone(),
            }),
    );
    if let Err(error) =
        validate_records(&combined).and_then(|()| validate_execution_records(&combined))
    {
        let before = collection.records.len();
        collection
            .records
            .retain(|record| !execution_interpretation(record.data["source"].as_str()));
        let quarantined = before - collection.records.len();
        for coverage in &mut collection.coverage {
            if coverage.source == "github.execution_artifact_bundle" && coverage.state == "complete"
            {
                coverage.state = "incomplete".into();
                coverage.records = 0;
                coverage.detail = Some(
                    "Raw artifact retained; execution interpretations quarantined because this collection conflicts with retained evidence".into(),
                );
            }
        }
        collection.coverage.push(Coverage {
            source: "observatory.execution_ingestion".into(),
            scope: source.into(),
            state: "incomplete".into(),
            pages: 0,
            records: 0,
            detail: Some(format!(
                "Quarantined {quarantined} execution interpretations; independent GitHub metadata and raw archives retained: {error}"
            )),
        });
    }
    persist_collection(store, collection, source, CollectionRequirements::Full)
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
