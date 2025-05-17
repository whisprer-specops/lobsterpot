use pcap::{Device, Capture};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, error};
use crate::features::{FeatureExtractor, PacketFeatures};


    // Replace the old extract_features with:
    fn extract_features(&mut self, packet: &pcap::Packet) -> PacketFeatures {
        self.feature_extractor.extract_features(packet)
    }
}

    fn extract_features(&self, packet: &pcap::Packet) -> PacketFeatures {
        use pnet::packet::{ethernet, ipv4, tcp, udp};
        use pnet::packet::Packet as PnetPacket;

        let eth_packet = ethernet::EthernetPacket::new(packet.data).unwrap();
        
        // Default values
        let mut features = PacketFeatures {
            length: packet.len() as u32,
            protocol: 0,
            flags: 0,
            source_port: 0,
            dest_port: 0,
            is_threat: false,  // Will be set by the detector
        };

        // Extract IP-level features
        if let Some(ip_packet) = ipv4::Ipv4Packet::new(eth_packet.payload()) {
            features.protocol = ip_packet.get_next_level_protocol().0;

            // Extract TCP/UDP features
            match features.protocol {
                6 => { // TCP
                    if let Some(tcp_packet) = tcp::TcpPacket::new(ip_packet.payload()) {
                        features.source_port = tcp_packet.get_source();
                        features.dest_port = tcp_packet.get_destination();
                        features.flags = tcp_packet.get_flags() as u8;
                    }
                },
                17 => { // UDP
                    if let Some(udp_packet) = udp::UdpPacket::new(ip_packet.payload()) {
                        features.source_port = udp_packet.get_source();
                        features.dest_port = udp_packet.get_destination();
                    }
                },
                _ => {}
            }
        }

        features
    }
