// src/database/models.rs
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::threat::detector::{ThreatSeverity, ThreatCategory};


#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub dest_ip: Option<String>,
    pub severity: ThreatSeverity,
    pub category: ThreatCategory,
    pub confidence: f32,
    pub details: String,
    pub status: ThreatStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ThreatStatus {
    Active,
    Mitigated,
    FalsePositive,
    Investigating,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub id: Uuid,
    pub threat_id: Uuid,
    pub indicator_type: IndicatorType,
    pub value: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IndicatorType {
    IP,
    Domain,
    URL,
    FileHash,
    UserAgent,
}
