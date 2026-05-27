use serde_json::Value;

pub fn print_success(data: &Value, format: &str) {
    match format {
        "table" => print_table(data),
        _ => print_json(data),
    }
}

pub fn print_error(message: &str) {
    let error_response = serde_json::json!({
        "success": false,
        "data": null,
        "error": message
    });
    println!("{}", serde_json::to_string_pretty(&error_response).unwrap());
    std::process::exit(1);
}

fn print_json(data: &Value) {
    let response = serde_json::json!({
        "success": true,
        "data": data,
        "error": null
    });
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}

fn print_table(data: &Value) {
    // 简单的表格输出，根据数据类型格式化
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            match value {
                Value::Array(arr) => {
                    println!("{}: {} items", key, arr.len());
                    if !arr.is_empty() {
                        // 如果数组元素是对象，打印表头
                        if let Some(first_obj) = arr[0].as_object() {
                            let headers: Vec<&str> = first_obj.keys().map(|s| s.as_str()).collect();
                            println!("  {}", headers.join(" | "));
                            println!("  {}", "-".repeat(headers.len() * 15));
                        }
                        for (i, item) in arr.iter().enumerate().take(10) {
                            if let Some(obj) = item.as_object() {
                                let values: Vec<String> = obj.values().map(|v| format_value(v)).collect();
                                println!("  [{}] {}", i, values.join(" | "));
                            } else {
                                println!("  [{}] {}", i, format_value(item));
                            }
                        }
                        if arr.len() > 10 {
                            println!("  ... and {} more items", arr.len() - 10);
                        }
                    }
                }
                Value::Object(_) => {
                    println!("{}:", key);
                    print_nested_object(value, 1);
                }
                _ => {
                    println!("{}: {}", key, format_value(value));
                }
            }
        }
    } else {
        println!("{}", format_value(data));
    }
}

fn print_nested_object(value: &Value, depth: usize) {
    let indent = "  ".repeat(depth);
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            match val {
                Value::Object(_) => {
                    println!("{}{}:", indent, key);
                    print_nested_object(val, depth + 1);
                }
                Value::Array(arr) => {
                    println!("{}{}: {} items", indent, key, arr.len());
                }
                _ => {
                    println!("{}{}: {}", indent, key, format_value(val));
                }
            }
        }
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{} fields}}", obj.len()),
    }
}
