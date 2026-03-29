use gtk::prelude::*;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
};

#[derive(Debug, Clone)]
pub enum DiscoveryMsg {
    StartDiscovery,
    DiscoveryComplete(Vec<crate::kodi::types::HostInfo>),
    DiscoveryError(String),
    AddHost(usize),
    Cancel,
}

pub struct RootWidgets;

#[derive(Debug, Default)]
pub struct DiscoveryModel {
    pub discovered_hosts: Vec<crate::kodi::types::HostInfo>,
    pub is_discovering: bool,
}

#[relm4::component]
impl SimpleComponent for DiscoveryModel {
    type Init = ();
    type Input = DiscoveryMsg;
    type Output = ();
    type Widgets = RootWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DiscoveryModel::default();
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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
