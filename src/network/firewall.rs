// src/network/firewall.rs
use std::process::Command;
use log::{info, error};

pub struct Firewall;

impl Firewall {
    pub fn block_ip(ip: &str) -> anyhow::Result<()> {
        info!("Blocking IP address: {}", ip);
        
        let output = Command::new("iptables")
            .args(&["-A", "INPUT", "-s", ip, "-j", "DROP"])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to block IP {}: {}", ip, error_msg);
            return Err(anyhow::anyhow!("Failed to block IP"));
        }

        info!("Successfully blocked IP address: {}", ip);
        Ok(())
    }

    pub fn update_rules(&self) -> anyhow::Result<()> {
        // Implement firewall rules update logic
        Ok(())
    }
}