use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
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

    pub fn discover_ssdp(&self, timeout_secs: u64) -> Result<Vec<HostInfo>, DiscoveryError> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.set_nonblocking(false)?;

        let multicast_addr: SocketAddr = (Ipv4Addr::new(239, 255, 255, 250), 1900).into();

        let msearch = "MSEARCH * HTTP/1.1\r\n\
            HOST: 239.255.255.250:1900\r\n\
            MAN: \"ssdp:discover\"\r\n\
            MX: 3\r\n\
            ST: ssdp:all\r\n\
            USER-AGENT: korers/0.1\r\n\
            \r\n";

        socket.send_to(msearch.as_bytes(), multicast_addr)?;

        socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;

        let mut hosts = Vec::new();
        let mut buffer = vec![0u8; 4096];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((size, _addr)) => {
                    let response = String::from_utf8_lossy(&buffer[..size]);
                    if let Some(host) = Self::parse_ssdp_response(&response) {
                        if !hosts.iter().any(|h: &HostInfo| h.address == host.address) {
                            hosts.push(host);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }

        Ok(hosts)
    }

    pub fn discover_all(&self, timeout_secs: u64) -> Result<Vec<HostInfo>, DiscoveryError> {
        let hosts = self.discover_ssdp(timeout_secs)?;

        if hosts.is_empty() {
            return Err(DiscoveryError::NoHostsFound);
        }

        Ok(hosts)
    }

    pub fn discover_single(&self, timeout_secs: u64) -> Result<HostInfo, DiscoveryError> {
        let hosts = self.discover_all(timeout_secs)?;
        hosts.into_iter().next().ok_or(DiscoveryError::NoHostsFound)
    }

    fn parse_ssdp_response(response: &str) -> Option<HostInfo> {
        let mut location: Option<(String, u16)> = None;
        let mut name = None;

        for line in response.lines() {
            let line = line.trim();
            if line.to_uppercase().starts_with("LOCATION:") {
                if let Some(loc) = Self::parse_location(line) {
                    location = Some(loc);
                }
            }
            if line.to_uppercase().starts_with("SERVER:") || line.to_uppercase().starts_with("X-USER-AGENT:") {
                let value = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();
                if !value.is_empty() && name.is_none() {
                    name = Some(value);
                }
            }
        }

        if let Some((address, port)) = location {
            let name = name.unwrap_or_else(|| format!("Kodi @ {}", address));
            return Some(HostInfo::new(name, address, port));
        }

        let ip_pattern = regex_lite::Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").ok()?;
        if let Some(caps) = ip_pattern.captures(response) {
            let address = caps.get(1)?.as_str().to_string();
            let name = name.unwrap_or_else(|| format!("Kodi @ {}", address));
            return Some(HostInfo::new(name, address, 8080));
        }

        None
    }

    fn parse_location(line: &str) -> Option<(String, u16)> {
        let url = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_string();

        if let Ok(parsed) = url::Url::parse(&url) {
            let host = parsed.host_str()?.to_string();
            let port = parsed.port().unwrap_or(8080);
            return Some((host, port));
        }

        if url.starts_with("//") {
            let without_slashes = url.trim_start_matches('/');
            if let Some(colon_pos) = without_slashes.rfind(':') {
                let address = without_slashes[..colon_pos].to_string();
                let port: u16 = without_slashes[colon_pos + 1..]
                    .split('/')
                    .next()?
                    .parse()
                    .ok()?;
                return Some((address, port));
            }
        }

        None
    }
}

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}
