//! Benchmark Suite for COGTOME
//!
//! Measures core metrics: execution success rate, average duration, and Motif reuse rate.
//!
//! Run with: cargo test --release -- --nocapture benchmark
//!
//! The benchmark runs N iterations of a "simple skill" (text-uppercase) and records
//! success/failure/duration for each run.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Debug, Clone)]
struct BenchmarkResult {
    iteration: usize,
    success: bool,
    duration_ms: u64,
    error: Option<String>,
}

#[derive(Debug)]
struct BenchmarkSummary {
    total: usize,
    successes: usize,
    failures: usize,
    success_rate: f64,
    avg_duration_ms: f64,
    min_duration_ms: u64,
    max_duration_ms: u64,
    p50_duration_ms: u64,
    p95_duration_ms: u64,
    motif_reuse_rate: f64,
}

/// Locate the cogtome binary (release build preferred)
fn find_cogtome_binary() -> PathBuf {
    let candidates = vec![
        PathBuf::from("target/release/cogtome"),
        PathBuf::from("target/debug/cogtome"),
        PathBuf::from("./cogtome"),
    ];
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    // Try to build
    eprintln!("Note: cogtome binary not found, running `cargo build --release` first...");
    let _ = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(".")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    PathBuf::from("target/release/cogtome")
}

/// Run a single execution of a skill and measure duration
fn run_skill_once(binary: &PathBuf, skill: &str, input: &str) -> BenchmarkResult {
    let start = Instant::now();
    let output = Command::new(binary)
        .args(["run", skill, "--input", input])
        .current_dir(".")
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let _stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if out.status.success() {
                BenchmarkResult {
                    iteration: 0,
                    success: true,
                    duration_ms,
                    error: None,
                }
            } else {
                let err_msg = if stderr.contains("error") {
                    stderr.lines().find(|l| l.contains("error")).unwrap_or("execution failed").to_string()
                } else {
                    format!("exit code: {:?}", out.status.code())
                };
                BenchmarkResult {
                    iteration: 0,
                    success: false,
                    duration_ms,
                    error: Some(err_msg),
                }
            }
        }
        Err(e) => BenchmarkResult {
            iteration: 0,
            success: false,
            duration_ms,
            error: Some(e.to_string()),
        },
    }
}

