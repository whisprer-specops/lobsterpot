// src/logging/analyzer.rs
use polars::prelude::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use super::models::{InteractionLog, ThreatLog, SystemHealthLog};

pub struct LogAnalyzer {
    db: Database,
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
