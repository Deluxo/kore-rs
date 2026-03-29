use crate::host::Host;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppView {
    #[default]
    HostList,
    Remote,
    NowPlaying,
    Settings,
}

#[derive(Debug, Default)]
pub struct AppModel {
    pub hosts: Vec<Host>,
    pub selected_host: Option<String>,
    pub current_view: AppView,
    pub is_connected: bool,
    pub is_discovering: bool,
}