/// Count how many skills share the same underlying Unit (Motif reuse indicator)
fn compute_motif_reuse_rate() -> f64 {
    let assemblies_dir = PathBuf::from("assemblies");
    if !assemblies_dir.exists() {
        return 0.0;
    }

    let mut unit_counts: HashMap<String, usize> = HashMap::new();
    let mut total_skills = 0;

    if let Ok(entries) = fs::read_dir(&assemblies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("manifest.json");
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                    total_skills += 1;
                    if let Some(units) = manifest.get("units").and_then(|u| u.as_array()) {
                        for unit in units {
                            if let Some(name) = unit.as_str() {
                                *unit_counts.entry(name.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if total_skills == 0 {
        return 0.0;
    }

    // Count skills that share at least one unit with another skill
    let mut skills_with_shared_unit = 0usize;
    if let Ok(entries) = fs::read_dir(&assemblies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }

            let manifest_path = path.join("manifest.json");
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(units) = manifest.get("units").and_then(|u| u.as_array()) {
                        let has_shared = units.iter().any(|u| {
                            if let Some(name) = u.as_str() {
                                unit_counts.get(name).copied().unwrap_or(1) > 1
                            } else {
                                false
                            }
                        });
                        if has_shared {
                            skills_with_shared_unit += 1;
                        }
                    }
                }
            }
        }
    }

    skills_with_shared_unit as f64 / total_skills as f64
}

fn summarize(results: Vec<BenchmarkResult>) -> BenchmarkSummary {
    let total = results.len();
    let successes = results.iter().filter(|r| r.success).count();
    let failures = total - successes;
    let success_rate = (successes as f64 / total as f64) * 100.0;

    let durations: Vec<u64> = results.iter().map(|r| r.duration_ms).collect();
    let mut sorted = durations.clone();
    sorted.sort();

    let avg = if total > 0 { durations.iter().sum::<u64>() as f64 / total as f64 } else { 0.0 };
    let min = sorted.first().copied().unwrap_or(0);
    let max = sorted.last().copied().unwrap_or(0);
    let p50_idx = (total as f64 * 0.50) as usize;
    let p95_idx = (total as f64 * 0.95) as usize;
    let p50 = sorted.get(p50_idx.min(total - 1)).copied().unwrap_or(0);
    let p95 = sorted.get(p95_idx.min(total - 1)).copied().unwrap_or(0);

    BenchmarkSummary {
        total,
        successes,
        failures,
        success_rate,
        avg_duration_ms: avg,
        min_duration_ms: min,
        max_duration_ms: max,
        p50_duration_ms: p50,
        p95_duration_ms: p95,
        motif_reuse_rate: compute_motif_reuse_rate(),
    }
}

fn print_summary(summary: &BenchmarkSummary) {
    println!();
    println!("{}", "═".repeat(70));
    println!("  COGTOME Benchmark Results");
    println!("{}", "═".repeat(70));
    println!();
    println!("  {:25} {:>8} / {:<8} ({:.1}%)",
        "Success Rate:", summary.successes, summary.total, summary.success_rate);
    println!("  {:25} {:>8}", "Failures:", summary.failures);
    println!();
    println!("  Duration Statistics:");
    println!("  {:25} {:>8.1} ms", "Average:", summary.avg_duration_ms);
    println!("  {:25} {:>8} ms", "Min:", summary.min_duration_ms);
    println!("  {:25} {:>8} ms", "Max:", summary.max_duration_ms);
    println!("  {:25} {:>8} ms", "P50:", summary.p50_duration_ms);
    println!("  {:25} {:>8} ms", "P95:", summary.p95_duration_ms);
    println!();
    println!("  {:25} {:>8.1}%", "Motif Reuse Rate:", summary.motif_reuse_rate * 100.0);
    println!();
    println!("{}", "═".repeat(70));
}

fn print_failures(results: &[BenchmarkResult]) {
    let failures: Vec<_> = results.iter().filter(|r| !r.success).collect();
    if failures.is_empty() {
        println!("  ✅ All runs succeeded!");
    } else {
        println!("  ❌ {} failure(s):", failures.len());
        for f in &failures {
            println!("    - Iteration {}: {} ({:?})", f.iteration, f.error.clone().unwrap_or_default(), f.duration_ms);
        }
    }
}

/// Main benchmark: run simple skill N times
/// Default: 100 iterations. Override with BENCHMARK_ITERATIONS env var.
fn run_benchmark(iterations: usize, skill: &str, input: &str) -> BenchmarkSummary {
    let binary = find_cogtome_binary();

    if !binary.exists() {
        eprintln!("Error: cogtome binary not found at {:?}", binary);
        std::process::exit(1);
    }

    println!("Running benchmark: {} iterations of skill '{}' using {:?}", iterations, skill, binary);
    println!("Skill input: {}", input);

    let mut results = Vec::with_capacity(iterations);

    for i in 0..iterations {
        if i % 10 == 0 || i == iterations - 1 {
            print!("\r  Progress: {}/{} ...", i, iterations);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        let mut result = run_skill_once(&binary, skill, input);
        result.iteration = i;
        results.push(result);
    }

    println!();
    let summary = summarize(results.clone());
    print_failures(&results);

    summary
}

/// Discovery benchmark: measure how fast COGTOME discovers all skills
fn benchmark_discovery(binary: &PathBuf) -> u64 {
    let start = Instant::now();
    let _ = Command::new(binary)
        .args(["discover"])
        .current_dir(".")
        .output();
    start.elapsed().as_millis() as u64
}

/// List available skills/units for benchmark targeting
fn list_available_skills() -> Vec<String> {
    let assemblies = PathBuf::from("assemblies");
    if !assemblies.exists() {
        return vec![];
    }
    let mut skills = Vec::new();
    if let Ok(entries) = fs::read_dir(&assemblies) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    skills.push(name.to_string());
                }
            }
        }
    }
    skills
}

#[test]
fn benchmark_simple_skill() {
    let iterations: usize = std::env::var("BENCHMARK_ITERATIONS")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    // Find a simple skill to benchmark
    let skill = if cfg!(test) {
        "text-uppercase".to_string()
    } else {
        std::env::var("BENCHMARK_SKILL").unwrap_or_else(|_| "text-uppercase".to_string())
    };

    let input = r#"{"text":"hello world benchmark"}"#;

    let summary = run_benchmark(iterations, &skill, input);
    print_summary(&summary);

    // Assertions
    assert!(summary.total == iterations, "Should run exactly {} iterations", iterations);
    assert!(summary.success_rate >= 0.0, "Success rate must be non-negative");

    if summary.total > 10 {
        assert!(
            summary.success_rate >= 90.0,
            "Success rate should be >= 90% (got {:.1}%)",
            summary.success_rate
        );
    }
}

#[test]
fn benchmark_discovery_speed() {
    let binary = find_cogtome_binary();
    if !binary.exists() {
        eprintln!("Skipping discovery benchmark: binary not found");
        return;
    }

    // Run discovery 5 times and average
    let runs = 5;
    let mut times = Vec::new();
    for _ in 0..runs {
        let ms = benchmark_discovery(&binary);
        times.push(ms);
    }

    let avg = times.iter().sum::<u64>() as f64 / runs as f64;
    let min = times.iter().min().copied().unwrap_or(0);
    let max = times.iter().max().copied().unwrap_or(0);

    println!();
    println!("Discovery Benchmark ({} runs): avg={:.1}ms, min={}ms, max={}ms", runs, avg, min, max);

    // Discovery should be reasonably fast
    assert!(avg < 5000.0, "Discovery should complete in < 5s (got {:.1}ms)", avg);
}

#[test]
fn benchmark_available_skills() {
    let skills = list_available_skills();
    println!();
    println!("Available skills for benchmarking:");
    for s in &skills {
        println!("  - {}", s);
    }
    assert!(!skills.is_empty(), "Should have at least one skill to benchmark");
}
