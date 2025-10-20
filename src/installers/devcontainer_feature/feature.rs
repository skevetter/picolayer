use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DevContainer Feature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub id: String,
    pub version: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub options: Option<HashMap<String, FeatureOption>>,
    pub container_env: Option<HashMap<String, String>>,
    pub entrypoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOption {
    #[serde(rename = "type")]
    pub option_type: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

impl Feature {
    /// Resolve feature options with defaults
    pub fn resolve_options(
        &self,
        provided_options: Option<HashMap<String, String>>,
    ) -> HashMap<String, String> {
        let mut resolved = provided_options.unwrap_or_default();

        if let Some(option_defs) = &self.options {
            for (name, option) in option_defs {
                if !resolved.contains_key(name)
                    && let Some(default) = &option.default
                {
                    let default_str = match default {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => String::new(),
                    };
                    resolved.insert(name.clone(), default_str);
                }
            }
        }

        resolved
    }
}
