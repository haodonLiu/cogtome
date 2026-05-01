//! Trace Dashboard - HTTP endpoint for COGTOME execution trace visualization
//!
//! Reads trace JSONL files from COGTOME_TRACE_DIR and serves formatted execution traces
//! via an HTML dashboard or REST API.
//!
//! Trace file format (from engine/mod.rs emit_trace):
//!   { "trace_id": "...", "skill": "...", "date": "YYYY-MM-DD", "started_at": "...",
//!     "completed_at": "...", "duration_ms": N, "status": "success|error", "nodes": [...] }

use anyhow::Result;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Trace directory resolved from env or default ~/.cogtome/traces
fn resolve_trace_dir() -> PathBuf {
    std::env::var("COGTOME_TRACE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".cogtome/traces"))
                .unwrap_or_else(|| PathBuf::from(".traces"))
        })
}

/// Query parameters for trace listing
#[derive(Debug, Deserialize)]
pub struct TraceQuery {
    pub skill: Option<String>,
    pub date: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// Single trace record (parsed from JSONL)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraceRecord {
    #[serde(rename = "trace_id")]
    pub trace_id: Option<String>,
    pub skill: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "started_at")]
    pub started_at: Option<String>,
    #[serde(rename = "completed_at")]
    pub completed_at: Option<String>,
    #[serde(rename = "duration_ms")]
    pub duration_ms: Option<u64>,
    pub status: String,
    pub nodes: Vec<NodeTrace>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeTrace {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub ok: bool,
    pub exit_code: Option<i32>,
    pub ms: Option<u64>,
    pub error: Option<String>,
}

/// Trace summary stats
#[derive(Debug, Serialize)]
pub struct TraceStats {
    pub total: usize,
    pub successes: usize,
    pub failures: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub skills: HashMap<String, usize>,
}

/// App state for trace dashboard
#[derive(Clone)]
struct TraceDashboardState {
    trace_dir: PathBuf,
}

impl TraceDashboardState {
    fn new() -> Self {
        Self {
            trace_dir: resolve_trace_dir(),
        }
    }
}

/// List all trace records (REST API)
async fn list_traces(
    axum::extract::State(state): axum::extract::State<TraceDashboardState>,
    Query(query): Query<TraceQuery>,
) -> Json<Vec<TraceRecord>> {
    let records = read_traces_from_dir(&state.trace_dir, &query);
    Json(records)
}

/// Get trace statistics
async fn trace_stats(
    axum::extract::State(state): axum::extract::State<TraceDashboardState>,
    Query(_query): Query<TraceQuery>,
) -> Json<TraceStats> {
    let records = read_traces_from_dir(&state.trace_dir, &default_trace_query());
    compute_stats(&records)
}

/// Get a single trace by ID
async fn get_trace(
    axum::extract::State(state): axum::extract::State<TraceDashboardState>,
    axum::extract::Path(trace_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let records = read_all_traces(&state.trace_dir);
    if let Some(record) = records.into_iter().find(|r| r.trace_id.as_ref() == Some(&trace_id)) {
        Json(record).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Trace not found" }))).into_response()
    }
}

/// Serve the HTML trace dashboard
async fn trace_dashboard() -> Html<String> {
    Html(build_dashboard_html())
}

fn default_trace_query() -> TraceQuery {
    TraceQuery {
        skill: None,
        date: None,
        limit: 50,
    }
}

fn read_traces_from_dir(dir: &PathBuf, query: &TraceQuery) -> Vec<TraceRecord> {
    let mut records = Vec::new();
    if !dir.exists() {
        return records;
    }

    let entries: Vec<_> = fs::read_dir(dir).into_iter().flatten().flatten().collect();

    for entry in entries {
        let path = entry.path();
        let skill_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let skill_filter = query.skill.as_deref();

        // Skip non-matching skills
        if let Some(filter) = skill_filter {
            if skill_name != filter {
                continue;
            }
        }

        if path.is_dir() {
            // Skill subdirectory: <skill_name>/<date>.jsonl
            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if sub_path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        let sub_date = sub_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if let Some(ref date_filter) = query.date {
                            if sub_date != date_filter {
                                continue;
                            }
                        }
                        let parsed = parse_jsonl_file(&sub_path);
                        records.extend(parsed);
                    }
                }
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // Top-level trace file
            let parsed = parse_jsonl_file(&path);
            records.extend(parsed);
        }
    }

    // Sort by started_at descending
    records.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
    });

    records.truncate(query.limit);
    records
}

fn read_all_traces(dir: &PathBuf) -> Vec<TraceRecord> {
    let query = TraceQuery {
        skill: None,
        date: None,
        limit: usize::MAX,
    };
    let mut records = read_traces_from_dir(dir, &query);
    records.truncate(10000);
    records
}

