use relm4::gtk;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt,
    view,
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
    type Widgets = HostListWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 10,

            gtk::Label {
                set_label: "Kodi Hosts",
                set_markup: Some("<span size='large' weight='bold'>Kodi Hosts</span>"),
                set_halign: gtk::Align::Start,
                set_margin_bottom: 10,
            },

            #[name = "discovery_button"]
            gtk::Button {
                set_label: "🔍 Discover Hosts",
                set_halign: gtk::Align::Fill,
                connect_clicked => HostListMsg::StartDiscovery,
            },

            gtk::Separator {
                set_margin_vertical: 10,
            },

            #[name = "host_list"]
            gtk::ListBox {
                set_vexpand: true,
                set_hexpand: true,
            },

            gtk::Button {
                set_label: "+ Add Host Manually",
                set_halign: gtk::Align::Fill,
                set_margin_top: 10,
            },
        }
    }

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .margin_all(10)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HostListModel::default();
        let widgets = HostListWidgets::from_builder(&root, ());

        ComponentParts { model, widgets }
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

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.discovery_button.set_sensitive(!self.is_discovering);
        if self.is_discovering {
            widgets.discovery_button.set_label("🔍 Discovering...");
        } else {
            widgets.discovery_button.set_label("🔍 Discover Hosts");
        }
    }
}

#[relm4::macros::widget_struct]
pub struct HostListWidgets {
    pub discovery_button: gtk::Button,
    pub host_list: gtk::ListBox,
}
