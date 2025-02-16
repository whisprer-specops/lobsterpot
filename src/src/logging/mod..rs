// src/logging/mod.rs
pub mod logger;
pub mod analyzer;
pub mod models;
pub mod database;

// Example usage of the logging system
pub async fn example_usage() -> Result<()> {
    use std::path::PathBuf;
    use chrono::{Duration, Utc};
    use crate::logging::{
        logger::Logger,
        models::ThreatLog,
        analyzer::LogAnalyzer
    };

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
