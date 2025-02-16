// src/threat/mod.rs
pub mod detector;
pub mod feed;
pub mod rules;
pub mod reputation;

// src/threat/detector.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;
use crate::network::packet::PacketInfo;
use super::feed::ThreatFeed;
use super::rules::RuleEngine;
use super::reputation::ReputationTracker;

pub struct ThreatDetector {
    threat_feed: Arc<ThreatFeed>,
    rule_engine: Arc<RuleEngine>,
    reputation_tracker: Arc<ReputationTracker>,
    blocked_ips: Arc<DashMap<std::net::IpAddr, BlockReason>>,
}

#[derive(Debug, Clone)]
pub struct ThreatInfo {
    pub severity: ThreatSeverity,
    pub category: ThreatCategory,
    pub confidence: f32,
    pub source: String,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatCategory {
    Malware,
    Phishing,
    BotNet,
    Scanner,
    Spam,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum BlockReason {
    ThreatFeed(String),
    RuleViolation(String),
    BadReputation(f32),
    ManualBlock,
}

impl ThreatDetector {
    pub async fn new() -> anyhow::Result<Self> {
        let threat_feed = Arc::new(ThreatFeed::new().await?);
        let rule_engine = Arc::new(RuleEngine::new());
        let reputation_tracker = Arc::new(ReputationTracker::new());
        let blocked_ips = Arc::new(DashMap::new());

        Ok(ThreatDetector {
            threat_feed,
            rule_engine,
            reputation_tracker,
            blocked_ips,
        })
    }

    pub async fn analyze_packet(&self, packet: &PacketInfo) -> Option<ThreatInfo> {
        // Check if IP is already blocked
        if self.blocked_ips.contains_key(&packet.source_ip) {
            return Some(ThreatInfo {
                severity: ThreatSeverity::High,
                category: ThreatCategory::Other("Previously Blocked".to_string()),
                confidence: 1.0,
                source: "Block List".to_string(),
                details: format!("IP {} is blocked", packet.source_ip),
            });
        }

        // Check threat feeds
        if let Some(threat) = self.threat_feed.check_ip(packet.source_ip).await {
            self.blocked_ips.insert(
                packet.source_ip,
                BlockReason::ThreatFeed(threat.details.clone()),
            );
            return Some(threat);
        }

        // Check rule violations
        if let Some(violation) = self.rule_engine.check_packet(packet) {
            self.blocked_ips.insert(
                packet.source_ip,
                BlockReason::RuleViolation(violation.details.clone()),
            );
            return Some(violation);
        }

        // Check reputation
        let reputation = self.reputation_tracker.get_reputation(packet.source_ip);
        if reputation < 0.3 {
            self.blocked_ips.insert(
                packet.source_ip,
                BlockReason::BadReputation(reputation),
            );
            return Some(ThreatInfo {
                severity: ThreatSeverity::Medium,
                category: ThreatCategory::Other("Bad Reputation".to_string()),
                confidence: 1.0 - reputation,
                source: "Reputation System".to_string(),
                details: format!("IP {} has poor reputation score: {}", packet.source_ip, reputation),
            });
        }

        None
    }
}

// src/threat/feed.rs
use std::time::Duration;
use cached::proc_macro::cached;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

pub struct ThreatFeed {
    client: Client,
    api_keys: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreatResponse {
    blacklisted: bool,
    category: Option<String>,
    confidence_score: Option<f32>,
    threat_type: Option<String>,
}

impl ThreatFeed {
    pub async fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let api_keys = Self::load_api_keys()?;

        Ok(ThreatFeed {
            client,
            api_keys,
        })
    }

    fn load_api_keys() -> anyhow::Result<HashMap<String, String>> {
        // In production, load from secure configuration
        let mut keys = HashMap::new();
        keys.insert(
            "alienvault".to_string(),
            std::env::var("ALIENVAULT_API_KEY").unwrap_or_default(),
        );
        Ok(keys)
    }

    #[cached(
        time = 3600, // Cache for 1 hour
        key = "String",
        convert = r#"{ format!("{}", ip) }"#,
        result = true
    )]
    pub async fn check_ip(&self, ip: IpAddr) -> Option<ThreatInfo> {
        // Check multiple threat intelligence feeds
        let feeds = vec![
            self.check_alienvault(ip),
            self.check_abuseipdb(ip),
        ];

        // Aggregate results
        let results = futures::future::join_all(feeds).await;
        
        // Return the highest confidence threat
        results.into_iter()
            .flatten()
            .max_by_key(|threat| (threat.confidence * 100.0) as u32)
    }

