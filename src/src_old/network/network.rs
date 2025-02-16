// src/network/mod.rs
pub mod capture;
pub mod analyzer;
pub mod packet;
pub mod protocols;

// src/network/packet.rs
use serde::{Serialize, Deserialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketInfo {
    pub timestamp: f64,
    pub source_ip: IpAddr,
    pub dest_ip: IpAddr,
    pub protocol: u8,
    pub length: usize,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub flags: Option<TcpFlags>,
    pub http_info: Option<HttpInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpFlags {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
    pub psh: bool,
    pub urg: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInfo {
    pub method: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub user_agent: Option<String>,
}

// src/network/protocols.rs
use pnet::packet::{tcp, udp, ip, ethernet};
use pnet::packet::Packet;
use std::net::IpAddr;

pub fn parse_tcp_flags(tcp_packet: &tcp::TcpPacket) -> TcpFlags {
    TcpFlags {
        syn: tcp_packet.get_flags() & tcp::TcpFlags::SYN != 0,
        ack: tcp_packet.get_flags() & tcp::TcpFlags::ACK != 0,
        fin: tcp_packet.get_flags() & tcp::TcpFlags::FIN != 0,
        rst: tcp_packet.get_flags() & tcp::TcpFlags::RST != 0,
        psh: tcp_packet.get_flags() & tcp::TcpFlags::PSH != 0,
        urg: tcp_packet.get_flags() & tcp::TcpFlags::URG != 0,
    }
}

pub fn parse_http_request(tcp_payload: &[u8]) -> Option<HttpInfo> {
    let payload_str = String::from_utf8_lossy(tcp_payload);
    if !payload_str.starts_with("GET") && !payload_str.starts_with("POST") {
        return None;
    }

    let lines: Vec<&str> = payload_str.lines().collect();
    if lines.is_empty() {
        return None;
    }

    // Parse first line for method and path
    let first_line: Vec<&str> = lines[0].split_whitespace().collect();
    if first_line.len() < 2 {
        return None;
    }

    let method = first_line[0].to_string();
    let path = first_line[1].to_string();

    // Parse headers
    let mut host = None;
    let mut user_agent = None;

    for line in lines.iter().skip(1) {
        if line.starts_with("Host: ") {
            host = Some(line[6..].to_string());
        } else if line.starts_with("User-Agent: ") {
            user_agent = Some(line[12..].to_string());
        }
    }

    Some(HttpInfo {
        method: Some(method),
        host,
        path: Some(path),
        user_agent,
    })
}

// src/network/capture.rs
use pcap::{Device, Capture, Active};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use tokio::sync::mpsc;
use std::sync::Arc;
use anyhow::Result;

pub struct PacketCapture {
    interface: String,
    cap: Capture<Active>,
    packet_tx: mpsc::Sender<PacketInfo>,
}

impl PacketCapture {
    pub fn new(interface: &str, packet_tx: mpsc::Sender<PacketInfo>) -> Result<Self> {
        let device = Device::list()?
            .into_iter()
            .find(|d| d.name == interface)
            .ok_or_else(|| anyhow::anyhow!("Interface not found"))?;

        let cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .immediate_mode(true)
            .open()?;

        Ok(PacketCapture {
            interface: interface.to_string(),
            cap,
            packet_tx,
        })
    }

    pub async fn start_capture(&mut self) -> Result<()> {
        while let Ok(packet) = self.cap.next_packet() {
            if let Some(packet_info) = self.process_packet(&packet) {
                if let Err(e) = self.packet_tx.send(packet_info).await {
                    tracing::error!("Failed to send packet: {}", e);
                }
            }
        }
        Ok(())
    }

    fn process_packet(&self, packet: &pcap::Packet) -> Option<PacketInfo> {
        if let Some(ethernet) = EthernetPacket::new(packet.data) {
            match ethernet.get_ethertype() {
                EtherTypes::Ipv4 => {
                    if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                        return self.process_ip_packet(packet.header.ts.tv_sec as f64, &ipv4);
                    }
                }
                _ => return None,
            }
        }
        None
    }

    fn process_ip_packet(&self, timestamp: f64, ip_packet: &Ipv4Packet) -> Option<PacketInfo> {
        let source_ip = IpAddr::V4(ip_packet.get_source());
        let dest_ip = IpAddr::V4(ip_packet.get_destination());
        let protocol = ip_packet.get_next_level_protocol().0;
        let length = ip_packet.payload().len();

        let (source_port, dest_port, flags, http_info) = match ip_packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                if let Some(tcp) = TcpPacket::new(ip_packet.payload()) {
                    let tcp_flags = parse_tcp_flags(&tcp);
                    let http = if tcp.get_destination() == 80 || tcp.get_source() == 80 {
                        parse_http_request(tcp.payload())
                    } else {
                        None
                    };
                    (Some(tcp.get_source()), Some(tcp.get_destination()), Some(tcp_flags), http)
                } else {
                    (None, None, None, None)
                }
            }
            IpNextHeaderProtocols::Udp => {
                if let Some(udp) = UdpPacket::new(ip_packet.payload()) {
                    (Some(udp.get_source()), Some(udp.get_destination()), None, None)
                } else {
                    (None, None, None, None)
                }
            }
            _ => (None, None, None, None),
        };

        Some(PacketInfo {
            timestamp,
            source_ip,
            dest_ip,
            protocol,
            length,
            source_port,
            dest_port,
            flags,
            http_info,
        })
    }
}

