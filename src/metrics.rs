//! Lightweight in-memory metrics for COGTOME runtime.
//!
//! Uses atomic counters and Mutex-protected hashmaps — no external dependencies.
//! Exposes metrics via `/metrics` JSON endpoint.

use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Global metrics state.
static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();

fn metrics() -> &'static Metrics {
    METRICS.get_or_init(|| Metrics {
        units: Mutex::new(HashMap::new()),
        requests: Mutex::new(HashMap::new()),
        foreach_iters: Mutex::new(HashMap::new()),
        unit_durations: Mutex::new(HashMap::new()),
        running_tasks: AtomicU64::new(0),
        start_time: Instant::now(),
    })
}

// ============================================================================
// Metric types
// ============================================================================

struct DurationStats {
    count: AtomicU64,
    sum_ms: AtomicU64,
}

struct Metrics {
    units: Mutex<HashMap<(String, String), AtomicU64>>,
    requests: Mutex<HashMap<(String, String), AtomicU64>>,
    foreach_iters: Mutex<HashMap<(String, String), AtomicU64>>,
    unit_durations: Mutex<HashMap<String, DurationStats>>,
    running_tasks: AtomicU64,
    start_time: Instant,
}

// ============================================================================
// Recording helpers (called from engine hot paths)
// ============================================================================

/// Record a successful unit execution.
#[inline]
pub fn record_unit_success(unit_name: &str, duration_secs: f64) {
    record_unit_result(unit_name, "success", duration_secs);
}

/// Record a failed unit execution.
#[inline]
pub fn record_unit_failure(unit_name: &str, status: &str, duration_secs: f64) {
    record_unit_result(unit_name, status, duration_secs);
}

fn record_unit_result(unit_name: &str, status: &str, duration_secs: f64) {
    let m = metrics();
    let ms = (duration_secs * 1000.0) as u64;

    // Increment unit counter
    {
        let mut units = m.units.lock().unwrap();
        let counter = units.entry((unit_name.to_string(), status.to_string()))
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    // Update duration stats
    {
        let mut durations = m.unit_durations.lock().unwrap();
        let stats = durations.entry(unit_name.to_string())
            .or_insert_with(|| DurationStats {
                count: AtomicU64::new(0),
                sum_ms: AtomicU64::new(0),
            });
        stats.count.fetch_add(1, Ordering::Relaxed);
        stats.sum_ms.fetch_add(ms, Ordering::Relaxed);
    }
}

/// Record an HTTP API request.
#[inline]
pub fn record_request(req_type: &str, status: &str) {
    let m = metrics();
    let mut requests = m.requests.lock().unwrap();
    let counter = requests.entry((req_type.to_string(), status.to_string()))
        .or_insert_with(|| AtomicU64::new(0));
    counter.fetch_add(1, Ordering::Relaxed);
}

/// Record a foreach iteration completion.
#[inline]
pub fn record_foreach_iteration(motif_name: &str, status: &str) {
    let m = metrics();
    let mut iters = m.foreach_iters.lock().unwrap();
    let counter = iters.entry((motif_name.to_string(), status.to_string()))
        .or_insert_with(|| AtomicU64::new(0));
    counter.fetch_add(1, Ordering::Relaxed);
}

/// Set the number of currently running tasks.
#[inline]
pub fn set_running_tasks(count: usize) {
    metrics().running_tasks.store(count as u64, Ordering::Relaxed);
}

/// Increment running tasks by 1.
#[inline]
pub fn inc_running_tasks() {
    metrics().running_tasks.fetch_add(1, Ordering::Relaxed);
}

/// Decrement running tasks by 1.
#[inline]
pub fn dec_running_tasks() {
    metrics().running_tasks.fetch_sub(1, Ordering::Relaxed);
}

// ============================================================================
// Snapshot for export
// ============================================================================

#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub uptime_secs: f64,
    pub units_executed: HashMap<String, HashMap<String, u64>>,
    pub unit_durations_ms: HashMap<String, DurationSnapshot>,
    pub requests: HashMap<String, HashMap<String, u64>>,
    pub foreach_iterations: HashMap<String, HashMap<String, u64>>,
    pub running_tasks: u64,
}

#[derive(Serialize)]
pub struct DurationSnapshot {
    pub count: u64,
    pub avg_ms: f64,
}

/// Take a snapshot of all metrics for export.
pub fn snapshot() -> MetricsSnapshot {
    let m = metrics();
    let uptime_secs = m.start_time.elapsed().as_secs_f64();

    let units_executed: HashMap<String, HashMap<String, u64>> = {
        let units = m.units.lock().unwrap();
        let mut result: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for ((name, status), counter) in units.iter() {
            result.entry(name.clone())
                .or_insert_with(HashMap::new)
                .insert(status.clone(), counter.load(Ordering::Relaxed));
        }
        result
    };

    let unit_durations_ms: HashMap<String, DurationSnapshot> = {
        let durations = m.unit_durations.lock().unwrap();
        durations.iter().map(|(name, stats)| {
            let count = stats.count.load(Ordering::Relaxed);
            let sum_ms = stats.sum_ms.load(Ordering::Relaxed);
            let avg_ms = if count > 0 { sum_ms as f64 / count as f64 } else { 0.0 };
            (name.clone(), DurationSnapshot { count, avg_ms })
        }).collect()
    };

    let requests: HashMap<String, HashMap<String, u64>> = {
        let reqs = m.requests.lock().unwrap();
        let mut result: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for ((rtype, status), counter) in reqs.iter() {
            result.entry(rtype.clone())
                .or_insert_with(HashMap::new)
                .insert(status.clone(), counter.load(Ordering::Relaxed));
        }
        result
    };

    let foreach_iterations: HashMap<String, HashMap<String, u64>> = {
        let iters = m.foreach_iters.lock().unwrap();
        let mut result: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for ((mname, status), counter) in iters.iter() {
            result.entry(mname.clone())
                .or_insert_with(HashMap::new)
                .insert(status.clone(), counter.load(Ordering::Relaxed));
        }
        result
    };

    let running_tasks = m.running_tasks.load(Ordering::Relaxed);

    MetricsSnapshot {
        uptime_secs,
        units_executed,
        unit_durations_ms,
        requests,
        foreach_iterations,
        running_tasks,
    }
}
