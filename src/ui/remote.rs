use relm4::gtk;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt,
    view,
};
use crate::kodi::{KodiClient, InputAction};

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
    ContextMenu,
}

#[derive(Debug, Default)]
pub struct RemoteModel {
    pub is_connected: bool,
    pub current_view: Option<String>,
}

#[relm4::component]
impl SimpleComponent for RemoteModel {
    type Init = ();
    type Input = RemoteMsg;
    type Output = ();
    type Widgets = RemoteWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_spacing: 20,
            set_margin_all: 20,

            gtk::Grid {
                set_row_spacing: 10,
                set_column_spacing: 10,
                set_halign: gtk::Align::Center,

                #[row = 0]
                #[column = 1]
                #[name = "btn_up"]
                gtk::Button {
                    set_label: "▲",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => RemoteMsg::NavigateUp,
                },

                #[row = 1]
                #[column = 0]
                #[name = "btn_left"]
                gtk::Button {
                    set_label: "◀",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => RemoteMsg::NavigateLeft,
                },

                #[row = 1]
                #[column = 1]
                #[name = "btn_select"]
                gtk::Button {
                    set_label: "●",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => RemoteMsg::Select,
                },

                #[row = 1]
                #[column = 2]
                #[name = "btn_right"]
                gtk::Button {
                    set_label: "▶",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => RemoteMsg::NavigateRight,
                },

                #[row = 2]
                #[column = 1]
                #[name = "btn_down"]
                gtk::Button {
                    set_label: "▼",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => RemoteMsg::NavigateDown,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,

                #[name = "btn_back"]
                gtk::Button {
                    set_label: "Back",
                    set_width_request: 80,
                    connect_clicked => RemoteMsg::Back,
                },

                #[name = "btn_home"]
                gtk::Button {
                    set_label: "Home",
                    set_width_request: 80,
                    connect_clicked => RemoteMsg::Home,
                },

                #[name = "btn_info"]
                gtk::Button {
                    set_label: "Info",
                    set_width_request: 80,
                    connect_clicked => RemoteMsg::Info,
                },

                #[name = "btn_context"]
                gtk::Button {
                    set_label: "Menu",
                    set_width_request: 80,
                    connect_clicked => RemoteMsg::ContextMenu,
                },
            },

            gtk::Separator {
                set_margin_vertical: 20,
            },

            gtk::Label {
                set_label: "Transport Controls",
                set_margin_bottom: 10,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,

                gtk::Button {
                    set_label: "⏮",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "⏯",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "⏹",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "⏭",
                    set_width_request: 50,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 20,

                gtk::Button {
                    set_label: "🔊-",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "🔇",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "🔊+",
                    set_width_request: 50,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 20,

                gtk::Button {
                    set_label: "1",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "2",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "3",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "4",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "5",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "6",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "7",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "8",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "9",
                    set_width_request: 40,
                },
                gtk::Button {
                    set_label: "0",
                    set_width_request: 40,
                },
            },
        }
    }

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RemoteModel::default();
        let widgets = RemoteWidgets::from_builder(&root, ());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            RemoteMsg::NavigateUp => tracing::debug!("Navigate Up"),
            RemoteMsg::NavigateDown => tracing::debug!("Navigate Down"),
            RemoteMsg::NavigateLeft => tracing::debug!("Navigate Left"),
            RemoteMsg::NavigateRight => tracing::debug!("Navigate Right"),
            RemoteMsg::Select => tracing::debug!("Select"),
            RemoteMsg::Back => tracing::debug!("Back"),
            RemoteMsg::Home => tracing::debug!("Home"),
            RemoteMsg::Info => tracing::debug!("Info"),
            RemoteMsg::ContextMenu => tracing::debug!("Context Menu"),
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}

#[relm4::macros::widget_struct]
pub struct RemoteWidgets {
    pub btn_up: gtk::Button,
    pub btn_down: gtk::Button,
    pub btn_left: gtk::Button,
    pub btn_right: gtk::Button,
    pub btn_select: gtk::Button,
    pub btn_back: gtk::Button,
    pub btn_home: gtk::Button,
    pub btn_info: gtk::Button,
    pub btn_context: gtk::Button,
}