fn parse_jsonl_file(path: &PathBuf) -> Vec<TraceRecord> {
    let mut records = Vec::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Ok(record) = serde_json::from_str::<TraceRecord>(&line) {
                records.push(record);
            }
        }
    }
    records
}

fn compute_stats(records: &[TraceRecord]) -> Json<TraceStats> {
    let total = records.len();
    let successes = records.iter().filter(|r| r.status == "success").count();
    let failures = total - successes;
    let success_rate = if total > 0 {
        (successes as f64 / total as f64) * 1000.0
    } else {
        0.0
    };

    let avg_duration_ms = if total > 0 {
        records
            .iter()
            .filter_map(|r| r.duration_ms)
            .sum::<u64>() as f64
            / total as f64
    } else {
        0.0
    };

    let mut skills: HashMap<String, usize> = HashMap::new();
    for r in records {
        if let Some(ref skill) = r.skill {
            *skills.entry(skill.clone()).or_insert(0) += 1;
        }
    }

    Json(TraceStats {
        total,
        successes,
        failures,
        success_rate: (success_rate.round() / 1000.0 * 100.0).round() / 100.0,
        avg_duration_ms: (avg_duration_ms * 10.0).round() / 10.0,
        skills,
    })
}

fn build_dashboard_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>COGTOME Trace Dashboard</title>
<style>
  :root {{
    --bg: #0f1117;
    --surface: #1a1d27;
    --border: #2a2d3a;
    --text: #e4e4e7;
    --muted: #71717a;
    --accent: #6366f1;
    --success: #22c55e;
    --error: #ef4444;
    --warn: #f59e0b;
  }}
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: 'Segoe UI', system-ui, sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; }}
  header {{ background: var(--surface); border-bottom: 1px solid var(--border); padding: 1rem 2rem; display: flex; align-items: center; gap: 1rem; }}
  header h1 {{ font-size: 1.25rem; color: var(--text); }}
  header .badge {{ background: var(--accent); color: white; font-size: 0.7rem; padding: 0.2rem 0.6rem; border-radius: 999px; font-weight: 600; }}
  .container {{ max-width: 1200px; margin: 0 auto; padding: 2rem; }}
  .filters {{ display: flex; gap: 1rem; margin-bottom: 1.5rem; flex-wrap: wrap; align-items: center; }}
  .filters input, .filters select, .filters button {{
    background: var(--surface); border: 1px solid var(--border); color: var(--text);
    padding: 0.5rem 1rem; border-radius: 6px; font-size: 0.875rem;
  }}
  .filters button {{ background: var(--accent); border: none; cursor: pointer; }}
  .filters button:hover {{ opacity: 0.85; }}
  .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; margin-bottom: 2rem; }}
  .stat-card {{ background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 1.25rem; }}
  .stat-card .label {{ font-size: 0.75rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.5rem; }}
  .stat-card .value {{ font-size: 2rem; font-weight: 700; }}
  .stat-card .value.success {{ color: var(--success); }}
  .stat-card .value.error {{ color: var(--error); }}
  table {{ width: 100%; border-collapse: collapse; background: var(--surface); border-radius: 8px; overflow: hidden; }}
  th {{ text-align: left; padding: 0.75rem 1rem; font-size: 0.75rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; border-bottom: 1px solid var(--border); }}
  td {{ padding: 0.75rem 1rem; font-size: 0.875rem; border-bottom: 1px solid var(--border); }}
  tr:last-child td {{ border-bottom: none; }}
  tr:hover td {{ background: rgba(255,255,255,0.02); }}
  .status-badge {{ display: inline-block; padding: 0.15rem 0.5rem; border-radius: 4px; font-size: 0.75rem; font-weight: 600; }}
  .status-badge.success {{ background: rgba(34,197,94,0.15); color: var(--success); }}
  .status-badge.error {{ background: rgba(239,68,68,0.15); color: var(--error); }}
  .node-list {{ font-size: 0.75rem; color: var(--muted); }}
  .node-ok {{ color: var(--success); }} .node-fail {{ color: var(--error); }}
  .empty {{ text-align: center; padding: 4rem; color: var(--muted); }}
  .loading {{ text-align: center; padding: 2rem; color: var(--muted); }}
</style>
</head>
<body>
<header>
  <h1>⚡ COGTOME Trace Dashboard</h1>
  <span class="badge">v0.2.0</span>
