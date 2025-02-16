// src/database/threat_db.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use super::models::{ThreatEntry, ThreatIndicator, ThreatStatus, IndicatorType};
use super::migrations::run_migrations;
use crate::threat::detector::ThreatInfo;
use std::net::IpAddr;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;


pub struct ThreatDatabase {
    pool: SqlitePool,
}

impl ThreatDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Run migrations
        run_migrations(&pool).await?;

        Ok(ThreatDatabase { pool })
    }

    pub async fn record_threat(&self, threat: ThreatInfo, source_ip: IpAddr) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let entry = ThreatEntry {
            id,
            timestamp: now,
            source_ip: source_ip.to_string(),
            dest_ip: None,
            severity: threat.severity,
            category: threat.category,
            confidence: threat.confidence,
            details: threat.details,
            status: ThreatStatus::Active,
            metadata: serde_json::json!({
                "source": threat.source,
                "detection_time": now.to_rfc3339(),
            }),
        };

        sqlx::query(
            r#"
            INSERT INTO threats 
            (id, timestamp, source_ip, dest_ip, severity, category, confidence, details, status, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(entry.timestamp.to_rfc3339())
        .bind(&entry.source_ip)
        .bind(&entry.dest_ip)
        .bind(format!("{:?}", entry.severity))
        .bind(format!("{:?}", entry.category))
        .bind(entry.confidence)
        .bind(&entry.details)
        .bind(format!("{:?}", entry.status))
        .bind(entry.metadata.to_string())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn add_indicator(&self, indicator: ThreatIndicator) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO threat_indicators 
            (id, threat_id, indicator_type, value, first_seen, last_seen, confidence)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(indicator.id.to_string())
        .bind(indicator.threat_id.to_string())
        .bind(format!("{:?}", indicator.indicator_type))
        .bind(&indicator.value)
        .bind(indicator.first_seen.to_rfc3339())
        .bind(indicator.last_seen.to_rfc3339())
        .bind(indicator.confidence)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_threat_status(&self, id: Uuid, status: ThreatStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE threats 
            SET status = ? 
            WHERE id = ?
            "#,
        )
        .bind(format!("{:?}", status))
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_threats(&self) -> Result<Vec<ThreatEntry>> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM threats 
            WHERE status = 'Active'
            ORDER BY timestamp DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let threats = rows.into_iter()
            .map(|row| ThreatEntry {
                id: Uuid::parse_str(&row.id).unwrap(),
                timestamp: DateTime::parse_from_rfc3339(&row.timestamp)
                    .unwrap()
                    .with_timezone(&Utc),
                source_ip: row.source_ip,
                dest_ip: row.dest_ip,
                severity: serde_json::from_str(&row.severity).unwrap(),
                category: serde_json::from_str(&row.category).unwrap(),
                confidence: row.confidence,
                details: row.details,
                status: serde_json::from_str(&row.status).unwrap(),
                metadata: serde_json::from_str(&row.metadata).unwrap(),
            })
            .collect();

        Ok(threats)
    }

    pub async fn get_threat_indicators(&self, threat_id: Uuid) -> Result<Vec<ThreatIndicator>> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM threat_indicators 
            WHERE threat_id = ?
            "#,
            threat_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let indicators = rows.into_iter()
            .map(|row| ThreatIndicator {
                id: Uuid::parse_str(&row.id).unwrap(),
                threat_id: Uuid::parse_str(&row.threat_id).unwrap(),
                indicator_type: serde_json::from_str(&row.indicator_type).unwrap(),
                value: row.value,
                first_seen: DateTime::parse_from_rfc3339(&row.first_seen)
                    .unwrap()
                    .with_timezone(&Utc),
                last_seen: DateTime::parse_from_rfc3339(&row.last_seen)
                    .unwrap()
                    .with_timezone(&Utc),
                confidence: row.confidence,
            })
            .collect();

        Ok(indicators)
    }

    pub async fn check_ip_history(&self, ip: IpAddr) -> Result<Vec<ThreatEntry>> {
        let rows = sqlx::query!(
            r#"
            SELECT * FROM threats 
            WHERE source_ip = ? OR dest_ip = ?
            ORDER BY timestamp DESC
            "#,
            ip.to_string(),
            ip.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let threats = rows.into_iter()
            .map(|row| ThreatEntry {
                id: Uuid::parse_str(&row.id).unwrap(),
                timestamp: DateTime::parse_from_rfc3339(&row.timestamp)
                    .unwrap()
                    .with_timezone(&Utc),
                source_ip: row.source_ip,
                dest_ip: row.dest_ip,
                severity: serde_json::from_str(&row.severity).unwrap(),
                category: serde_json::from_str(&row.category).unwrap(),
                confidence: row.confidence,
                details: row.details,
                status: serde_json::from_str(&row.status).unwrap(),
                metadata: serde_json::from_str(&row.metadata).unwrap(),
            })
            .collect();

        Ok(threats)
    }
}

// Example usage
pub async fn example_usage() -> Result<()> {
    let db = ThreatDatabase::new("sqlite:threat.db").await?;

    // Record a new threat
    let threat = ThreatInfo {
        severity: ThreatSeverity::High,
        category: ThreatCategory::Malware,
        confidence: 0.95,
        source: "Threat Feed".to_string(),
        details: "Malicious activity detected".to_string(),
    };

    let source_ip: IpAddr = "192.168.1.100".parse()?;
    let threat_id = db.record_threat(threat, source_ip).await?;

    // Add an indicator
    let indicator = ThreatIndicator {
        id: Uuid::new_v4(),
        threat_id,
        indicator_type: IndicatorType::IP,
        value: source_ip.to_string(),
        first_seen: Utc::now(),
        last_seen: Utc::now(),
        confidence: 0.95,
    };

    db.add_indicator(indicator).await?;

    // Query active threats
    let active_threats = db.get_active_threats().await?;
    println!("Active threats: {:?}", active_threats);

    Ok(())
}
