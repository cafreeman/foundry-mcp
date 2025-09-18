use serde_yml::{Number, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum YamlEditorError {
    #[error("File IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yml::Error),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Type mismatch at path {path}: expected {expected}, found {found}")]
    TypeMismatch { path: String, expected: String, found: String },

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub struct YamlEditor {
    content: Value,
    file_path: PathBuf,
    modified: bool,
}

impl YamlEditor {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, YamlEditorError> {
        let path_buf = path.as_ref().to_path_buf();
        let raw = fs::read_to_string(&path_buf)?;
        let content: Value = serde_yml::from_str(&raw)?;
        Ok(Self {
            content,
            file_path: path_buf,
            modified: false,
        })
    }

    fn navigate_to_path(&mut self, path: &str) -> Result<&mut Value, YamlEditorError> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &mut self.content;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if let Ok(index) = part.parse::<usize>() {
                current = current
                    .as_sequence_mut()
                    .ok_or_else(|| YamlEditorError::TypeMismatch {
                        path: parts[..=i].join("."),
                        expected: "array".to_string(),
                        found: value_type_name(current).to_string(),
                    })?
                    .get_mut(index)
                    .ok_or_else(|| YamlEditorError::PathNotFound(parts[..=i].join(".")))?;
            } else {
                current = current
                    .as_mapping_mut()
                    .ok_or_else(|| YamlEditorError::TypeMismatch {
                        path: parts[..=i].join("."),
                        expected: "object".to_string(),
                        found: value_type_name(current).to_string(),
                    })?
                    .get_mut(&Value::String(part.to_string()))
                    .ok_or_else(|| YamlEditorError::PathNotFound(parts[..=i].join(".")))?;
            }
        }

        Ok(current)
    }

    pub fn update_path(&mut self, path: &str, value: Value) -> Result<(), YamlEditorError> {
        let target = self.navigate_to_path(path)?;
        *target = value;
        self.modified = true;
        Ok(())
    }

    pub fn append_to_array(&mut self, path: &str, value: Value) -> Result<(), YamlEditorError> {
        let target = self.navigate_to_path(path)?;
        let seq = target.as_sequence_mut().ok_or_else(|| YamlEditorError::TypeMismatch {
            path: path.to_string(),
            expected: "array".to_string(),
            found: value_type_name(target).to_string(),
        })?;
        seq.push(value);
        self.modified = true;
        Ok(())
    }

    pub fn remove_path(&mut self, path: &str) -> Result<(), YamlEditorError> {
        // Remove the last segment from the path to get parent and the key/index to remove
        let mut parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(YamlEditorError::InvalidOperation(
                "Cannot remove root value".to_string(),
            ));
        }
        let last = parts.pop().unwrap();
        let parent_path = parts.join(".");
        let parent = if parent_path.is_empty() {
            &mut self.content
        } else {
            self.navigate_to_path(&parent_path)?
        };

        if let Ok(index) = last.parse::<usize>() {
            let seq = parent.as_sequence_mut().ok_or_else(|| YamlEditorError::TypeMismatch {
                path: parent_path.clone(),
                expected: "array".to_string(),
                found: value_type_name(parent).to_string(),
            })?;
            if index >= seq.len() {
                return Err(YamlEditorError::PathNotFound(path.to_string()));
            }
            seq.remove(index);
        } else {
            let map = parent.as_mapping_mut().ok_or_else(|| YamlEditorError::TypeMismatch {
                path: parent_path.clone(),
                expected: "object".to_string(),
                found: value_type_name(parent).to_string(),
            })?;
            let removed = map.remove(&Value::String(last.to_string()));
            if removed.is_none() {
                return Err(YamlEditorError::PathNotFound(path.to_string()));
            }
        }

        self.modified = true;
        Ok(())
    }

    pub fn get_value(&self, path: &str) -> Result<&Value, YamlEditorError> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.content;

        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if let Ok(index) = part.parse::<usize>() {
                current = current
                    .as_sequence()
                    .ok_or_else(|| YamlEditorError::TypeMismatch {
                        path: parts[..=i].join("."),
                        expected: "array".to_string(),
                        found: value_type_name(current).to_string(),
                    })?
                    .get(index)
                    .ok_or_else(|| YamlEditorError::PathNotFound(parts[..=i].join(".")))?;
            } else {
                current = current
                    .as_mapping()
                    .ok_or_else(|| YamlEditorError::TypeMismatch {
                        path: parts[..=i].join("."),
                        expected: "object".to_string(),
                        found: value_type_name(current).to_string(),
                    })?
                    .get(&Value::String(part.to_string()))
                    .ok_or_else(|| YamlEditorError::PathNotFound(parts[..=i].join(".")))?;
            }
        }
        Ok(current)
    }

    pub fn set_string(&mut self, path: &str, value: &str) -> Result<(), YamlEditorError> {
        self.update_path(path, Value::String(value.to_string()))
    }

    pub fn set_bool(&mut self, path: &str, value: bool) -> Result<(), YamlEditorError> {
        self.update_path(path, Value::Bool(value))
    }

    pub fn increment_number(&mut self, path: &str) -> Result<(), YamlEditorError> {
        let target = self.navigate_to_path(path)?;
        match target {
            Value::Number(n) => {
                // serde_yml uses serde_yaml-like Number; convert to i64 or f64 then increment
                if let Some(i) = n.as_i64() {
                    *target = Value::Number(Number::from(i + 1));
                } else if let Some(u) = n.as_u64() {
                    *target = Value::Number(Number::from(u + 1));
                } else if let Some(f) = n.as_f64() {
                    *target = Value::Number(Number::from(f + 1.0));
                } else {
                    return Err(YamlEditorError::InvalidOperation(
                        format!("Unsupported numeric type at {path}"),
                    ));
                }
            }
            other => {
                return Err(YamlEditorError::TypeMismatch {
                    path: path.to_string(),
                    expected: "number".to_string(),
                    found: value_type_name(other).to_string(),
                })
            }
        }
        self.modified = true;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), YamlEditorError> {
        let serialized = serde_yml::to_string(&self.content)?;
        fs::write(&self.file_path, serialized)?;
        self.modified = false;
        Ok(())
    }

    pub fn save_if_modified(&mut self) -> Result<bool, YamlEditorError> {
        if self.modified {
            self.save()?;
            return Ok(true);
        }
        Ok(false)
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "array",
        Value::Mapping(_) => "object",
        _ => "unknown",
    }
}

