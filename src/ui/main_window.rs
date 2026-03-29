use gtk::prelude::*;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
};
use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::HostInfo;

#[derive(Debug, Clone)]
pub enum AppMsg {
    HostSelected(String),
    DiscoverHosts,
    AddHost(Host),
    RemoveHost(String),
    RefreshHosts,
    ShowRemote,
    ShowNowPlaying,
    ShowSettings,
}

#[derive(Debug, Clone)]
pub enum HostListMsg {
    SelectHost(String),
    StartDiscovery,
    HostsFound(Vec<HostInfo>),
}

#[derive(Debug, Clone)]
pub enum RemoteMsg {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Back,
    Home,
    Info,
    PlayPause,
    Stop,
    VolumeUp,
    VolumeDown,
    Mute,
}

#[derive(Debug, Clone)]
pub enum NowPlayingMsg {
    Refresh,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek(i64),
}

pub struct RootWidgets;

#[derive(Debug, Default)]
pub struct AppModel {
    pub hosts: Vec<Host>,
    pub selected_host: Option<String>,
    pub current_view: AppView,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppView {
    #[default]
    HostList,
    Remote,
    NowPlaying,
    Settings,
}

impl AppModel {
    pub fn load_hosts(&mut self) {
        match HostManager::new() {
            Ok(manager) => {
                self.hosts = manager.hosts().to_vec();
            }
            Err(e) => {
                tracing::error!("Failed to load hosts: {}", e);
            }
        }
    }

    pub fn save_hosts(&mut self) -> Result<(), String> {
        let mut manager = HostManager::new().map_err(|e| e.to_string())?;
        for host in &self.hosts {
            if manager.get_host(&host.id).is_none() {
                manager.add_host(host.clone()).map_err(|e| e.to_string())?;
            }
        }
        self.hosts = manager.hosts().to_vec();
        Ok(())
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = RootWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = AppModel::default();
        model.load_hosts();

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::HostSelected(id) => {
                self.selected_host = Some(id.clone());
                tracing::info!("Host selected: {}", id);
            }
            AppMsg::DiscoverHosts => {
                tracing::info!("Starting host discovery...");
            }
            AppMsg::AddHost(host) => {
                if let Err(e) = self.save_hosts() {
                    tracing::error!("Failed to add host: {}", e);
                }
            }
            AppMsg::RemoveHost(id) => {
                if let Err(e) = self.save_hosts() {
                    tracing::error!("Failed to remove host: {}", e);
                }
            }
            AppMsg::RefreshHosts => {
                self.load_hosts();
            }
            AppMsg::ShowRemote => {
                self.current_view = AppView::Remote;
            }
            AppMsg::ShowNowPlaying => {
                self.current_view = AppView::NowPlaying;
            }
            AppMsg::ShowSettings => {
                self.current_view = AppView::Settings;
            }
        }
    }
}
