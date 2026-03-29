use relm4::gtk;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, RelmApp, RelmWidgetExt,
    view, view_tree,
};
use crate::host::{Host, HostManager};
use crate::kodi::{KodiClient, HostInfo};

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
    type Widgets = AppWidgets;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Korers - Kodi Remote"),
            set_default_size: (900, 700),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "header_bar"]
                gtk::HeaderBar {
                    set_show_title_buttons: true,
                    set_title_widget: Some(&gtk::Label::new(Some("Korers"))),
                },

                #[name = "content_box"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[name = "sidebar"]
                    gtk::StackSidebar {
                        set_width_request: 200,
                    },

                    #[name = "content_stack"]
                    gtk::Stack {
                        set_hexpand: true,
                    },
                },

                gtk::Statusbar {
                    #[name = "statusbar"]
                    set_margin_top: 5,
                },
            }
        }
    }

    fn init_root() -> Self::Root {
        gtk::ApplicationWindow::builder()
            .title("Korers - Kodi Remote")
            .default_width(900)
            .default_height(700)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = AppModel::default();
        model.load_hosts();

        let widgets = AppWidgets::from_builder(&root, ());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
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

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
    }
}

#[relm4::macros::widget_struct]
pub struct AppWidgets {
    pub header_bar: gtk::HeaderBar,
    pub content_box: gtk::Box,
    pub sidebar: gtk::StackSidebar,
    pub content_stack: gtk::Stack,
    pub statusbar: gtk::Statusbar,
}
