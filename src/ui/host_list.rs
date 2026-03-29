use gtk::prelude::*;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
};
use crate::host::Host;

#[derive(Debug, Clone)]
pub enum HostListMsg {
    SelectHost(String),
    AddHost(Host),
    RemoveHost(String),
    StartDiscovery,
    DiscoveryFinished(Vec<Host>),
}

pub struct RootWidgets;

#[derive(Debug, Default)]
pub struct HostListModel {
    pub hosts: Vec<Host>,
    pub selected_host: Option<String>,
    pub is_discovering: bool,
}

#[relm4::component]
impl SimpleComponent for HostListModel {
    type Init = ();
    type Input = HostListMsg;
    type Output = ();
    type Widgets = RootWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HostListModel::default();
        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
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
