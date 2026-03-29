#[derive(Debug, Clone)]
pub enum DiscoveryMsg {
    StartDiscovery,
    DiscoveryComplete(Vec<crate::kodi::types::HostInfo>),
    DiscoveryError(String),
    AddHost(usize),
    Cancel,
}

#[derive(Debug, Default)]
pub struct DiscoveryModel {
    pub discovered_hosts: Vec<crate::kodi::types::HostInfo>,
    pub is_discovering: bool,
}

impl DiscoveryModel {
    pub fn update(&mut self, msg: DiscoveryMsg) {
        match msg {
            DiscoveryMsg::StartDiscovery => {
                self.is_discovering = true;
                tracing::info!("Starting discovery");
            }
            DiscoveryMsg::DiscoveryComplete(hosts) => {
                self.is_discovering = false;
                self.discovered_hosts = hosts;
                tracing::info!("Discovery complete, found {} hosts", self.discovered_hosts.len());
            }
            DiscoveryMsg::DiscoveryError(e) => {
                self.is_discovering = false;
                tracing::error!("Discovery error: {}", e);
            }
            DiscoveryMsg::AddHost(_index) => {
                tracing::info!("Adding host");
            }
            DiscoveryMsg::Cancel => {
                self.is_discovering = false;
                tracing::info!("Discovery cancelled");
            }
        }
    }
}
