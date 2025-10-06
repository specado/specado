use crate::types::{LossinessCode, LossinessEntry, LossinessLevel, LossinessReport};
use serde_json::json;

pub fn clamp_value(
    value: f64,
    range: [f64; 2],
    path: impl Into<String>,
    report: &mut LossinessReport,
) -> f64 {
    let [min, max] = range;
    let path = path.into();

    if value < min {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Clamp,
            level: LossinessLevel::Warn,
            path: path.clone(),
            reason: format!("Value {} below minimum {}", value, min),
            suggested_fix: Some(format!("Use value >= {}", min)),
            details: Some(json!({
                "requested": value,
                "clamped_to": min,
            })),
        });
        min
    } else if value > max {
        report.add_entry(LossinessEntry {
            code: LossinessCode::Clamp,
            level: LossinessLevel::Warn,
            path: path.clone(),
            reason: format!("Value {} above maximum {}", value, max),
            suggested_fix: Some(format!("Use value <= {}", max)),
            details: Some(json!({
                "requested": value,
                "clamped_to": max,
            })),
        });
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StrictMode;

    fn fresh_report() -> LossinessReport {
        LossinessReport::new(StrictMode::Warn)
    }

    #[test]
    fn clamps_value_below_min() {
        let mut report = fresh_report();
        let result = clamp_value(-1.0, [0.0, 2.0], "sampling.temperature", &mut report);
        assert_eq!(result, 0.0);
        assert!(report.is_lossy);
        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].path, "sampling.temperature");
    }

    #[test]
    fn clamps_value_above_max() {
        let mut report = fresh_report();
        let result = clamp_value(3.5, [0.0, 2.0], "sampling.temperature", &mut report);
        assert_eq!(result, 2.0);
        assert!(report.is_lossy);
        assert_eq!(
            report.entries[0].details.as_ref().unwrap()["clamped_to"],
            2.0
        );
    }

    #[test]
    fn leaves_value_in_range() {
        let mut report = fresh_report();
        let result = clamp_value(1.2, [0.0, 2.0], "sampling.temperature", &mut report);
        assert_eq!(result, 1.2);
        assert!(!report.is_lossy);
        assert!(report.entries.is_empty());
    }
}
