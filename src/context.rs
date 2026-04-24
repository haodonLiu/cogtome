use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ExecContext {
    pub params: Value,
    pub steps: HashMap<String, StepResult>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub output: Value,
    pub exit_code: i32,
}

impl ExecContext {
    pub fn new(params: Value) -> Self {
        Self {
            params,
            steps: HashMap::new(),
        }
    }

    /// 解析变量表达式：${params.x}, ${steps.name.output.x}, ${steps.name.exit_code}, ${env.HOME}
    pub fn resolve_var(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();
        if expr.starts_with("${") && expr.ends_with("}") {
            let inner = expr[2..expr.len() - 1].trim();
            let parts: Vec<&str> = inner.split('.').collect();

            if parts.len() >= 2 && parts[0] == "params" {
                return Self::get_nested(&self.params, &parts[1..]);
            }

            if parts.len() >= 3 && parts[0] == "steps" {
                let step_name = parts[1];
                if let Some(step) = self.steps.get(step_name) {
                    if parts[2] == "output" && parts.len() > 3 {
                        return Self::get_nested(&step.output, &parts[3..]);
                    } else if parts[2] == "exit_code" {
                        return Some(Value::Number(step.exit_code.into()));
                    }
                }
                return None;
            }

            if parts.len() == 2 && parts[0] == "env" {
                return std::env::var(parts[1]).ok().map(Value::String);
            }
        }

        // 非模板表达式，原样返回字符串
        Some(Value::String(expr.to_string()))
    }

    fn get_nested(value: &Value, path: &[&str]) -> Option<Value> {
        let mut current = value;
        for key in path {
            current = current.get(key)?;
        }
        Some(current.clone())
    }
}
