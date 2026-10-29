//! Render escaped offline task tables and labelled SVG charts.
use crate::snapshot::Snapshot;
use serde_json::Value;

pub(super) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn cell(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "unknown".into(),
        Some(Value::String(s)) => escape(s),
        Some(value) => escape(&value.to_string()),
    }
}

fn bounds_cell(value: Option<&Value>) -> String {
    let Some(value) = value.filter(|value| value.is_object()) else {
        return "unavailable".into();
    };
    format!(
        "[{}, {}]",
        cell(value.get("lower_person_hours")),
        cell(value.get("upper_person_hours"))
    )
}

fn table(report: &Value) -> String {
    let mut html = String::from(
        "<table><caption>Task outcomes; bounds approximate deterministic conservative enclosures for complete direct frames. Floating-point endpoints are not outward rounded; these are not confidence intervals.</caption><thead><tr><th>Task</th><th>Status</th><th>Person-hours point result</th><th>Person-hours bounds</th><th>Retained units</th><th>Issues</th></tr></thead><tbody>",
    );
    for task in report["tasks"].as_array().into_iter().flatten() {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            cell(task.get("task_id")),
            cell(task.get("lifecycle_status")),
            cell(task.get("person_hours")),
            bounds_cell(task.get("person_hours_bounds")),
            cell(task.get("retained_output")),
            cell(task.get("issues"))
        ));
    }
    html.push_str("</tbody></table>");
    html
}

fn chart(report: &Value, field: &str, label: &str) -> String {
    let tasks = report["tasks"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let shown = tasks.len().min(100);
    let height = shown * 32 + 36;
    let max = tasks
        .iter()
        .take(shown)
        .filter_map(|t| t[field].as_f64())
        .fold(0.0, f64::max)
        .max(1.0);
    let mut svg = format!(
        "<h3>{}</h3><svg role=\"img\" aria-label=\"{} by task\" viewBox=\"0 0 900 {}\"><title>{} by task</title>",
        escape(label),
        escape(label),
        height,
        escape(label)
    );
    for (i, task) in tasks.iter().take(shown).enumerate() {
        let y = i * 32 + 24;
        let id = cell(task.get("task_id"));
        svg.push_str(&format!(
            "<text x=\"0\" y=\"{y}\" font-size=\"13\">{id}</text>"
        ));
        if let Some(value) = task[field].as_f64() {
            let width = 500.0 * (value / max);
            svg.push_str(&format!("<rect x=\"280\" y=\"{}\" width=\"{width:.2}\" height=\"18\" fill=\"#28758c\"/><text x=\"{}\" y=\"{y}\" font-size=\"13\">{value:.3}</text>",y-15,790));
        } else {
            svg.push_str(&format!(
                "<text x=\"280\" y=\"{y}\" font-size=\"13\">unknown</text>"
            ));
        }
    }
    svg.push_str("</svg>");
    if tasks.len() > shown {
        svg.push_str(
            "<p>Chart limited to the first 100 tasks; the table and JSON retain all tasks.</p>",
        );
    }
    svg
}

pub(super) fn render(snapshot: &Snapshot, report: &Value, sources: &Value) -> String {
    format!(
        r#"<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Engineering Work Observatory</title><style>body{{font:16px/1.55 system-ui,sans-serif;max-width:1100px;margin:3rem auto;padding:0 1rem;color:#172b3a;background:#f7fafc}}h1{{font-size:2rem}}pre{{padding:1.2rem;overflow:auto;background:white;border:1px solid #c8d5df;border-radius:8px}}code{{overflow-wrap:anywhere}}p{{max-width:80ch}}table{{border-collapse:collapse;width:100%;background:white}}th,td{{border:1px solid #ccd7dd;text-align:left;padding:.5rem;vertical-align:top;overflow-wrap:anywhere}}svg{{width:100%;background:white}}.stamp{{color:#415869}}</style><h1>Engineering Work Observatory</h1><p class="stamp">Knowledge cut: {}<br>Snapshot: <code>{}</code></p><p>This private, descriptive evidence report separates human effort, elapsed acceptance time and retained output. Missing observations remain unknown. No causal AI effect is estimated.</p><h2>Task accounting</h2>{}{}{}<details><summary>Full calculations and qualifications</summary><pre>{}</pre></details><h2>Source coverage and product usage</h2><pre>{}</pre><p>Product model calls, commits and passing CI do not establish human effort or retained correctness. report.json, records.csv and snapshot.json may contain private records and are not anonymised publications. Original evidence remains in the private state directory; back it up with the snapshots.</p></html>"#,
        escape(&snapshot.as_of.to_rfc3339()),
        escape(&snapshot.sha256),
        table(report),
        chart(report, "person_hours", "Human effort (person-hours)"),
        chart(
            report,
            "retained_output",
            "Retained requirement output (reference units)"
        ),
        escape(&serde_json::to_string_pretty(report).expect("JSON serializes")),
        escape(&serde_json::to_string_pretty(sources).expect("JSON serializes"))
    )
}
