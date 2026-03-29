use relm4::gtk;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt,
    view,
};
use crate::kodi::discovery::DiscoveryService;

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

#[relm4::component]
impl SimpleComponent for DiscoveryModel {
    type Init = ();
    type Input = DiscoveryMsg;
    type Output = ();
    type Widgets = DiscoveryWidgets;

    view! {
        gtk::Dialog {
            set_title: Some("Discover Kodi Hosts"),
            set_modal: true,
            set_default_size: (400, 300),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                set_margin_all: 10,

                #[name = "spinner"]
                gtk::Spinner {
                    set_spinning: false,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_size_request: (50, 50),
                },

                #[name = "status_label"]
                gtk::Label {
                    set_label: "Searching for Kodi hosts...",
                    set_halign: gtk::Align::Center,
                },

                #[name = "host_list"]
                gtk::ListBox {
                    set_vexpand: true,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::End,
                    set_spacing: 10,

                    #[name = "cancel_button"]
                    gtk::Button {
                        set_label: "Cancel",
                        connect_clicked => DiscoveryMsg::Cancel,
                    },

                    #[name = "add_button"]
                    gtk::Button {
                        set_label: "Add Selected",
                        set_sensitive: false,
                        connect_clicked => DiscoveryMsg::AddHost(0),
                    },
                },
            }
        }
    }

    fn init_root() -> Self::Root {
        gtk::Dialog::builder()
            .title("Discover Kodi Hosts")
            .modal(true)
            .default_width(400)
            .default_height(300)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DiscoveryModel::default();
        let widgets = DiscoveryWidgets::from_builder(&root, ());

        ComponentParts { model, widgets }
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

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.spinner.set_spinning(self.is_discovering);
        widgets.add_button.set_sensitive(!self.is_discovering && !self.discovered_hosts.is_empty());

        if self.is_discovering {
            widgets.status_label.set_label("Searching for Kodi hosts...");
        } else if self.discovered_hosts.is_empty() {
            widgets.status_label.set_label("No hosts found");
        } else {
            widgets.status_label.set_label(&format!(
                "Found {} host(s)",
                self.discovered_hosts.len()
            ));
        }
    }
}

#[relm4::macros::widget_struct]
pub struct DiscoveryWidgets {
    pub spinner: gtk::Spinner,
    pub status_label: gtk::Label,
    pub host_list: gtk::ListBox,
    pub cancel_button: gtk::Button,
    pub add_button: gtk::Button,
}