</header>
<div class="container">
  <div class="stats-grid" id="stats">
    <div class="stat-card"><div class="label">Total Executions</div><div class="value" id="stat-total">—</div></div>
    <div class="stat-card"><div class="label">Success Rate</div><div class="value" id="stat-rate">—</div></div>
    <div class="stat-card"><div class="label">Avg Duration</div><div class="value" id="stat-avg">—</div></div>
    <div class="stat-card"><div class="label">Skills Tracked</div><div class="value" id="stat-skills">—</div></div>
  </div>

  <div class="filters">
    <input type="text" id="skill-filter" placeholder="Filter by skill name..." onkeyup="if(event.key==='Enter')loadTraces()">
    <input type="text" id="date-filter" placeholder="Date (YYYY-MM-DD)">
    <select id="limit-select">
      <option value="25">25 records</option>
      <option value="50" selected>50 records</option>
      <option value="100">100 records</option>
      <option value="500">500 records</option>
    </select>
    <button onclick="loadTraces()">Refresh</button>
    <button onclick="loadStats()" style="background: var(--surface); border: 1px solid var(--border);">Reload Stats</button>
  </div>

  <div id="loading" class="loading">Loading traces...</div>
  <div id="content"></div>
</div>

<script>
const API = '/api/traces';
const STATS_API = '/api/traces/stats';

async function loadStats() {{
  try {{
    const resp = await fetch(STATS_API);
    if (!resp.ok) return;
    const s = await resp.json();
    document.getElementById('stat-total').textContent = s.total;
    const rate = s.success_rate.toFixed(1) + '%';
    document.getElementById('stat-rate').textContent = rate;
    document.getElementById('stat-rate').style.color = s.success_rate >= 80 ? 'var(--success)' : s.success_rate >= 50 ? 'var(--warn)' : 'var(--error)';
    document.getElementById('stat-avg').textContent = Math.round(s.avg_duration_ms) + 'ms';
    document.getElementById('stat-skills').textContent = Object.keys(s.skills).length;
  }} catch(e) {{ console.error(e); }}
}}

function formatDuration(ms) {{
  if (!ms) return '—';
  if (ms < 1000) return ms + 'ms';
  return (ms/1000).toFixed(2) + 's';
}}

async function loadTraces() {{
  const skill = document.getElementById('skill-filter').value.trim();
  const date = document.getElementById('date-filter').value.trim();
  const limit = document.getElementById('limit-select').value;

  let url = API + '?limit=' + limit;
  if (skill) url += '&skill=' + encodeURIComponent(skill);
  if (date) url += '&date=' + encodeURIComponent(date);

  document.getElementById('loading').style.display = 'block';
  document.getElementById('content').innerHTML = '';

  try {{
    const resp = await fetch(url);
    const traces = await resp.json();
    document.getElementById('loading').style.display = 'none';

    if (!traces || traces.length === 0) {{
      document.getElementById('content').innerHTML = '<div class="empty">No traces found. Execute some Skills to see data here.</div>';
      return;
    }}

    let html = `<table><thead><tr>
      <th>Skill</th><th>Started</th><th>Duration</th><th>Status</th><th>Nodes</th>
    </tr></thead><tbody>`;
    for (const t of traces) {{
      const statusCls = t.status === 'success' ? 'success' : 'error';
      const nodes = t.nodes || [];
      const nodeHtml = nodes.map(n =>
        `<span class="${{n.ok ? 'node-ok' : 'node-fail'}}">${{n.id}}${{n.ms ? '(' + formatDuration(n.ms) + ')' : ''}}</span>`
      ).join(' ');

      html += `<tr>
        <td><strong>${{t.skill || 'unknown'}}</strong></td>
        <td style="color:var(--muted)">${{t.started_at || '—'}}</td>
        <td>${{formatDuration(t.duration_ms)}}</td>
        <td><span class="status-badge ${{statusCls}}">${{t.status}}</span></td>
        <td><div class="node-list">${{nodeHtml || '—'}}</div></td>
      </tr>`;
    }}
    html += '</tbody></table>';
    document.getElementById('content').innerHTML = html;
  }} catch(e) {{
    document.getElementById('loading').style.display = 'none';
    document.getElementById('content').innerHTML = '<div class="empty">Failed to load traces: ' + e.message + '</div>';
  }}
}}

loadStats();
loadTraces();
setInterval(() => {{ loadStats(); loadTraces(); }}, 30000);
</script>
</body>
</html>"#
    )
}

/// Start the trace dashboard server on a given port
pub async fn start_dashboard(port: u16) -> Result<()> {
    let state = TraceDashboardState::new();

    let app = Router::new()
        .route("/", get(trace_dashboard))
        .route("/api/traces", get(list_traces))
        .route("/api/traces/stats", get(trace_stats))
        .route("/api/traces/:trace_id", get(get_trace))
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(port = port, addr = %addr, "trace dashboard listening");

    axum::serve(listener, app).await?;
    Ok(())
}
