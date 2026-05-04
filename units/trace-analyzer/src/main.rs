//! trace-analyzer — analyzes COGTOME trace JSONL files and produces summary stats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct Input {
    #[serde(default)]
    skill_name: Option<String>,
    #[serde(default = "default_days")]
    days: u32,
    #[serde(default = "default_trace_dir")]
    trace_dir: String,
}

fn default_days() -> u32 { 7 }
fn default_trace_dir() -> String { "~/.cogtome/traces".to_string() }

#[derive(Debug, Clone, Deserialize)]
struct TraceRecord {
    #[serde(rename = "trace_id")]
    trace_id: String,
    skill: String,
    #[serde(rename = "started_at")]
    started_at: String,
    #[serde(rename = "duration_ms")]
    duration_ms: u64,
    status: String,
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, Deserialize)]
struct Node {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    ok: bool,
    ms: Option<u64>,
    error: Option<String>,
    branch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Output {
    skill: String,
    executions: usize,
    success_rate: f64,
    avg_duration_ms: f64,
    p95_duration_ms: f64,
    error_types: Vec<ErrorType>,
    node_stats: Vec<NodeStat>,
    slowest_nodes: Vec<SlowestNode>,
    suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ErrorType {
    #[serde(rename = "type")]
    error_type: String,
    count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NodeStat {
    id: String,
    count: usize,
    #[serde(rename = "avg_ms")]
    avg_ms: f64,
    #[serde(rename = "fail_count")]
    fail_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SlowestNode {
    id: String,
    #[serde(rename = "avg_ms")]
    avg_ms: f64,
}

fn resolve_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(path.trim_start_matches("~/")))
            .unwrap_or_else(|_| PathBuf::from(path))
    } else {
        PathBuf::from(path)
    }
}

fn get_trace_files(trace_dir: &PathBuf, skill_name: &Option<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let base = trace_dir;

    let skills: Vec<String> = if let Some(ref s) = skill_name {
        vec![s.clone()]
    } else {
        std::fs::read_dir(base)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    };

    for skill in skills {
        let skill_dir = base.join(&skill);
        if !skill_dir.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&skill_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
    }
    files
}

fn load_traces(files: &[PathBuf]) -> Vec<TraceRecord> {
    let mut records = Vec::new();
    for path in files {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().filter_map(|l| l.ok()) {
                if let Ok(record) = serde_json::from_str::<TraceRecord>(&line) {
                    records.push(record);
                }
            }
        }
    }
    records
}

fn compute_p95(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    if idx == 0 { return 0.0; }
    let idx = (idx - 1).min(sorted.len() - 1);
    sorted[idx] as f64
}

fn analyze(records: &[TraceRecord]) -> Output {
    if records.is_empty() {
        return Output {
            skill: String::new(),
            executions: 0,
            success_rate: 0.0,
            avg_duration_ms: 0.0,
            p95_duration_ms: 0.0,
            error_types: vec![],
            node_stats: vec![],
            slowest_nodes: vec![],
            suggestions: vec!["No trace data found. Run some skills first.".to_string()],
        };
    }

    let n = records.len();
    let skill = records[0].skill.clone();

    let successes = records.iter().filter(|r| r.status == "success").count();
    let success_rate = successes as f64 / n as f64;

    let durations: Vec<u64> = records.iter().map(|r| r.duration_ms).collect();
    let avg_duration_ms = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<u64>() as f64 / durations.len() as f64
    };
    let p95_duration_ms = compute_p95(&durations);

    let mut error_counts: HashMap<String, usize> = HashMap::new();
    for r in records {
        if r.status != "success" {
            for node in &r.nodes {
                if !node.ok {
                    let err = node.error.clone().unwrap_or_else(|| "unknown".to_string());
                    *error_counts.entry(err).or_insert(0) += 1;
                }
            }
        }
    }
    let mut error_types: Vec<ErrorType> = error_counts
        .into_iter()
        .map(|(error_type, count)| ErrorType { error_type, count })
        .collect();
    error_types.sort_by(|a, b| b.count.cmp(&a.count));

    let mut node_data: HashMap<String, (usize, u64, usize)> = HashMap::new();
    for r in records {
        for node in &r.nodes {
            let entry = node_data.entry(node.id.clone()).or_insert((0, 0, 0));
            entry.0 += 1;
            if let Some(ms) = node.ms {
                entry.1 += ms;
            }
            if !node.ok {
                entry.2 += 1;
            }
        }
    }
    let mut node_stats: Vec<NodeStat> = node_data
        .iter()
        .map(|(id, (count, total_ms, fail_count))| {
            let avg = if *count > 0 { *total_ms as f64 / *count as f64 } else { 0.0 };
            NodeStat {
                id: id.clone(),
                count: *count,
                avg_ms: (avg * 10.0).round() / 10.0,
                fail_count: *fail_count,
            }
        })
        .collect();
    node_stats.sort_by(|a, b| b.count.cmp(&a.count));

    let mut slowest: Vec<SlowestNode> = node_stats
        .iter()
        .filter(|n| n.avg_ms > 0.0)
        .take(5)
        .map(|n| SlowestNode {
            id: n.id.clone(),
            avg_ms: n.avg_ms,
        })
        .collect();
    slowest.sort_by(|a, b| b.avg_ms.partial_cmp(&a.avg_ms).unwrap_or(std::cmp::Ordering::Equal));

    let mut suggestions = Vec::new();
    if success_rate < 0.9 {
        let pct = (success_rate * 100.0).round() as i32;
        suggestions.push(format!("Success rate is {}%. Investigate failing nodes.", pct));
    }
    if let Some(first) = error_types.first() {
        suggestions.push(format!("Most common error: '{}' ({} occurrences)", first.error_type, first.count));
    }
    for node in &slowest {
        if node.avg_ms > 1000.0 {
            suggestions.push(format!("Node '{}' is slow (avg {:.0}ms). Consider optimization.", node.id, node.avg_ms));
        }
    }
    if suggestions.is_empty() {
        suggestions.push("No obvious issues detected. Execution looks healthy.".to_string());
    }

    Output {
        skill,
        executions: n,
        success_rate: (success_rate * 1000.0).round() / 1000.0,
        avg_duration_ms: (avg_duration_ms * 10.0).round() / 10.0,
        p95_duration_ms: (p95_duration_ms * 10.0).round() / 10.0,
        error_types,
        node_stats,
        slowest_nodes: slowest,
        suggestions,
    }
}

fn main() {
    let input: Input = match serde_json::from_reader(std::io::stdin()) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to parse input: {}", e);
            std::process::exit(1);
        }
    };

    let trace_dir = resolve_path(&input.trace_dir);
    let files = get_trace_files(&trace_dir, &input.skill_name);
    let records = load_traces(&files);
    let output = analyze(&records);

    match std::io::stdout().write_all(serde_json::to_string_pretty(&output).unwrap().as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to write output: {}", e);
            std::process::exit(1);
        }
    }
}
