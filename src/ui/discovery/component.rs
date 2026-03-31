use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::DiscoveryService;

pub struct DiscoveryWidgets {
    pub btn_discover: gtk::Button,
}

pub struct DiscoveryState {
    host_manager: Rc<RefCell<Option<HostManager>>>,
}

impl DiscoveryState {
    pub fn new(host_manager: Rc<RefCell<Option<HostManager>>>) -> Self {
        Self { host_manager }
    }
}

pub fn create_discover_button(host_manager: Rc<RefCell<Option<HostManager>>>) -> (gtk::Button, DiscoveryState) {
    let state = DiscoveryState::new(host_manager);
    let btn_discover = gtk::Button::with_label("Discover");
    (btn_discover, state)
}

pub fn connect_discover_handler(btn: &gtk::Button, state: &DiscoveryState) {
    let mgr = state.host_manager.clone();
    btn.connect_clicked(move |_| {
        let discovery = DiscoveryService::new();
        match discovery.discover_all(3) {
            Ok(infos) => {
                for info in infos {
                    let host = Host::new(info.name.clone(), info.address.clone(), info.port);
                    let mut m = mgr.borrow_mut();
                    if let Some(ref mut manager) = *m {
                        let _ = manager.add_host(host);
                    }
                }
            }
            Err(e) => tracing::warn!("Discovery failed: {:?}", e),
        }
    });
}
