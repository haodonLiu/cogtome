use crate::context::{is_truthy, ExecContext, StepResult};
use crate::engine::motif_manifest::{
    AggregateBlock, AggregateMode, BackoffStrategy, ErrorStrategy, ForeachBlock,
    RetryConfig, StepErrorStrategy,
};
use crate::engine::unit_runner::UnitRunner;
use anyhow::{anyhow, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;

// ============================================================================
// Execution Events (Observability)
// ============================================================================

/// Emit a structured event to stderr in JSON Lines format.
pub fn emit_event(event_type: &str, data: serde_json::Map<String, Value>) {
    let mut event = data;
    event.insert("event".to_string(), Value::String(event_type.to_string()));
    event.insert("timestamp".to_string(), Value::String(timestamp_now()));
    if let Ok(json) = serde_json::to_string(&event) {
        eprintln!("{}", json);
    }
}

pub fn timestamp_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    format!("{}.{:09}", secs, nanos)
}

pub fn emit_step_start(step_name: &str, unit_name: Option<&str>) {
    let mut data = serde_json::Map::new();
    data.insert("step".to_string(), Value::String(step_name.to_string()));
    if let Some(u) = unit_name {
        data.insert("unit".to_string(), Value::String(u.to_string()));
    }
    emit_event("step_start", data);
}

pub fn emit_step_end(step_name: &str, duration_ms: u64, status: &str, exit_code: Option<i32>) {
    let mut data = serde_json::Map::new();
    data.insert("step".to_string(), Value::String(step_name.to_string()));
    data.insert("duration_ms".to_string(), Value::Number(duration_ms.into()));
    data.insert("status".to_string(), Value::String(status.to_string()));
    if let Some(code) = exit_code {
        data.insert("exit_code".to_string(), Value::Number(code.into()));
    }
    emit_event("step_end", data);
}

// ============================================================================
// Foreach Execution
// ============================================================================

/// Execute a unit call with retry and error handling strategies.
/// Returns (output, exit_code, step_failed) where step_failed indicates if the step errored.
pub async fn execute_unit_with_error_handling(
    runner: &UnitRunner,
    unit_name: &str,
    input: Value,
    retry: &Option<RetryConfig>,
    on_error: &Option<StepErrorStrategy>,
    fallback: &Option<Value>,
    env_whitelist: &Option<Vec<String>>,
) -> Result<(Value, i32, bool)> {
    let mut last_error = None;
    let strategy = on_error.unwrap_or(StepErrorStrategy::Fail);
    let retry_config = retry.as_ref();

    let mut attempt = 0;
    let max_attempts = retry_config.map(|r| r.max).unwrap_or(1);

    loop {
        match runner
            .call(
                unit_name,
                input.clone(),
                env_whitelist.as_ref().map(|v| v.as_slice()),
            )
            .await
        {
            Ok((output, exit_code)) => {
                return Ok((output, exit_code, false));
            }
            Err(e) => {
                last_error = Some(e);
                attempt += 1;

                if attempt >= max_attempts {
                    break;
                }

                if let Some(config) = retry_config {
                    let delay = match config.backoff {
                        BackoffStrategy::Exponential => 2u64.pow(attempt - 1).min(60),
                        BackoffStrategy::Linear => attempt as u64,
                    };
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                }
            }
        }
    }

    match strategy {
        StepErrorStrategy::Fail => {
            Err(last_error.unwrap_or_else(|| anyhow!("Unit '{}' failed", unit_name)))
        }
        StepErrorStrategy::Continue | StepErrorStrategy::Fallback => {
            let output = fallback.clone().unwrap_or(Value::Null);
            Ok((output, -1, true))
        }
    }
}

pub fn resolve_step_input(
    ctx: &ExecContext,
    input: &std::collections::HashMap<String, String>,
) -> Result<serde_json::Map<String, Value>> {
    let mut step_input = serde_json::Map::new();
    for (k, v) in input {
        let val = ctx.resolve_var(v).unwrap_or(Value::Null);
        step_input.insert(k.clone(), val);
    }
    Ok(step_input)
}

pub fn resolve_aggregate_item(aggregate: &AggregateBlock, ctx: &ExecContext) -> Result<Value> {
    if aggregate.map.is_empty() {
        return Ok(Value::Null);
    }

    let mut obj = serde_json::Map::new();
    for (k, v) in &aggregate.map {
        let val = ctx.resolve_var(v).unwrap_or(Value::Null);
        obj.insert(k.clone(), val);
    }
    Ok(Value::Object(obj))
}

