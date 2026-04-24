use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

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
                Self::eval_condition(condition, item, *idx).unwrap_or(false)
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
                Self::eval_expression(expression, item, idx).unwrap_or(Value::Null)
            })
            .collect();

        Some(Value::Array(mapped))
    }

    /// 评估布尔条件表达式，支持 ==, !=, >, <, >=, <=, &&, ||
    fn eval_condition(condition: &str, item: &Value, idx: usize) -> Option<bool> {
        let condition = condition.trim();

        // Handle && and ||
        if let Some(pos) = Self::find_top_level_op(condition, "&&") {
            let left = &condition[..pos];
            let right = &condition[pos + 2..];
            return Some(
                Self::eval_condition(left, item, idx)? && Self::eval_condition(right, item, idx)?
            );
        }
        if let Some(pos) = Self::find_top_level_op(condition, "||") {
            let left = &condition[..pos];
            let right = &condition[pos + 2..];
            return Some(
                Self::eval_condition(left, item, idx)? || Self::eval_condition(right, item, idx)?
            );
        }

        // Handle comparisons
        for &(op, _) in &[
            ("==", 2), ("!=", 2), (">=", 2), ("<=", 2), (">", 1), ("<", 1),
        ] {
            if let Some(pos) = Self::find_top_level_op(condition, op) {
                let left = condition[..pos].trim();
                let right = condition[pos + op.len()..].trim();
                return Some(Self::compare_values(
                    &Self::eval_expression(left, item, idx)?,
                    op,
                    &Self::eval_expression(right, item, idx)?,
                ));
            }
        }

        // Simple truthy check
        let val = Self::eval_expression(condition, item, idx)?;
        Some(is_truthy(&val))
    }

    /// 查找顶层运算符（忽略括号内的内容）
    fn find_top_level_op(expr: &str, op: &str) -> Option<usize> {
        let mut depth: i32 = 0;
        let mut in_string = false;
        let mut string_char = ' ';

        let bytes = expr.as_bytes();
        let op_bytes = op.as_bytes();

        let mut i = 0;
        while i <= bytes.len() - op_bytes.len() {
            let ch = bytes[i] as char;

            if !in_string {
                match ch {
                    '\'' | '"' => {
                        in_string = true;
                        string_char = ch;
                    }
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' => depth = depth.saturating_sub(1),
                    _ => {}
                }
            } else if ch == string_char {
                in_string = false;
            }

            if depth == 0 && !in_string {
                let mut match_op = true;
                for j in 0..op_bytes.len() {
                    if bytes[i + j] != op_bytes[j] {
                        match_op = false;
                        break;
                    }
                }
                if match_op {
                    return Some(i);
                }
            }
            i += 1;
        }
        None
    }

    /// 比较两个值
    fn compare_values(left: &Value, op: &str, right: &Value) -> bool {
        match op {
            "==" => {
                // Deep equality check
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        l.as_f64() == r.as_f64()
                    }
                    (Value::String(l), Value::String(r)) => l == r,
                    (Value::Bool(l), Value::Bool(r)) => l == r,
                    (Value::Null, Value::Null) => true,
                    _ => false,
                }
            }
            "!=" => !Self::compare_values(left, "==", right),
            ">" => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        l.as_f64() > r.as_f64()
                    }
                    _ => false,
                }
            }
            "<" => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        l.as_f64() < r.as_f64()
                    }
                    _ => false,
                }
            }
            ">=" => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        l.as_f64() >= r.as_f64()
                    }
                    _ => false,
                }
            }
            "<=" => {
                match (left, right) {
                    (Value::Number(l), Value::Number(r)) => {
                        l.as_f64() <= r.as_f64()
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// 评估表达式，支持 item, __index, 数值, 字符串
    fn eval_expression(expr: &str, item: &Value, idx: usize) -> Option<Value> {
        let expr = expr.trim();

        // Boolean literals
        if expr == "true" {
            return Some(Value::Bool(true));
        }
        if expr == "false" {
            return Some(Value::Bool(false));
        }

        // Null
        if expr == "null" || expr == "nil" {
            return Some(Value::Null);
        }

        // String literal
        if (expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\'')) {
            return Some(Value::String(expr[1..expr.len() - 1].to_string()));
        }

        // Number literal
        if let Ok(n) = expr.parse::<f64>() {
            return Some(Value::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))));
        }

        // item reference
        if expr == "item" {
            return Some(item.clone());
        }

        // __index reference
        if expr == "__index" {
            return Some(Value::Number(serde_json::Number::from(idx)));
        }

        // Nested field access: item.field or item.field.nested
        if expr.starts_with("item.") {
            let path = &expr[5..]; // skip "item."
            return Self::resolve_field_path(item, path);
        }

        None
    }

    /// 解析 item.field.nested 路径
    fn resolve_field_path(value: &Value, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value.clone();

        for part in parts {
            if part.is_empty() {
                continue;
            }
            // Handle array index like field[0]
            if let Some(open) = part.find('[') {
                let field = &part[..open];
                let close = part.find(']')?;
                let idx_str = &part[open + 1..close];
                let idx: i64 = idx_str.parse().ok()?;

                current = current.get(field)?.clone();
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
                current = current.get(part)?.clone();
            }
        }

        Some(current)
    }

    /// 支持 field[index] 和 field[-1] 以及 field.length
    fn resolve_nested(value: &Value, path: &[&str]) -> Option<Value> {
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

/// 判断 JSON 值是否为真
pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => !s.is_empty() && s != "false" && s != "0",
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}
