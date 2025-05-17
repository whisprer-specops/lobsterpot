// src/healing/component.rs
use async_trait::async_trait;
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use tokio::fs;
use super::errors::HealingError;

#[derive(Debug)]
pub struct Component {
    name: String,
    path: PathBuf,
    backup_path: PathBuf,
    expected_hash: String,
    pid: Option<u32>,
}

#[async_trait]
pub trait Healable {
    async fn check_health(&self) -> Result<bool, HealingError>;
    async fn isolate(&mut self) -> Result<(), HealingError>;
    async fn repair(&mut self) -> Result<(), HealingError>;
    async fn replace(&mut self) -> Result<(), HealingError>;
}

impl Component {
    pub fn new<P: AsRef<Path>>(
        name: &str,
        path: P,
        backup_path: P,
        expected_hash: &str
    ) -> Self {
        Component {
            name: name.to_string(),
            path: path.as_ref().to_path_buf(),
            backup_path: backup_path.as_ref().to_path_buf(),
            expected_hash: expected_hash.to_string(),
            pid: None,
        }
    }

    async fn calculate_hash<P: AsRef<Path>>(path: P) -> Result<String, HealingError> {
        let content = fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn verify_integrity(&self) -> Result<bool, HealingError> {
        let current_hash = Self::calculate_hash(&self.path).await?;
        Ok(current_hash == self.expected_hash)
    }
}

#[async_trait]
impl Healable for Component {
    async fn check_health(&self) -> Result<bool, HealingError> {
        // Check process status using sysinfo
        let system = sysinfo::System::new_all();
        let process_alive = self.pid
            .map(|pid| system.process(sysinfo::Pid::from(pid as usize)).is_some())
            .unwrap_or(false);

        // Check file integrity
        let integrity_ok = self.verify_integrity().await?;

        Ok(process_alive && integrity_ok)
    }

    async fn isolate(&mut self) -> Result<(), HealingError> {
        if let Some(pid) = self.pid {
            use std::process::Command;
            
            // Try to gracefully stop the process
            Command::new("kill")
                .arg("-15")
                .arg(&pid.to_string())
                .output()
                .map_err(|e| HealingError::SystemError(e.to_string()))?;

            // Wait a bit and force kill if still running
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            let system = sysinfo::System::new_all();
            if system.process(sysinfo::Pid::from(pid as usize)).is_some() {
                Command::new("kill")
                    .arg("-9")
                    .arg(&pid.to_string())
                    .output()
                    .map_err(|e| HealingError::SystemError(e.to_string()))?;
            }

            self.pid = None;
        }
        Ok(())
    }

    async fn repair(&mut self) -> Result<(), HealingError> {
        if !self.backup_path.exists() {
            return Err(HealingError::BackupNotFound(
                self.backup_path.to_string_lossy().to_string()
            ));
        }

        // Copy from backup
        fs::copy(&self.backup_path, &self.path).await?;

        // Verify the repair
        if !self.verify_integrity().await? {
            return Err(HealingError::RestartFailure(
                "Repair verification failed".to_string()
            ));
        }

        Ok(())
    }

    async fn replace(&mut self) -> Result<(), HealingError> {
        // For this example, we'll just use the backup
        self.repair().await
    }
}