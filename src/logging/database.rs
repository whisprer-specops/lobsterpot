// src/logging/database.rs (corrected version)
use sqlx::{sqlite::SqlitePool, Row};
use anyhow::Result;
use super::models::{InteractionLog, ThreatLog, SystemHealthLog};

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