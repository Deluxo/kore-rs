use crate::host::Host;

#[derive(Debug, Clone)]
pub enum HostListMsg {
    SelectHost(String),
    AddHost(Host),
    RemoveHost(String),
    StartDiscovery,
    DiscoveryFinished(Vec<Host>),
}

#[derive(Debug, Default)]
pub struct HostListModel {
    pub hosts: Vec<Host>,
    pub selected_host: Option<String>,
    pub is_discovering: bool,
}

impl HostListModel {
    pub fn update(&mut self, msg: HostListMsg) {
        match msg {
            HostListMsg::SelectHost(id) => {
                self.selected_host = Some(id.clone());
                tracing::info!("Host selected: {}", id);
            }
            HostListMsg::AddHost(host) => {
                self.hosts.push(host);
                tracing::info!("Host added");
            }
            HostListMsg::RemoveHost(id) => {
                self.hosts.retain(|h| h.id != id);
                tracing::info!("Host removed: {}", id);
            }
            HostListMsg::StartDiscovery => {
                self.is_discovering = true;
                tracing::info!("Starting discovery...");
            }
            HostListMsg::DiscoveryFinished(hosts) => {
                self.is_discovering = false;
                for host in hosts {
                    if !self.hosts.iter().any(|h| h.address == host.address) {
                        self.hosts.push(host);
                    }
                }
                tracing::info!("Discovery finished, found {} hosts", self.hosts.len());
            }
        }
    }
}
