use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

use crate::kodi::types::HostInfo;

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Timeout waiting for responses")]
    Timeout,
    #[error("No hosts found")]
    NoHostsFound,
}

pub struct DiscoveryService;

impl DiscoveryService {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(&self, timeout_secs: u64) -> Result<Vec<HostInfo>, DiscoveryError> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;

        let message = b"MSEARCH";
        socket.send_to(message, (Ipv4Addr::BROADCAST, 9770))?;

        let mut hosts = Vec::new();
        let mut buffer = vec![0u8; 4096];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((size, _addr)) => {
                    let response = String::from_utf8_lossy(&buffer[..size]);
                    if let Some(host) = Self::parse_response(&response) {
                        if !hosts.iter().any(|h: &HostInfo| h.address == host.address) {
                            hosts.push(host);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }

        if hosts.is_empty() {
            return Err(DiscoveryError::NoHostsFound);
        }

        Ok(hosts)
    }

    fn parse_response(response: &str) -> Option<HostInfo> {
        let mut name = None;
        let mut address = None;
        let mut port = 8080;

        for line in response.lines() {
            let line = line.trim();
            if line.starts_with("Location:") || line.starts_with("location:") {
                if let Some(location) = line.split(':').skip(2).collect::<Vec<_>>().join(":").split(':').last() {
                    let parts: Vec<&str> = location.trim_start_matches('/').split(':').collect();
                    if parts.len() >= 1 {
                        address = Some(parts[0].to_string());
                    }
                    if parts.len() >= 2 {
                        port = parts[1].parse().unwrap_or(8080);
                    }
                }
            }
            if line.starts_with("Server:") || line.starts_with("server:") {
                let value = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                if !value.is_empty() {
                    name = Some(value);
                }
            }
        }

        let name_for_ip = name.clone();

        if let (Some(addr), Some(n)) = (address, name) {
            return Some(HostInfo::new(n, addr, port));
        }

        let ip_pattern = regex_lite::Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").ok()?;
        if let Some(caps) = ip_pattern.captures(response) {
            let address = caps.get(1)?.as_str().to_string();
            let host_name = name_for_ip.unwrap_or_else(|| format!("Kodi @ {}", address));
            return Some(HostInfo::new(host_name, address, port));
        }

        None
    }

    pub fn discover_single(&self, timeout_secs: u64) -> Result<HostInfo, DiscoveryError> {
        let hosts = self.discover(timeout_secs)?;
        hosts.into_iter().next().ok_or(DiscoveryError::NoHostsFound)
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}
