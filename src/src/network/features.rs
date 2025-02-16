// src/network/features.rs
use pnet::packet::{ethernet::EthernetPacket, ipv4::Ipv4Packet, tcp::TcpPacket, udp::UdpPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFeatures {
    // Size features
    pub packet_size: u32,
    pub payload_size: u32,
    pub header_size: u32,

    // Protocol features
    pub protocol: u8,
    pub is_tcp: bool,
    pub is_udp: bool,
    pub is_icmp: bool,

    // Port features
    pub source_port: u16,
    pub dest_port: u16,
    pub is_well_known_port: bool,

    // TCP specific features
    pub tcp_flags: Option<u8>,
    pub window_size: Option<u16>,
    pub urgent_pointer: Option<u16>,

    // Traffic pattern features
    pub packets_per_second: f64,
    pub bytes_per_second: f64,
    pub avg_packet_size: f64,

    // Behavioral features
    pub is_response: bool,
    pub connection_attempts: u32,
    pub payload_entropy: f64,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct ConnectionKey {
    source_ip: [u8; 4],
    dest_ip: [u8; 4],
    source_port: u16,
    dest_port: u16,
    protocol: u8,
}

#[derive(Debug)]
struct ConnectionStats {
    last_seen: Instant,
    packet_count: u32,
    byte_count: u64,
    attempts: u32,
}

pub struct FeatureExtractor {
    window_size: Duration,
    packet_history: VecDeque<(Instant, u32)>,
    connection_history: DashMap<ConnectionKey, ConnectionStats>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            window_size: Duration::from_secs(1),
            packet_history: VecDeque::new(),
            connection_history: DashMap::new(),
        }
    }

    pub fn extract_features(&mut self, packet: &pcap::Packet) -> PacketFeatures {
        let now = Instant::now();
        let eth_packet = EthernetPacket::new(packet.data).expect("Failed to parse ethernet packet");
        
        let mut features = PacketFeatures {
            packet_size: packet.len() as u32,
            payload_size: 0,
            header_size: 0,
            protocol: 0,
            is_tcp: false,
            is_udp: false,
            is_icmp: false,
            source_port: 0,
            dest_port: 0,
            is_well_known_port: false,
            tcp_flags: None,
            window_size: None,
            urgent_pointer: None,
            packets_per_second: 0.0,
            bytes_per_second: 0.0,
            avg_packet_size: 0.0,
            is_response: false,
            connection_attempts: 0,
            payload_entropy: 0.0,
        };

        // Extract IP-level features
        if let Some(ip_packet) = Ipv4Packet::new(eth_packet.payload()) {
            features.protocol = ip_packet.get_next_level_protocol().0;
            features.header_size = ip_packet.get_header_length() as u32 * 4;
            
            let connection_key = ConnectionKey {
                source_ip: ip_packet.get_source().octets(),
                dest_ip: ip_packet.get_destination().octets(),
                source_port: 0, // Will be set below
                dest_port: 0,   // Will be set below
                protocol: features.protocol,
            };

            match ip_packet.get_next_level_protocol() {
                IpNextHeaderProtocols::Tcp => {
                    features.is_tcp = true;
                    if let Some(tcp_packet) = TcpPacket::new(ip_packet.payload()) {
                        self.extract_tcp_features(&mut features, &tcp_packet);
                        self.update_connection_stats(&connection_key, packet.len() as u32, Some(&tcp_packet));
                    }
                },
                IpNextHeaderProtocols::Udp => {
                    features.is_udp = true;
                    if let Some(udp_packet) = UdpPacket::new(ip_packet.payload()) {
                        self.extract_udp_features(&mut features, &udp_packet);
                        self.update_connection_stats(&connection_key, packet.len() as u32, None);
                    }
                },
                IpNextHeaderProtocols::Icmp => {
                    features.is_icmp = true;
                },
                _ => {}
            }

            features.payload_size = packet.len() as u32 - features.header_size;
            features.payload_entropy = self.calculate_entropy(ip_packet.payload());
        }

        // Update traffic patterns
        self.update_traffic_patterns(&mut features, packet.len() as u32, now);

        features
    }

    fn extract_tcp_features(&self, features: &mut PacketFeatures, tcp: &TcpPacket) {
        features.source_port = tcp.get_source();
        features.dest_port = tcp.get_destination();
        features.tcp_flags = Some(tcp.get_flags());
        features.window_size = Some(tcp.get_window());
        features.urgent_pointer = Some(tcp.get_urgent_ptr());
        features.is_well_known_port = tcp.get_destination() <= 1024;
        features.is_response = tcp.get_flags() & 0x12 != 0; // SYN+ACK
    }

    fn extract_udp_features(&self, features: &mut PacketFeatures, udp: &UdpPacket) {
        features.source_port = udp.get_source();
        features.dest_port = udp.get_destination();
        features.is_well_known_port = udp.get_destination() <= 1024;
    }

    fn update_traffic_patterns(&mut self, features: &mut PacketFeatures, packet_size: u32, now: Instant) {
        // Remove old packets from history
        while let Some(front) = self.packet_history.front() {
            if now - front.0 > self.window_size {
                self.packet_history.pop_front();
            } else {
                break;
            }
        }

        // Add new packet
        self.packet_history.push_back((now, packet_size));

        // Calculate rates
        let window_duration = self.window_size.as_secs_f64();
        let packet_count = self.packet_history.len() as f64;
        let total_bytes: u32 = self.packet_history.iter().map(|(_, size)| size).sum();

        features.packets_per_second = packet_count / window_duration;
        features.bytes_per_second = total_bytes as f64 / window_duration;
        features.avg_packet_size = if packet_count > 0.0 {
            total_bytes as f64 / packet_count
        } else {
            0.0
        };
    }

    fn update_connection_stats(&self, key: &ConnectionKey, size: u32, tcp: Option<&TcpPacket>) {
        self.connection_history.entry(key.clone())
            .and_modify(|stats| {
                stats.last_seen = Instant::now();
                stats.packet_count += 1;
                stats.byte_count += size as u64;
                if let Some(tcp) = tcp {
                    if tcp.get_flags() & 0x02 != 0 { // SYN flag
                        stats.attempts += 1;
                    }
                }
            })
            .or_insert_with(|| ConnectionStats {
                last_seen: Instant::now(),
                packet_count: 1,
                byte_count: size as u64,
                attempts: if tcp.map_or(false, |t| t.get_flags() & 0x02 != 0) { 1 } else { 0 },
            });
    }

    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut frequencies = [0u32; 256];
        for &byte in data {
            frequencies[byte as usize] += 1;
        }

        let total = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &frequencies {
            if count > 0 {
                let probability = count as f64 / total;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }
}