pub fn apply_aggregate_mode(
    aggregate: &AggregateBlock,
    results: Vec<Value>,
) -> Result<Value> {
    match aggregate.mode {
        AggregateMode::Array => Ok(Value::Array(results)),
        AggregateMode::Object => {
            let mut obj = serde_json::Map::new();
            for (i, v) in results.into_iter().enumerate() {
                obj.insert(i.to_string(), v);
            }
            Ok(Value::Object(obj))
        }
        AggregateMode::Sum => {
            let total: f64 = results.iter().filter_map(|v| v.as_f64()).sum();
            Ok(serde_json::json!(total))
        }
        AggregateMode::Join => {
            let sep = aggregate
                .join
                .as_ref()
                .map(|j| j.separator.as_str())
                .unwrap_or("");
            let parts: Vec<String> = results
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            Ok(Value::String(parts.join(sep)))
        }
    }
}

pub fn empty_aggregate(aggregate: &AggregateBlock) -> Value {
    match aggregate.mode {
        AggregateMode::Array => serde_json::json!([]),
        AggregateMode::Object => serde_json::json!({}),
        AggregateMode::Sum => serde_json::json!(0),
        AggregateMode::Join => serde_json::json!(""),
    }
}

pub async fn execute_foreach_serial(
    foreach: &ForeachBlock,
    ctx: &ExecContext,
    runner: &UnitRunner,
    max_iterations_hard: u32,
) -> Result<Value> {
    let items = ctx
        .resolve_var(&foreach.over)
        .ok_or_else(|| anyhow!("foreach.over expression could not be resolved"))?
        .as_array()
        .ok_or_else(|| anyhow!("foreach.over must resolve to an array"))?
        .clone();

    if items.is_empty() {
        return Ok(empty_aggregate(&foreach.aggregate));
    }

    let max_iter = foreach.max_iterations.min(max_iterations_hard);
    if items.len() > max_iter as usize {
        anyhow::bail!(
            "Foreach attempted {} iterations (limit: {}). Hint: Increase max_iterations or batch process.",
            items.len(),
            max_iter
        );
    }

    let mut results: Vec<Value> = Vec::new();

    for (idx, item) in items.into_iter().enumerate() {
        let sub_ctx = ctx.fork_for_iteration(foreach.as_var.clone(), item.clone(), idx);

        let mut step_ctx = sub_ctx;
        let mut success = true;
        let mut error_msg = None;

        for step in &foreach.flow {
            if let Some(cond) = &step.if_cond {
                let val = step_ctx.resolve_var(cond).unwrap_or(Value::Null);
                if !is_truthy(&val) {
                    continue;
                }
            }

            let input = resolve_step_input(&step_ctx, &step.input)?;

            if let Some(unit_name) = &step.unit {
                match execute_unit_with_error_handling(
                    runner,
                    unit_name,
                    Value::Object(input),
                    &step.retry,
                    &step.on_error,
                    &step.fallback,
                    &step.env_whitelist,
                )
                .await
                {
                    Ok((output, exit_code, _failed)) => {
                        step_ctx = step_ctx.with_local_step(
                            step.name.clone(),
                            StepResult {
                                output,
                                exit_code,
                            },
                        );
                    }
                    Err(e) => {
                        success = false;
                        error_msg = Some(e.to_string());
                        break;
                    }
                }
            }
        }

        if success {
            let aggregated = resolve_aggregate_item(&foreach.aggregate, &step_ctx)?;
            results.push(aggregated);
        } else if foreach.on_error == ErrorStrategy::Continue {
            let mut aggregated = resolve_aggregate_item(&foreach.aggregate, &step_ctx)?;
            if let Some(obj) = aggregated.as_object_mut() {
                obj.insert(
                    "__error".to_string(),
                    Value::String(error_msg.unwrap_or_default()),
                );
            } else {
                let mut err_obj = serde_json::Map::new();
                err_obj.insert(
                    "__error".to_string(),
                    Value::String(error_msg.unwrap_or_default()),
                );
                aggregated = Value::Object(err_obj);
            }
            results.push(aggregated);
        }
    }

    apply_aggregate_mode(&foreach.aggregate, results)
}

