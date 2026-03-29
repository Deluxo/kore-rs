use gtk::prelude::*;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
};

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

pub struct RootWidgets;

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
    type Widgets = RootWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RemoteModel::default();
        ComponentParts { model, widgets: () }
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
}