    async fn check_alienvault(&self, ip: IpAddr) -> Option<ThreatInfo> {
        if let Some(api_key) = self.api_keys.get("alienvault") {
            let url = format!("https://otx.alienvault.com/api/v1/indicators/IPv4/{}/reputation", ip);
            let response = self.client.get(&url)
                .header("X-OTX-API-KEY", api_key)
                .send()
                .await
                .ok()?;

            let threat: ThreatResponse = response.json().await.ok()?;
            
            if threat.blacklisted {
                return Some(ThreatInfo {
                    severity: ThreatSeverity::High,
                    category: Self::categorize_threat(threat.threat_type),
                    confidence: threat.confidence_score.unwrap_or(0.8),
                    source: "AlienVault".to_string(),
                    details: format!("IP {} found in AlienVault", ip),
                });
            }
        }
        None
    }

    fn categorize_threat(threat_type: Option<String>) -> ThreatCategory {
        match threat_type.as_deref() {
            Some("malware") => ThreatCategory::Malware,
            Some("phishing") => ThreatCategory::Phishing,
            Some("botnet") => ThreatCategory::BotNet,
            Some("scanner") => ThreatCategory::Scanner,
            Some("spam") => ThreatCategory::Spam,
            Some(other) => ThreatCategory::Other(other.to_string()),
            None => ThreatCategory::Other("Unknown".to_string()),
        }
    }
}

// src/threat/rules.rs
use regex::Regex;
use std::sync::OnceLock;

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule + Send + Sync>>,
}

#[async_trait::async_trait]
trait Rule: Send + Sync {
    fn check(&self, packet: &PacketInfo) -> Option<ThreatInfo>;
}

struct PortScanRule {
    threshold: u32,
    time_window: Duration,
}

struct SuspiciousUserAgentRule {
    patterns: Vec<Regex>,
}

struct RateLimitRule {
    max_requests: u32,
    time_window: Duration,
}

impl RuleEngine {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![
            Box::new(PortScanRule::new(100, Duration::from_secs(60))),
            Box::new(SuspiciousUserAgentRule::new()),
            Box::new(RateLimitRule::new(1000, Duration::from_secs(60))),
        ];

        RuleEngine { rules }
    }

    pub fn check_packet(&self, packet: &PacketInfo) -> Option<ThreatInfo> {
        self.rules.iter()
            .find_map(|rule| rule.check(packet))
    }
}

impl PortScanRule {
    fn new(threshold: u32, time_window: Duration) -> Self {
        PortScanRule {
            threshold,
            time_window,
        }
    }
}

impl Rule for PortScanRule {
    fn check(&self, packet: &PacketInfo) -> Option<ThreatInfo> {
        // Implement port scan detection logic
        None
    }
}

// src/threat/reputation.rs
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct ReputationTracker {
    scores: DashMap<IpAddr, ReputationScore>,
}

struct ReputationScore {
    score: f32,
    last_updated: Instant,
    incidents: Vec<Incident>,
}

struct Incident {
    timestamp: Instant,
    severity: f32,
    reason: String,
}

impl ReputationTracker {
    pub fn new() -> Self {
        ReputationTracker {
            scores: DashMap::new(),
        }
    }

    pub fn get_reputation(&self, ip: IpAddr) -> f32 {
        self.scores
            .get(&ip)
            .map(|score| score.score)
            .unwrap_or(1.0)
    }

    pub fn update_reputation(&self, ip: IpAddr, incident: Incident) {
        self.scores.entry(ip).and_modify(|score| {
            score.incidents.push(incident);
            score.score = Self::calculate_score(&score.incidents);
            score.last_updated = Instant::now();
        }).or_insert_with(|| ReputationScore {
            score: 1.0,
            last_updated: Instant::now(),
            incidents: vec![incident],
        });
    }

    fn calculate_score(incidents: &[Incident]) -> f32 {
        let now = Instant::now();
        let recent_incidents: Vec<_> = incidents.iter()
            .filter(|i| now.duration_since(i.timestamp) < Duration::from_secs(3600))
            .collect();

        if recent_incidents.is_empty() {
            return 1.0;
        }

        let base_score = 1.0;
        let penalty: f32 = recent_incidents.iter()
            .map(|i| i.severity)
            .sum();

        (base_score - penalty).max(0.0)
    }
}

// Example usage
pub async fn example_usage() -> anyhow::Result<()> {
    // Initialize threat detector
    let detector = ThreatDetector::new().await?;

    // Example packet
    let packet = PacketInfo {
        timestamp: 0.0,
        source_ip: "192.168.1.100".parse()?,
        dest_ip: "192.168.1.1".parse()?,
        protocol: 6, // TCP
        length: 64,
        source_port: Some(12345),
        dest_port: Some(80),
        flags: None,
        http_info: None,
    };

    // Check for threats
    if let Some(threat) = detector.analyze_packet(&packet).await {
        println!("Threat detected: {:?}", threat);
    } else {
        println!("No threats detected");
    }

    Ok(())
}