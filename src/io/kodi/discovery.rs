use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use super::types::HostInfo;

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

    pub fn discover_ssdp(&self, timeout_secs: u64) -> Result<Vec<HostInfo>, DiscoveryError> {
        let multicast_addr: SocketAddr = (Ipv4Addr::new(239, 255, 255, 250), 1900).into();
        let bind_addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, 0).into();

        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;

        let msearch = [
            "M-SEARCH * HTTP/1.1",
            "HOST: 239.255.255.250:1900",
            "MAN: \"ssdp:discover\"",
            "MX: 3",
            "ST: ssdp:all",
            "",
            "",
        ]
        .join("\r\n");

        socket.send_to(msearch.as_bytes(), multicast_addr)?;

        let mut hosts: Vec<HostInfo> = Vec::new();
        let mut buf = [0u8; 4096];
        let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

        while std::time::Instant::now() < deadline {
            match socket.recv_from(&mut buf) {
                Ok((len, _src)) => {
                    let response = String::from_utf8_lossy(&buf[..len]);
                    if let Some(info) = Self::parse_ssdp_response(&response) {
                        let addr = info.address.clone();
                        if !hosts.iter().any(|h| h.address == addr) {
                            hosts.push(info);
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(hosts)
    }

    pub fn discover_all(&self, timeout_secs: u64) -> Result<Vec<HostInfo>, DiscoveryError> {
        let hosts = self.discover_ssdp(timeout_secs)?;
        if hosts.is_empty() {
            Err(DiscoveryError::NoHostsFound)
        } else {
            Ok(hosts)
        }
    }

    pub fn discover_single(&self, timeout_secs: u64) -> Result<HostInfo, DiscoveryError> {
        let hosts = self.discover_all(timeout_secs)?;
        Ok(hosts.into_iter().next().unwrap())
    }

    fn parse_ssdp_response(response: &str) -> Option<HostInfo> {
        let mut name = None;
        let mut location = None;

        for line in response.lines() {
            let lower = line.to_lowercase();
            if lower.starts_with("server:") {
                name = line.split(':').nth(1).map(|s| s.trim().to_string());
            } else if lower.starts_with("x-user-agent:") {
                name = line.split(':').nth(1).map(|s| s.trim().to_string());
            } else if lower.starts_with("location:") {
                location = line.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }

        let location = location.or_else(|| {
            response.lines().find_map(|line| {
                let lower = line.to_lowercase();
                if lower.starts_with("location:") {
                    line.split(':').nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
        })?;

        let (address, port) = Self::parse_location(&location);
        let hostname = name.unwrap_or_else(|| address.clone());

        Some(HostInfo {
            name: hostname,
            address,
            port,
            mac_address: None,
            wol_port: None,
        })
    }

    fn parse_location(location: &str) -> (String, u16) {
        let location = location.trim();
        if location.starts_with("http://") || location.starts_with("https://") {
            let rest = location.trim_start_matches("http://")
                .trim_start_matches("https://");
            if let Some(host_port) = rest.split('/').next() {
                if let Some(port_str) = host_port.split(':').nth(1) {
                    if let Ok(port) = port_str.parse::<u16>() {
                        let host = host_port.split(':').next().unwrap_or(host_port);
                        return (host.to_string(), port);
                    }
                }
                return (host_port.to_string(), 8080);
            }
        } else if location.starts_with("//") {
            let rest = location.trim_start_matches("//");
            if let Some(host_port) = rest.split('/').next() {
                if let Some(port_str) = host_port.split(':').nth(1) {
                    if let Ok(port) = port_str.parse::<u16>() {
                        let host = host_port.split(':').next().unwrap_or(host_port);
                        return (host.to_string(), port);
                    }
                }
                return (host_port.to_string(), 8080);
            }
        }

        // Try to extract IP with regex
        let re = regex_lite::Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").unwrap();
        if let Some(caps) = re.captures(location) {
            let ip = caps.get(1).unwrap().as_str().to_string();
            // Try to find port
            if let Some(port_str) = location.split(':').last() {
                if let Ok(port) = port_str.trim_end_matches('/').parse::<u16>() {
                    return (ip, port);
                }
            }
            return (ip, 8080);
        }

        ("0.0.0.0".to_string(), 8080)
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}