pub async fn execute_foreach_parallel(
    foreach: &ForeachBlock,
    ctx: &ExecContext,
    runner: &UnitRunner,
    max_iterations_hard: u32,
) -> Result<Value> {
    let items = ctx
        .resolve_var(&foreach.over)
        .ok_or_else(|| anyhow!("foreach.over expression could not be resolved"))?
        .as_array()
        .ok_or_else(|| anyhow!("foreach.over must resolve to an array"))?
        .clone();

    if items.is_empty() {
        return Ok(empty_aggregate(&foreach.aggregate));
    }

    let max_iter = foreach.max_iterations.min(max_iterations_hard);
    if items.len() > max_iter as usize {
        anyhow::bail!(
            "Foreach attempted {} iterations (limit: {}). Hint: Increase max_iterations or batch process.",
            items.len(),
            max_iter
        );
    }

    let max_concurrent = std::env::var("COGTOME_MAX_CONCURRENT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(50)
        .min(items.len())
        .max(1);
    let concurrency_limiter = Arc::new(Semaphore::new(max_concurrent));

    let cancel_token = CancellationToken::new();
    let fail_fast_flag = Arc::new(AtomicBool::new(false));
    let first_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let mut futures = FuturesUnordered::new();

    for (idx, item) in items.into_iter().enumerate() {
        let foreach_clone = foreach.clone();
        let ctx_clone = ctx.clone();
        let runner_clone = runner.clone();
        let cancel_clone = cancel_token.clone();
        let ff_flag = fail_fast_flag.clone();
        let ff_error = first_error.clone();
        let permit_clone = concurrency_limiter.clone();

        futures.push(async move {
            let _permit = permit_clone.acquire().await.ok();

            if cancel_clone.is_cancelled() {
                return None;
            }

            let sub_ctx = ctx_clone.fork_for_iteration(
                foreach_clone.as_var.clone(),
                item.clone(),
                idx,
            );

            let mut step_ctx = sub_ctx;
            let mut success = true;
            let mut error_msg = None;

            for step in &foreach_clone.flow {
                if cancel_clone.is_cancelled() {
                    return None;
                }

                if let Some(cond) = &step.if_cond {
                    let val = step_ctx.resolve_var(cond).unwrap_or(Value::Null);
                    if !is_truthy(&val) {
                        continue;
                    }
                }

                let input = match resolve_step_input(&step_ctx, &step.input) {
                    Ok(i) => i,
                    Err(e) => {
                        success = false;
                        error_msg = Some(e.to_string());
                        if foreach_clone.on_error == ErrorStrategy::FailFast {
                            ff_flag.store(true, Ordering::SeqCst);
                            let mut err = ff_error.lock().await;
                            if err.is_none() {
                                *err = Some(e.to_string());
                            }
                            cancel_clone.cancel();
                        }
                        break;
                    }
                };

                if let Some(ref unit_name) = step.unit {
                    let result = execute_unit_with_error_handling(
                        &runner_clone,
                        unit_name,
                        Value::Object(input),
                        &step.retry,
                        &step.on_error,
                        &step.fallback,
                        &step.env_whitelist,
                    )
                    .await;

                    match result {
                        Ok((output, exit_code, _failed)) => {
                            step_ctx = step_ctx.with_local_step(
                                step.name.clone(),
                                StepResult { output, exit_code },
                            );
                        }
                        Err(e) => {
                            success = false;
                            error_msg = Some(e.to_string());

                            if foreach_clone.on_error == ErrorStrategy::FailFast {
                                ff_flag.store(true, Ordering::SeqCst);
                                let mut err = ff_error.lock().await;
                                if err.is_none() {
                                    *err = Some(e.to_string());
                                }
                                cancel_clone.cancel();
                                return None;
                            }
                            break;
                        }
                    }
                }
            }

            if success {
                match resolve_aggregate_item(&foreach_clone.aggregate, &step_ctx) {
                    Ok(v) => Some(Some(v)),
                    Err(_) => Some(None),
                }
            } else if foreach_clone.on_error == ErrorStrategy::Continue {
                match resolve_aggregate_item(&foreach_clone.aggregate, &step_ctx) {
                    Ok(mut agg) => {
                        if let Some(obj) = agg.as_object_mut() {
                            if let Some(msg) = error_msg {
                                obj.insert("__error".to_string(), Value::String(msg));
                            }
                        }
                        Some(Some(agg))
                    }
                    Err(_) => Some(None),
                }
            } else {
                Some(None)
            }
        });
    }

    let mut results: Vec<Value> = Vec::new();

    while let Some(result) = futures.next().await {
        if let Some(Some(v)) = result {
            results.push(v);
        }
    }

    if fail_fast_flag.load(Ordering::SeqCst) {
        let err = first_error.lock().await;
        if let Some(e) = err.as_ref() {
            anyhow::bail!("Parallel foreach failed (fail_fast): {}", e);
        }
    }

    apply_aggregate_mode(&foreach.aggregate, results)
}
