use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use super::expression::{eval_condition, eval_expression, is_truthy};

#[derive(Debug, Clone)]
pub struct ExecContext {
    pub params: Value,
    pub locals: HashMap<String, Value>,   // 迭代变量 (item, __index, as_var)
    pub steps: Arc<HashMap<String, StepResult>>,  // O(1) clone via Arc
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
            locals: HashMap::new(),
            steps: Arc::new(HashMap::new()),
        }
    }

    /// 变量解析优先级：
    /// 1. locals (迭代变量: item, __index, as_var)
    /// 2. steps (先局部后快照 - 注意 foreach 内不可修改外部 steps)
    /// 3. params (用户输入)
    /// 4. env (环境变量)
    pub fn resolve_var(&self, expr: &str) -> Option<Value> {
        let expr = expr.trim();

        // 三目运算符检测
        if let Some(ternary_result) = self.try_ternary(expr) {
            return Some(ternary_result);
        }

        if expr.starts_with("${") && expr.ends_with("}") {
            let inner = expr[2..expr.len() - 1].trim();
            return self.evaluate_inner(inner);
        }

        // 非模板表达式，原样返回字符串
        Some(Value::String(expr.to_string()))
    }

    /// 创建子上下文（foreach 迭代用）- O(1) 快照
    pub fn fork_for_iteration(&self, iteration_var: String, iteration_value: Value, index: usize) -> Self {
        let mut locals = self.locals.clone();
        locals.insert(iteration_var, iteration_value.clone());
        locals.insert("__index".to_string(), serde_json::json!(index));
        locals.insert("item".to_string(), iteration_value);  // item 作为默认迭代变量名

        Self {
            params: self.params.clone(),
            locals,
            steps: Arc::clone(&self.steps),  // O(1) 克隆
        }
    }

    /// 在子上下文中记录 step 结果（copy-on-write）
    pub fn with_local_step(&self, name: String, result: StepResult) -> Self {
        let mut new_steps = (*self.steps).clone();  // 首次写入时深拷贝
        new_steps.insert(name, result);
        Self {
            params: self.params.clone(),
            locals: self.locals.clone(),
            steps: Arc::new(new_steps),
        }
    }

    fn try_ternary(&self, expr: &str) -> Option<Value> {
        let mut depth = 0;
        let mut q_pos = None;
        let mut c_pos = None;

        for (i, ch) in expr.char_indices() {
            match ch {
                '?' if depth == 0 && q_pos.is_none() => q_pos = Some(i),
                ':' if depth == 0 && q_pos.is_some() && c_pos.is_none() => c_pos = Some(i),
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }

        let q_pos = q_pos?;
        let c_pos = c_pos?;

        let condition = expr[..q_pos].trim();
        let true_val = expr[q_pos + 1..c_pos].trim();
        let false_val = expr[c_pos + 1..].trim();

        let cond_val = self.resolve_var(condition)?;
        if is_truthy(&cond_val) {
            self.resolve_var(true_val)
        } else {
            self.resolve_var(false_val)
        }
    }

    fn evaluate_inner(&self, inner: &str) -> Option<Value> {
        // 0. 函数调用检测 (filter, map)
        if let Some((fn_name, args)) = Self::parse_function_call(inner) {
            return self.eval_function(fn_name, args);
        }

        // 1. 查找 locals (迭代变量)
        if let Some(val) = self.locals.get(inner) {
            return Some(val.clone());
        }

        let parts: Vec<&str> = inner.split('.').collect();

        // 2. params 访问
        if parts.len() >= 2 && parts[0] == "params" {
            return Self::resolve_nested(&self.params, &parts[1..]);
        }

        // 3. steps 访问
        if parts.len() >= 3 && parts[0] == "steps" {
            let step_name = parts[1];
            if let Some(step) = self.steps.get(step_name) {
                if parts[2] == "output" {
                    if parts.len() > 3 {
                        return Self::resolve_nested(&step.output, &parts[3..]);
                    }
                    return Some(step.output.clone());
                } else if parts[2] == "exit_code" {
                    return Some(Value::Number(step.exit_code.into()));
                }
            }
            return None;
        }

        // 4. env 访问
        if parts.len() == 2 && parts[0] == "env" {
            return std::env::var(parts[1]).ok().map(Value::String);
        }

        None
    }

    /// 解析函数调用，返回 (函数名, 参数列表)
    fn parse_function_call(inner: &str) -> Option<(&str, Vec<&str>)> {
        // 快速检测：必须有括号且不在字符串字面量中
        let open_paren = inner.find('(')?;
        let close_paren = inner.rfind(')')?;
        if close_paren <= open_paren {
            return None;
        }

        let fn_name = inner[..open_paren].trim();
        if fn_name.is_empty() {
            return None;
        }

        let args_str = &inner[open_paren + 1..close_paren];
        let args = Self::split_args(args_str);

        Some((fn_name, args))
    }

    /// 分割函数参数，处理嵌套括号和引号
    fn split_args(args_str: &str) -> Vec<&str> {
        let mut args = Vec::new();
        let mut depth: i32 = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut start = 0;

        for (i, ch) in args_str.char_indices() {
            match ch {
                '\'' | '"' if !in_string => {
                    in_string = true;
                    string_char = ch;
                }
                '\'' | '"' if in_string && ch == string_char => {
                    in_string = false;
                }
                '(' | '[' | '{' if !in_string => depth += 1,
                ')' | ']' | '}' if !in_string => depth = depth.saturating_sub(1),
                ',' if !in_string && depth == 0 => {
                    args.push(args_str[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }

        let last = args_str[start..].trim();
        if !last.is_empty() {
            args.push(last);
        }

        args
    }

    /// 评估函数调用
    fn eval_function(&self, fn_name: &str, args: Vec<&str>) -> Option<Value> {
        match fn_name {
            "filter" => self.eval_filter(args),
            "map" => self.eval_map(args),
            _ => None,
        }
    }

    /// filter(array, 'condition_expr') - 过滤数组
    fn eval_filter(&self, args: Vec<&str>) -> Option<Value> {
        if args.len() != 2 {
            return None;
        }

        let array_expr = args[0];
        let condition = args[1].trim_matches('"').trim_matches('\'');

        let array = self.resolve_var(array_expr)?;
        let array = array.as_array()?;

        let filtered: Vec<Value> = array
            .iter()
            .enumerate()
            .filter(|(idx, item)| {
                eval_condition(condition, item, *idx).unwrap_or(false)
            })
            .map(|(_, item)| item.clone())
            .collect();

        Some(Value::Array(filtered))
    }

    /// map(array, 'expression') - 映射数组
    fn eval_map(&self, args: Vec<&str>) -> Option<Value> {
        if args.len() != 2 {
            return None;
        }

        let array_expr = args[0];
        let expression = args[1].trim_matches('"').trim_matches('\'');

        let array = self.resolve_var(array_expr)?;
        let array = array.as_array()?;

        let mapped: Vec<Value> = array
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                eval_expression(expression, item, idx).unwrap_or(Value::Null)
            })
            .collect();

        Some(Value::Array(mapped))
    }

    /// 支持 field[index] 和 field[-1] 以及 field.length
    pub fn resolve_nested(value: &Value, path: &[&str]) -> Option<Value> {
        if path.is_empty() {
            return Some(value.clone());
        }

        let mut current = value.clone();

        for part in path {
            // 检查 .length 属性
            if *part == "length" {
                if let Some(arr) = current.as_array() {
                    return Some(Value::Number(arr.len().into()));
                }
                if let Some(s) = current.as_str() {
                    return Some(Value::Number(s.len().into()));
                }
                return None;
            }

            // 检查数组索引 [index] 或 [-1]
            if let Some(open) = part.find('[') {
                let field = &part[..open];
                let close = part.find(']')?;
                let idx_str = &part[open + 1..close];
                let idx: i64 = idx_str.parse().ok()?;

                if !field.is_empty() {
                    current = current.get(field)?.clone();
                }

                if let Some(arr) = current.as_array() {
                    let actual_idx = if idx < 0 {
                        arr.len() as i64 + idx
                    } else {
                        idx
                    };
                    if actual_idx < 0 || (actual_idx as usize) >= arr.len() {
                        return None;
                    }
                    current = arr[actual_idx as usize].clone();
                } else {
                    return None;
                }
            } else {
                current = current.get(*part)?.clone();
            }
        }

        Some(current)
    }
}
