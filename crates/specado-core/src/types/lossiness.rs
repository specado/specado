use super::prompt::StrictMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessReport {
    pub is_lossy: bool,
    pub strict_mode: StrictMode,
    pub entries: Vec<LossinessEntry>,
    pub omissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LossinessEntry {
    pub code: LossinessCode,
    pub level: LossinessLevel,
    pub path: String,
    pub reason: String,
    pub suggested_fix: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum LossinessCode {
    Clamp,
    Drop,
    Emulate,
    Conflict,
    Relocate,
    Unsupported,
    MapFallback,
    PerformanceImpact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LossinessLevel {
    Info,
    Warn,
    Error,
}

impl LossinessReport {
    pub fn new(strict_mode: StrictMode) -> Self {
        Self {
            is_lossy: false,
            strict_mode,
            entries: Vec::new(),
            omissions: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: LossinessEntry) {
        self.is_lossy = true;
        self.entries.push(entry);
    }

    pub fn add_omission(&mut self, path: impl Into<String>) {
        self.omissions.push(path.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn add_entry_marks_report_lossy() {
        let mut report = LossinessReport::new(StrictMode::Warn);
        report.add_entry(LossinessEntry {
            code: LossinessCode::Clamp,
            level: LossinessLevel::Warn,
            path: "body.temperature".into(),
            reason: "Value out of bounds".into(),
            suggested_fix: Some("Clamp to provider range".into()),
            details: Some(json!({"min": 0.0, "max": 1.0})),
        });

        assert!(report.is_lossy);
        assert_eq!(report.entries.len(), 1);
    }

    #[test]
    fn add_omission_tracks_paths() {
        let mut report = LossinessReport::new(StrictMode::Strict);
        report.add_omission("response.annotations");
        report.add_omission("response.metadata");

        assert_eq!(report.omissions.len(), 2);
        assert_eq!(report.omissions[0], "response.annotations");
    }
}
