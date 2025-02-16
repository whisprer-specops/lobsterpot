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