use serde::{Deserialize, Serialize};

/// Remote/serializable shape for the `health_metrics` table.
/// One row per metric per timestamp (e.g. cpu_usage_percent, memory_usage_percent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthMetric {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub timestamp: String,
    pub device_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub metric_name: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Default for HealthMetric {
    fn default() -> Self {
        Self {
            id: None,
            timestamp: String::new(),
            device_id: 0,
            source: None,
            metric_name: String::new(),
            value: 0.0,
            unit: None,
            created_at: None,
        }
    }
}

impl HealthMetric {
    /// Build a metric for insert (id and created_at omitted; DB sets them).
    pub fn new(
        device_id: i64,
        timestamp: String,
        metric_name: String,
        value: f64,
        source: Option<String>,
        unit: Option<String>,
    ) -> Self {
        Self {
            id: None,
            timestamp,
            device_id,
            source,
            metric_name,
            value,
            unit,
            created_at: None,
        }
    }
}
