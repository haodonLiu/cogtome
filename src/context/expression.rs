use serde_json::Value;

/// 评估布尔条件表达式，支持 ==, !=, >, <, >=, <=, &&, ||
pub fn eval_condition(condition: &str, item: &Value, idx: usize) -> Option<bool> {
    let condition = condition.trim();

    // Handle && and ||
    if let Some(pos) = find_top_level_op(condition, "&&") {
        let left = &condition[..pos];
        let right = &condition[pos + 2..];
        return Some(
            eval_condition(left, item, idx)? && eval_condition(right, item, idx)?
        );
    }
    if let Some(pos) = find_top_level_op(condition, "||") {
        let left = &condition[..pos];
        let right = &condition[pos + 2..];
        return Some(
            eval_condition(left, item, idx)? || eval_condition(right, item, idx)?
        );
    }

    // Handle comparisons
    for &(op, _) in &[
        ("==", 2), ("!=", 2), (">=", 2), ("<=", 2), (">", 1), ("<", 1),
    ] {
        if let Some(pos) = find_top_level_op(condition, op) {
            let left = condition[..pos].trim();
            let right = condition[pos + op.len()..].trim();
            return Some(compare_values(
                &eval_expression(left, item, idx)?,
                op,
                &eval_expression(right, item, idx)?,
            ));
        }
    }

    // Simple truthy check
    let val = eval_expression(condition, item, idx)?;
    Some(is_truthy(&val))
}

/// 查找顶层运算符（忽略括号内的内容）
pub fn find_top_level_op(expr: &str, op: &str) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut string_char = ' ';

    let bytes = expr.as_bytes();
    let op_bytes = op.as_bytes();

    // Early exit if op is longer than expression
    if op_bytes.len() > bytes.len() {
        return None;
    }

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
pub fn compare_values(left: &Value, op: &str, right: &Value) -> bool {
    match op {
        "==" => {
            // Deep equality check using JSON string serialization for complex types
            match (left, right) {
                (Value::Number(l), Value::Number(r)) => {
                    l.as_f64() == r.as_f64()
                }
                (Value::String(l), Value::String(r)) => l == r,
                (Value::Bool(l), Value::Bool(r)) => l == r,
                (Value::Null, Value::Null) => true,
                (Value::Array(l), Value::Array(r)) => {
                    // Compare arrays element by element
                    l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| compare_values(a, "==", b))
                }
                (Value::Object(l), Value::Object(r)) => {
                    // Compare objects by serializing to string
                    serde_json::to_string(l).ok() == serde_json::to_string(r).ok()
                }
                _ => false,
            }
        }
        "!=" => !compare_values(left, "==", right),
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
pub fn eval_expression(expr: &str, item: &Value, idx: usize) -> Option<Value> {
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
        return resolve_field_path(item, path);
    }

    None
}

/// 解析 item.field.nested 路径
pub fn resolve_field_path(value: &Value, path: &str) -> Option<Value> {
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
