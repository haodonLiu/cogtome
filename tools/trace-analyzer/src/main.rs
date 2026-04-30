//! trace-analyzer: reads COGTOME execution traces, produces Agent-readable summary.
//!
//! Input (stdin JSON):
//!   { "skill_name": "daily-summary", "days": 7, "trace_dir": "~/.cogtome/traces" }
//!
//! Output (stdout JSON):
//!   { "skill": "...", "executions": N, "success_rate": 0.X, "avg_duration_ms": N,
//!     "p95_duration_ms": N, "error_types": [...], "slowest_nodes": [...], "suggestions": [...] }

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let input: serde_json::Value = serde_json::from_reader(std::io::stdin()).unwrap();
    let trace_dir = expand_path(
        input
            .get("trace_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("~/.cogtome/traces"),
    );
    let skill_name = input.get("skill_name").and_then(|v| v.as_str());
    let days = input
        .get("days")
        .and_then(|v| v.as_i64())
        .unwrap_or(7) as i64;

    let cutoff = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        - days as i64 * 86400;

    // Collect matching .jsonl files
    let mut trace_files: Vec<(String, PathBuf)> = Vec::new();

    if let Some(skill) = skill_name {
        let skill_path = trace_dir.join(skill);
        if skill_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&skill_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                        let date_str = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        trace_files.push((date_str.to_string(), path));
                    }
                }
            }
        }
    } else if trace_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&trace_dir) {
            for entry in entries.flatten() {
                let skill_path = entry.path();
                if skill_path.is_dir() {
                    let skill = skill_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if let Ok(sub_entries) = fs::read_dir(&skill_path) {
                        for sub in sub_entries.flatten() {
                            let path = sub.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                                let date_str = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                                trace_files.push((format!("{}/{}", skill, date_str), path));
                            }
                        }
                    }
                }
            }
        }
    }

    // Filter by date and parse traces
    let mut traces: Vec<TraceRecord> = Vec::new();
    for (_, file_path) in &trace_files {
        let date_str = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if let Ok(date_parsed) = parse_date(date_str) {
            if date_parsed < cutoff {
                continue;
            }
        }
        if let Ok(file) = File::open(file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Ok(record) = serde_json::from_str::<TraceRecord>(&line) {
                    traces.push(record);
                }
            }
        }
    }

    if traces.is_empty() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "skill": skill_name.unwrap_or("all"),
                "executions": 0,
                "suggestions": vec!["No trace data found for the requested period. Keep executing Skills to accumulate data."]
            })).unwrap()
        );
        return;
    }

    let total = traces.len();
    let successes: usize = traces.iter().filter(|t| t.status == "success").count();
    let mut durations: Vec<u64> = traces.iter().map(|t| t.duration_ms).collect();
    durations.sort();

    let avg_ms = durations.iter().sum::<u64>() / total as u64;
    let p95_idx = ((total as f64) * 0.95) as usize;
    let p95_ms = durations[p95_idx.min(total - 1)];

    // Error types
    let mut error_counts: HashMap<String, usize> = HashMap::new();
    for t in &traces {
        if t.status != "success" {
            let err = t
                .nodes
                .last()
                .and_then(|n| n.error.as_ref())
                .filter(|e| !e.is_empty())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            *error_counts.entry(err).or_insert(0) += 1;
        }
    }
    let mut error_types: Vec<_> = error_counts
        .into_iter()
        .map(|(t, c)| serde_json::json!({ "type": t, "count": c }))
        .collect();
    error_types.sort_by(|a, b| {
        b["count"]
            .as_u64()
            .unwrap()
            .cmp(&a["count"].as_u64().unwrap())
    });

    // Node stats
    let mut node_hits: HashMap<String, usize> = HashMap::new();
    let mut node_fails: HashMap<String, usize> = HashMap::new();
    let mut node_ms: HashMap<String, Vec<u64>> = HashMap::new();

    for t in &traces {
        for n in &t.nodes {
            *node_hits.entry(n.id.clone()).or_insert(0) += 1;
            if !n.ok {
                *node_fails.entry(n.id.clone()).or_insert(0) += 1;
            }
            node_ms
                .entry(n.id.clone())
                .or_insert_with(Vec::new)
                .push(n.ms.unwrap_or(0));
        }
    }

    let mut node_stats: Vec<_> = node_hits
        .iter()
        .map(|(id, hits)| {
            let fails = node_fails.get(id).copied().unwrap_or(0);
            let avg_n = node_ms
                .get(id)
                .map(|v| v.iter().sum::<u64>() / v.len() as u64)
                .unwrap_or(0);
            serde_json::json!({ "node": id, "hits": hits, "failures": fails, "avg_ms": avg_n })
        })
        .collect();
    node_stats.sort_by(|a, b| {
        b["hits"]
            .as_u64()
            .unwrap()
            .cmp(&a["hits"].as_u64().unwrap())
    });

    let mut slowest: Vec<_> = node_stats.iter().cloned().collect();
    slowest.sort_by(|a, b| {
        b["avg_ms"]
            .as_u64()
            .unwrap()
            .cmp(&a["avg_ms"].as_u64().unwrap())
    });
    slowest.truncate(3);

    // Suggestions
    let mut suggestions: Vec<String> = Vec::new();
    let success_rate = successes as f64 / total as f64;
    if success_rate < 0.8 {
        suggestions.push(format!(
            "Success rate is only {}/{} ({:.0}%). Investigate errors first.",
            successes, total, success_rate * 100.0
        ));
    }
    if p95_ms > 5000 {
        suggestions.push(format!(
            "P95 latency is {}ms — consider adding caching or optimizing slow steps.",
            p95_ms
        ));
    }
    if let Some(first_err) = error_types.first() {
        suggestions.push(format!(
            "Top error: '{}' occurred {} times — worth adding a retry or fallback handler.",
            first_err["type"], first_err["count"]
        ));
    }
    for ns in &slowest {
        if let Some(ms) = ns["avg_ms"].as_u64() {
            if ms > 1000 {
                suggestions.push(format!("Node '{}' averages {}ms — review if necessary.", ns["node"], ms));
            }
        }
    }
    if traces.len() < 3 {
        suggestions.push(format!(
            "Only {} execution(s) recorded. More data needed for reliable patterns.",
            traces.len()
        ));
    }

    let output = serde_json::json!({
        "skill": skill_name.unwrap_or("all"),
        "executions": total,
        "success_rate": ((successes as f64 / total as f64) * 1000.0).round() / 1000.0,
        "avg_duration_ms": avg_ms,
        "p95_duration_ms": p95_ms,
        "error_types": error_types,
        "node_stats": &node_stats[..node_stats.len().min(10)],
        "slowest_nodes": slowest,
        "suggestions": suggestions
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

// ───────────────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct TraceRecord {
    #[serde(rename = "trace_id")]
    _trace_id: Option<String>,
    skill: Option<String>,
    #[serde(rename = "started_at")]
    _started_at: Option<String>,
    #[serde(rename = "duration_ms")]
    duration_ms: u64,
    status: String,
    nodes: Vec<NodeTrace>,
}

#[derive(Debug, serde::Deserialize)]
struct NodeTrace {
    id: String,
    #[serde(rename = "type")]
    _type: Option<String>,
    ok: bool,
    ms: Option<u64>,
    error: Option<String>,
}

fn expand_path(s: &str) -> PathBuf {
    if s.starts_with('~') {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(&s[1..]))
            .unwrap_or_else(|_| PathBuf::from(s))
    } else {
        PathBuf::from(s)
    }
}

fn parse_date(s: &str) -> Result<i64, &'static str> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 { return Err("invalid date format"); }
    let year: i64 = parts[0].parse().map_err(|_| "invalid year")?;
    let month: i64 = parts[1].parse().map_err(|_| "invalid month")?;
    let day: i64 = parts[2].parse().map_err(|_| "invalid day")?;
    Ok(days_from_ymd(year, month, day) * 86400)
}

fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_ymd(year: i64, month: i64, day: i64) -> i64 {
    let days_before_month: [i64; 12] =
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap_extra = if is_leap(year) && month > 2 {
        1
    } else {
        0
    };
    (year - 1970) * 365
        + (year - 1969) / 4
        - (year - 1901) / 100
        + (year - 1601) / 400
        + days_before_month[(month - 1) as usize]
        + leap_extra
        + day
        - 1
}
