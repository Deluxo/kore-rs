use std::fs;
use std::path::PathBuf;

use super::Host;

const CONFIG_DIR: &str = "korers";
const HOSTS_FILE: &str = "hosts.json";

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Host not found: {0}")]
    HostNotFound(String),
}

pub struct HostManager {
    hosts: Vec<Host>,
    config_dir: PathBuf,
}

impl HostManager {
    pub fn new() -> Result<Self, ManagerError> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_DIR);

        fs::create_dir_all(&config_dir)?;

        let mut manager = Self {
            hosts: Vec::new(),
            config_dir,
        };

        manager.load()?;

        Ok(manager)
    }

    pub fn hosts(&self) -> &[Host] {
        &self.hosts
    }

    pub fn add_host(&mut self, host: Host) -> Result<(), ManagerError> {
        self.hosts.push(host);
        self.save()?;
        Ok(())
    }

    pub fn remove_host(&mut self, id: &str) -> Result<(), ManagerError> {
        let pos = self
            .hosts
            .iter()
            .position(|h| h.id == id)
            .ok_or_else(|| ManagerError::HostNotFound(id.to_string()))?;
        self.hosts.remove(pos);
        self.save()?;
        Ok(())
    }

    pub fn update_host(&mut self, host: Host) -> Result<(), ManagerError> {
        let pos = self
            .hosts
            .iter()
            .position(|h| h.id == host.id)
            .ok_or_else(|| ManagerError::HostNotFound(host.id.clone()))?;
        self.hosts[pos] = host;
        self.save()?;
        Ok(())
    }

    pub fn get_host(&self, id: &str) -> Option<&Host> {
        self.hosts.iter().find(|h| h.id == id)
    }

    pub fn get_host_mut(&mut self, id: &str) -> Option<&mut Host> {
        self.hosts.iter_mut().find(|h| h.id == id)
    }

    fn path(&self) -> PathBuf {
        self.config_dir.join(HOSTS_FILE)
    }

    fn load(&mut self) -> Result<(), ManagerError> {
        let path = self.path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            self.hosts = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save(&self) -> Result<(), ManagerError> {
        let content = serde_json::to_string_pretty(&self.hosts)?;
        fs::write(self.path(), content)?;
        Ok(())
    }
}

impl Default for HostManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            hosts: Vec::new(),
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(CONFIG_DIR),
        })
    }
}
