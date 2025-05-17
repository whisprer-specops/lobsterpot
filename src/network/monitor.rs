// src/network/monitor.rs
use pcap::{Device, Capture};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, error};
use crate::threat::detector::ThreatDetector;
use super::features::{FeatureExtractor, PacketFeatures};
use super::firewall::Firewall;


pub struct NetworkMonitor {
    interface: String,
    threat_detector: Arc<Mutex<ThreatDetector>>,
    feature_extractor: FeatureExtractor,  // Now using the enhanced version
    packet_buffer: Vec<PacketFeatures>,   // Now storing the enhanced features
    batch_size: usize,
}

impl NetworkMonitor {
    pub fn new(interface: &str, threat_detector: ThreatDetector) -> Self {
        Self {
            interface: interface.to_string(),
            threat_detector: Arc::new(Mutex::new(threat_detector)),
            feature_extractor: FeatureExtractor::new(),
            packet_buffer: Vec::new(),
            batch_size: 1000,  // Or whatever size makes sense
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting network monitoring on interface: {}", self.interface);
        
        let device = Device::from(self.interface.as_str())?;
        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .buffer_size(16777216)?  // 16MB buffer
            .open()?;

        while let Ok(packet) = cap.next_packet() {
            let features = self.feature_extractor.extract_features(&packet);
            
            let is_threat = {
                let detector = self.threat_detector.lock().await;
                detector.detect_threat(&features).await?
            };

            if is_threat {
                info!("Threat detected! Processing packet...");
                self.handle_threat(&packet, &features).await?;
            }

            self.packet_buffer.push(features);

            if self.packet_buffer.len() >= self.batch_size {
                self.update_models().await?;
            }
        }

        Ok(())
    }

    async fn handle_threat(&self, packet: &pcap::Packet, features: &PacketFeatures) -> anyhow::Result<()> {
        // Log threat
        let threat_info = format!(
            "Threat detected - Proto: {}, Ports: {}→{}, Length: {}", 
            features.protocol, features.source_port, features.dest_port, features.length
        );
        error!("{}", threat_info);

        // Block the IP if needed
        if let Err(e) = self.block_threatening_ip(packet).await {
            error!("Failed to block IP: {}", e);
        }

        // Store threat info
        self.store_threat_info(features).await?;

        Ok(())
    }

    async fn update_models(&mut self) -> anyhow::Result<()> {
        let batch_data = std::mem::take(&mut self.packet_buffer);
        
        // Update threat detector with new data
        let mut detector = self.threat_detector.lock().await;
        detector.update_models(batch_data).await?;

        Ok(())
    }

    async fn block_threatening_ip(&self, packet: &pcap::Packet) -> anyhow::Result<()> {
        // Extract IP from packet and block it using the firewall module
        use pnet::packet::ethernet::EthernetPacket;
        use pnet::packet::ipv4::Ipv4Packet;
        
        if let Some(eth) = EthernetPacket::new(packet.data) {
            if let Some(ip) = Ipv4Packet::new(eth.payload()) {
                let ip_str = ip.get_source().to_string();
                firewall::Firewall::block_ip(&ip_str)?;
                info!("Blocked threatening IP: {}", ip_str);
            }
        }
        
        Ok(())
    }

    async fn store_threat_info(&self, features: &PacketFeatures) -> anyhow::Result<()> {
        let detector = self.threat_detector.lock().await;
        let db = detector.db.lock().await;

        sqlx::query!(
            r#"
            INSERT INTO interactions (timestamp, interaction_details, outcome)
            VALUES (datetime('now'), ?, 'blocked')
            "#,
            serde_json::to_string(features)?
        )
        .execute(&*db)
        .await?;

        Ok(())
    }
}