// src/network/analyzer.rs
use std::collections::HashMap;
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use std::net::IpAddr;

pub struct PacketAnalyzer {
    packet_rx: mpsc::Receiver<PacketInfo>,
    connections: HashMap<ConnectionKey, ConnectionInfo>,
    suspicious_ips: HashMap<IpAddr, SuspiciousActivity>,
}

#[derive(Hash, Eq, PartialEq)]
struct ConnectionKey {
    source_ip: IpAddr,
    dest_ip: IpAddr,
    source_port: u16,
    dest_port: u16,
    protocol: u8,
}

struct ConnectionInfo {
    start_time: Instant,
    packets_sent: u64,
    bytes_sent: u64,
    last_seen: Instant,
}

struct SuspiciousActivity {
    failed_connections: u32,
    scan_attempts: u32,
    last_seen: Instant,
}

impl PacketAnalyzer {
    pub fn new(packet_rx: mpsc::Receiver<PacketInfo>) -> Self {
        PacketAnalyzer {
            packet_rx,
            connections: HashMap::new(),
            suspicious_ips: HashMap::new(),
        }
    }

    pub async fn start_analysis(&mut self) {
        while let Some(packet) = self.packet_rx.recv().await {
            self.analyze_packet(packet);
        }
    }

    fn analyze_packet(&mut self, packet: PacketInfo) {
        // Update connection tracking
        if let (Some(sport), Some(dport)) = (packet.source_port, packet.dest_port) {
            let key = ConnectionKey {
                source_ip: packet.source_ip,
                dest_ip: packet.dest_ip,
                source_port: sport,
                dest_port: dport,
                protocol: packet.protocol,
            };

            let now = Instant::now();
            let conn = self.connections.entry(key).or_insert_with(|| ConnectionInfo {
                start_time: now,
                packets_sent: 0,
                bytes_sent: 0,
                last_seen: now,
            });

            conn.packets_sent += 1;
            conn.bytes_sent += packet.length as u64;
            conn.last_seen = now;

            // Check for suspicious TCP behavior
            if let Some(flags) = packet.flags {
                if flags.syn && !flags.ack {
                    let suspicious = self.suspicious_ips
                        .entry(packet.source_ip)
                        .or_insert_with(|| SuspiciousActivity {
                            failed_connections: 0,
                            scan_attempts: 0,
                            last_seen: now,
                        });

                    suspicious.scan_attempts += 1;
                    suspicious.last_seen = now;

                    if suspicious.scan_attempts > 100 {
                        tracing::warn!("Possible port scan detected from IP: {}", packet.source_ip);
                    }
                }
            }
        }

        // Clean up old connections
        self.cleanup_old_connections();
    }

    fn cleanup_old_connections(&mut self) {
        let now = Instant::now();
        self.connections.retain(|_, info| {
            now.duration_since(info.last_seen) < Duration::from_secs(300)
        });
        self.suspicious_ips.retain(|_, info| {
            now.duration_since(info.last_seen) < Duration::from_secs(3600)
        });
    }
}

// Example usage
pub async fn example_usage() -> Result<()> {
    // Create channel for packet information
    let (packet_tx, packet_rx) = mpsc::channel(1000);

    // Initialize packet capture
    let mut capture = PacketCapture::new("eth0", packet_tx)?;

    // Initialize packet analyzer
    let mut analyzer = PacketAnalyzer::new(packet_rx);

    // Start capture and analysis in separate tasks
    tokio::spawn(async move {
        if let Err(e) = capture.start_capture().await {
            tracing::error!("Capture error: {}", e);
        }
    });

    // Start analysis
    analyzer.start_analysis().await;

    Ok(())
}