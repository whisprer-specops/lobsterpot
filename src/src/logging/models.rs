/// src/logging/models.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionLog {
    pub timestamp: DateTime<Utc>,
    pub interaction_type: String,
    pub details: String,
    pub outcome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatLog {
    pub timestamp: DateTime<Utc>,
    pub threat_type: String,
    pub source: String,
    pub action_taken: String,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealthLog {
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub status: String,
    pub metrics: serde_json::Value,
}
