// src/logging/mod.rs
pub mod logger;
pub mod analyzer;
pub mod models;
pub mod database;

// src/logging/models.rs
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

// src/logging/database.rs
use sqlx::{sqlite::SqlitePool, Row};
use anyhow::Result;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Initialize tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS interaction_logs (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                interaction_type TEXT NOT NULL,
                details TEXT NOT NULL,
                outcome TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS threat_logs (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                threat_type TEXT NOT NULL,
                source TEXT NOT NULL,
                action_taken TEXT NOT NULL,
                success INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS system_health_logs (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                component TEXT NOT NULL,
                status TEXT NOT NULL,
                metrics TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Database { pool })
    }

    pub async fn log_interaction(&self, log: &InteractionLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO interaction_logs (timestamp, interaction_type, details, outcome)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(log.timestamp.to_rfc3339())
        .bind(&log.interaction_type)
        .bind(&log.details)
        .bind(&log.outcome)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_threat(&self, log: &ThreatLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO threat_logs (timestamp, threat_type, source, action_taken, success)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(log.timestamp.to_rfc3339())
        .bind(&log.threat_type)
        .bind(&log.source)
        .bind(&log.action_taken)
        .bind(log.success as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_system_health(&self, log: &SystemHealthLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO system_health_logs (timestamp, component, status, metrics)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(log.timestamp.to_rfc3339())
        .bind(&log.component)
        .bind(&log.status)
        .bind(log.metrics.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// src/logging/logger.rs
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use super::models::{InteractionLog, ThreatLog, SystemHealthLog};
use super::database::Database;

pub struct Logger {
    db: Database,
    log_dir: PathBuf,
}

impl Logger {
    pub async fn new(database_url: &str, log_dir: PathBuf) -> Result<Self> {
        let db = Database::new(database_url).await?;
        Ok(Logger { db, log_dir })
    }

    pub async fn log_interaction(&self, log: InteractionLog) -> Result<()> {
        // Log to database
        self.db.log_interaction(&log).await?;

        // Log to file
        let log_path = self.log_dir.join("interactions.log");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        let log_line = serde_json::to_string(&log)?;
        file.write_all(format!("{}\n", log_line).as_bytes()).await?;

        Ok(())
    }

    pub async fn log_threat(&self, log: ThreatLog) -> Result<()> {
        // Log to database
        self.db.log_threat(&log).await?;

        // Log to file
        let log_path = self.log_dir.join("threats.log");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        let log_line = serde_json::to_string(&log)?;
        file.write_all(format!("{}\n", log_line).as_bytes()).await?;

        Ok(())
    }

    pub async fn log_system_health(&self, log: SystemHealthLog) -> Result<()> {
        // Log to database
        self.db.log_system_health(&log).await?;

        // Log to file
        let log_path = self.log_dir.join("system_health.log");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await?;

        let log_line = serde_json::to_string(&log)?;
        file.write_all(format!("{}\n", log_line).as_bytes()).await?;

        Ok(())
    }
}

// src/logging/analyzer.rs
use polars::prelude::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use super::models::{InteractionLog, ThreatLog, SystemHealthLog};

pub struct LogAnalyzer {
    db: Database,
}

impl LogAnalyzer {
    pub fn new(db: Database) -> Self {
        LogAnalyzer { db }
    }

    pub async fn analyze_threats(&self, time_window: Duration) -> Result<ThreatAnalysis> {
        let since = Utc::now() - time_window;
        
        let threats = sqlx::query_as!(
            ThreatLog,
            r#"
            SELECT * FROM threat_logs 
            WHERE timestamp > ?
            "#,
            since.to_rfc3339()
        )
        .fetch_all(&self.db.pool)
        .await?;

        let mut analysis = ThreatAnalysis {
            total_threats: threats.len(),
            threat_types: HashMap::new(),
            success_rate: 0.0,
            common_sources: HashMap::new(),
        };

        for threat in threats {
            *analysis.threat_types.entry(threat.threat_type).or_insert(0) += 1;
            *analysis.common_sources.entry(threat.source).or_insert(0) += 1;
            if threat.success {
                analysis.success_rate += 1.0;
            }
        }

        if !threats.is_empty() {
            analysis.success_rate /= threats.len() as f64;
        }

        Ok(analysis)
    }

    pub async fn analyze_system_health(&self, time_window: Duration) -> Result<HealthAnalysis> {
        let since = Utc::now() - time_window;
        
        let health_logs = sqlx::query_as!(
            SystemHealthLog,
            r#"
            SELECT * FROM system_health_logs 
            WHERE timestamp > ?
            "#,
            since.to_rfc3339()
        )
        .fetch_all(&self.db.pool)
        .await?;

        let mut analysis = HealthAnalysis {
            component_status: HashMap::new(),
            alert_count: 0,
            uptime_percentage: HashMap::new(),
        };

        // Process logs using Polars for efficient analysis
        let df = DataFrame::new(vec![
            Series::new("timestamp", health_logs.iter().map(|log| log.timestamp).collect::<Vec<_>>()),
            Series::new("component", health_logs.iter().map(|log| &log.component).collect::<Vec<_>>()),
            Series::new("status", health_logs.iter().map(|log| &log.status).collect::<Vec<_>>()),
        ])?;

        // Group by component and calculate statistics
        let grouped = df.lazy()
            .groupby([col("component")])
            .agg([
                col("status").count().alias("total_records"),
                col("status").eq(lit("healthy")).sum().alias("healthy_count"),
            ])
            .collect()?;

        for row in grouped.iter() {
            let component = row.get("component")?.get_str(0)?;
            let total = row.get("total_records")?.get_f64(0)?;
            let healthy = row.get("healthy_count")?.get_f64(0)?;
            
            analysis.uptime_percentage.insert(
                component.to_string(),
                (healthy / total) * 100.0,
            );
        }

        Ok(analysis)
    }
}

#[derive(Debug)]
pub struct ThreatAnalysis {
    pub total_threats: usize,
    pub threat_types: HashMap<String, usize>,
    pub success_rate: f64,
    pub common_sources: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct HealthAnalysis {
    pub component_status: HashMap<String, String>,
    pub alert_count: usize,
    pub uptime_percentage: HashMap<String, f64>,
}

// Example usage
pub async fn example_usage() -> Result<()> {
    // Initialize logger
    let logger = Logger::new("sqlite:logs.db", PathBuf::from("logs")).await?;

    // Log some events
    logger.log_threat(ThreatLog {
        timestamp: Utc::now(),
        threat_type: "malicious_ip".to_string(),
        source: "192.168.1.100".to_string(),
        action_taken: "blocked".to_string(),
        success: true,
    }).await?;

    // Initialize analyzer
    let analyzer = LogAnalyzer::new(logger.db);

    // Analyze last 24 hours
    let threat_analysis = analyzer.analyze_threats(Duration::hours(24)).await?;
    println!("Threat Analysis: {:?}", threat_analysis);

    Ok(())
}