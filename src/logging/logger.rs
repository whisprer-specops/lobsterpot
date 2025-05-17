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
}