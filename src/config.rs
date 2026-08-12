use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Warn,
    Error,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Thresholds {
    pub name_max: usize,
    pub desc_max: usize,
    pub body_max_lines: usize,
    pub body_max_tokens: usize,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            name_max: 64,
            desc_max: 1024,
            body_max_lines: 500,
            body_max_tokens: 5000,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub thresholds: Thresholds,
    pub rules: HashMap<String, Severity>,
}

impl Config {
    /// Absent file -> `Ok(default)`. Present but unparseable -> `Err(message)`.
    pub fn load(root: &Path) -> Result<Self, String> {
        let Ok(s) = std::fs::read_to_string(root.join("sklint.toml")) else {
            return Ok(Self::default());
        };
        toml::from_str(&s).map_err(|e| e.to_string())
    }

    pub fn severity(&self, rule: &str, default: Severity) -> Severity {
        self.rules.get(rule).copied().unwrap_or(default)
    }
}
