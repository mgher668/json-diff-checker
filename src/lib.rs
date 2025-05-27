use serde::{Deserialize, Serialize};
pub use serde_json::Value;
pub mod json_diff {
    use super::*;

    pub fn get_all_items(value: &Value, current_path: String) -> Vec<(String, Value)> {
        let mut items = Vec::new();

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    // If the key contains special characters, wrap it in square brackets and
                    // quotes
                    let new_path = if current_path.is_empty() {
                        if needs_escaping(key) {
                            format!("[\"{}\"]", key)
                        } else {
                            key.clone()
                        }
                    } else {
                        if needs_escaping(key) {
                            format!("{}[\"{}\"]", current_path, key)
                        } else {
                            format!("{}.{}", current_path, key)
                        }
                    };

                    items.push((new_path.clone(), val.clone()));

                    if val.is_object() || val.is_array() {
                        items.extend(get_all_items(val, new_path));
                    }
                }
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", current_path, i);

                    items.push((new_path.clone(), val.clone()));

                    if val.is_object() || val.is_array() {
                        items.extend(get_all_items(val, new_path));
                    }
                }
            }
            _ => {
                if !current_path.is_empty() {
                    items.push((current_path, value.clone()));
                }
            }
        }

        items
    }

    pub fn get_value_by_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
        let parts = parse_path(path);
        let mut current = value;

        for part in parts {
            match part {
                PathPart::Key(key) => {
                    current = current.get(key)?;
                }
                PathPart::Index(index) => {
                    current = current.get(index)?;
                }
            }
        }

        Some(current)
    }

    pub fn parse_path(path: &str) -> Vec<PathPart> {
        let mut parts = Vec::new();
        let mut chars = path.chars().peekable();
        let mut current = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '[' => {
                    // If there is accumulated string, add it as a key
                    if !current.is_empty() {
                        parts.push(PathPart::Key(current.clone()));
                        current.clear();
                    }

                    // Check if it is in the format ["key"]
                    if chars.peek() == Some(&'"') {
                        chars.next(); // Skip the starting quote
                        let mut key = String::new();
                        let mut escaped = false;

                        while let Some(ch) = chars.next() {
                            if escaped {
                                key.push(ch);
                                escaped = false;
                            } else if ch == '\\' {
                                escaped = true;
                            } else if ch == '"' {
                                // Skip the ending ]
                                if chars.peek() == Some(&']') {
                                    chars.next();
                                }
                                break;
                            } else {
                                key.push(ch);
                            }
                        }

                        parts.push(PathPart::Key(key));
                    } else {
                        // Digit index
                        let mut index_str = String::new();
                        while let Some(&ch) = chars.peek() {
                            if ch == ']' {
                                chars.next();
                                break;
                            }
                            index_str.push(ch);
                            chars.next();
                        }

                        if let Ok(index) = index_str.parse::<usize>() {
                            parts.push(PathPart::Index(index));
                        }
                    }
                }
                '.' => {
                    if !current.is_empty() {
                        parts.push(PathPart::Key(current.clone()));
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            parts.push(PathPart::Key(current));
        }

        parts
    }

    pub fn needs_escaping(key: &str) -> bool {
        key.contains('.') || key.contains('[') || key.contains(']') || key.contains('"')
    }

    pub fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => {
                if let (Some(f1), Some(f2)) = (n1.as_f64(), n2.as_f64()) {
                    (f1 - f2).abs() < f64::EPSILON
                } else {
                    n1 == n2
                }
            }
            _ => a == b,
        }
    }

    pub fn is_parent_missing(missing_paths: &[String], path: &str) -> bool {
        missing_paths
            .iter()
            .any(|missing| path.starts_with(missing) && path.len() > missing.len())
    }

    pub fn same_type(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(_), Value::Bool(_)) => true,
            (Value::Number(_), Value::Number(_)) => true,
            (Value::String(_), Value::String(_)) => true,
            (Value::Array(_), Value::Array(_)) => true,
            (Value::Object(_), Value::Object(_)) => true,
            _ => false,
        }
    }

    pub fn get_value_type(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum PathPart {
        Key(String),
        Index(usize),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TypeMismatch {
        pub path: String,
        pub base_type: String,
        pub compare_type: String,
        pub base_value: Value,
        pub compare_value: Value,
    }
}
