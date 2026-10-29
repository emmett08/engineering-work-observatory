//! Export a frozen calculation and source coverage without sending private traces elsewhere.
use crate::{analysis::analyse, snapshot::Snapshot};
use anyhow::{Result, ensure};
use serde_json::json;
use std::{fs, path::Path};
mod html;
mod usage;
pub use usage::source_summary;

fn csv_cell(value: &str) -> String {
    let mut safe = value.to_owned();
    if safe.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        safe.insert(0, '\'');
    }
    format!("\"{}\"", safe.replace('"', "\"\""))
}

pub fn write_report(snapshot: &Snapshot, output: &Path) -> Result<()> {
    snapshot.verify()?;
    ensure!(
        !output.exists() || output.read_dir()?.next().is_none(),
        "report directory must be new or empty to preserve prior results"
    );
    let report = serde_json::to_value(analyse(&snapshot.evidence(), snapshot.as_of)?)?;
    let sources = source_summary(snapshot);
    let document = json!({"schema":"observatory.report/1","snapshot_sha256":snapshot.sha256,
        "as_of":snapshot.as_of,"snapshot_producer_revision":snapshot.implementation_revision,
        "snapshot_producer_source_sha256":snapshot.implementation_source_sha256,"analyser_source_sha256":env!("OBSERVATORY_SOURCE_SHA256"),
        "analyser_revision":env!("OBSERVATORY_BUILD_REVISION"),"analyser_package_version":env!("CARGO_PKG_VERSION"),
        "calculation_version":snapshot.calculation_version,"analysis":report,"sources":sources});
    fs::create_dir_all(output)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(output, fs::Permissions::from_mode(0o700))?;
    }
    fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&document)?,
    )?;
    fs::write(
        output.join("snapshot.json"),
        serde_json::to_vec_pretty(snapshot)?,
    )?;
    let mut csv = String::from("id,kind,observed_at,source,data_json\n");
    for r in &snapshot.records {
        csv.push_str(
            &[
                csv_cell(&r.record.id),
                csv_cell(&r.record.kind),
                csv_cell(&r.observed_at.to_rfc3339()),
                csv_cell(&r.source),
                csv_cell(&serde_json::to_string(&r.record.data)?),
            ]
            .join(","),
        );
        csv.push('\n');
    }
    fs::write(output.join("records.csv"), csv)?;
    let html = html::render(snapshot, &report, &sources);
    fs::write(output.join("index.html"), html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn user_text_cannot_become_html_or_formula() {
        assert_eq!(html::escape("<script>&\""), "&lt;script&gt;&amp;&quot;");
        assert_eq!(csv_cell("=SUM(A1)"), "\"'=SUM(A1)\"");
    }
}
