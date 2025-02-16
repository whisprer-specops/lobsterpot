// src/healing/mod.rs
pub mod monitor;
pub mod component;
pub mod errors;

// src/healing/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HealingError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),
    
    #[error("Failed to restart component: {0}")]
    RestartFailure(String),
    
    #[error("Backup not found: {0}")]
    BackupNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("System error: {0}")]
    SystemError(String),
}

// src/healing/component.rs
use async_trait::async_trait;
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::healing::errors::HealingError;

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
        // This would typically involve downloading or fetching a fresh copy
        // For this example, we'll just use the backup
        self.repair().await
    }
}

// src/healing/monitor.rs
use tokio::time::{Duration, interval};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::healing::component::{Component, Healable};
use crate::healing::errors::HealingError;

pub struct HealthMonitor {
    components: Arc<Mutex<HashMap<String, Component>>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        HealthMonitor {
            components: Arc::new(Mutex::new(HashMap::new())),
            check_interval,
        }
    }

    pub async fn register_component(&self, component: Component) {
        let mut components = self.components.lock().await;
        components.insert(component.name.clone(), component);
    }

    pub async fn start_monitoring(&self) -> Result<(), HealingError> {
        let components = Arc::clone(&self.components);
        let mut interval = interval(self.check_interval);

        loop {
            interval.tick().await;
            let mut components = components.lock().await;

            for component in components.values_mut() {
                match component.check_health().await {
                    Ok(true) => {
                        println!("Component {} is healthy", component.name);
                    },
                    Ok(false) => {
                        println!("Component {} needs healing", component.name);
                        self.heal_component(component).await?;
                    },
                    Err(e) => {
                        println!("Error checking component {}: {}", component.name, e);
                    }
                }
            }
        }
    }

    async fn heal_component(&self, component: &mut Component) -> Result<(), HealingError> {
        println!("Starting healing process for {}", component.name);

        // First try to isolate the component
        component.isolate().await?;

        // Try to repair
        if let Err(e) = component.repair().await {
            println!("Repair failed: {}. Attempting replacement...", e);
            // If repair fails, try replacement
            component.replace().await?;
        }

        println!("Healing complete for {}", component.name);
        Ok(())
    }
}

// Example usage
pub async fn example_usage() -> Result<(), HealingError> {
    // Create a health monitor
    let monitor = HealthMonitor::new(Duration::from_secs(60));

    // Create a component
    let component = Component::new(
        "firewall",
        "/path/to/firewall",
        "/path/to/backup/firewall",
        "expected_hash_value"
    );

    // Register the component
    monitor.register_component(component).await;

    // Start monitoring
    monitor.start_monitoring().await?;

    Ok(())
}