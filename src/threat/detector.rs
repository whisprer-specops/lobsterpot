// src/threat/detector.rs (corrected version)
use anyhow::Result;
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::network::features::PacketFeatures;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatCategory {
    Malware,
    Phishing,
    DDoS,
    Intrusion,
    DataExfiltration,
    Unknown,
}

#[derive(Debug)]
pub struct ThreatInfo {
    pub severity: ThreatSeverity,
    pub category: ThreatCategory,
    pub confidence: f32,
    pub source: String,
    pub details: String,
}

pub struct ThreatDetector {
    pub db: Arc<Mutex<SqlitePool>>,
}

impl ThreatDetector {
    pub fn new(db: Arc<Mutex<SqlitePool>>) -> Self {
        Self { db }
    }

    pub async fn detect_threat(&self, features: &PacketFeatures) -> Result<bool> {
        // Simple detection logic for now - we'll improve this later
        // Check for suspicious port numbers
        if features.dest_port == 4444 || features.dest_port == 31337 {
            return Ok(true);
        }

        // Check for unusual payload size
        if features.packet_size > 10000 && features.is_tcp {
            return Ok(true);
        }

        // Check for unusual entropy in payload 
        if features.payload_entropy > 7.5 {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn update_models(&mut self, new_data: Vec<PacketFeatures>) -> Result<()> {
        // Placeholder for model updating logic
        Ok(())
    }
}