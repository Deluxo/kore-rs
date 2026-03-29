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

impl RemoteModel {
    pub fn update(&mut self, msg: RemoteMsg) {
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
