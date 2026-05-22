use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::host::Host;
use crate::io::host::manager::HostManager;
use crate::io::kodi::DiscoveryService;
use crate::io::kodi::KodiClient;

use super::layout::{show_add_host_dialog, show_edit_host_dialog, HostListWidgets};

pub struct HostListState {
    host_manager: Rc<RefCell<Option<HostManager>>>,
    client: Rc<RefCell<Option<KodiClient>>>,
}

impl HostListState {
    pub fn new(
        host_manager: Rc<RefCell<Option<HostManager>>>,
        client: Rc<RefCell<Option<KodiClient>>>,
    ) -> Self {
        Self {
            host_manager,
            client,
        }
    }
}

pub fn connect_host_dialog_handlers(dialog: &gtk::Dialog, widgets: &HostListWidgets, state: &HostListState) {
    let mgr = state.host_manager.clone();
    widgets.btn_discover.connect_clicked(move |_| {
        let discovery = DiscoveryService::new();
        match discovery.discover_all(3) {
            Ok(infos) => {
                for info in infos {
                    let host = Host::new(info.name.clone(), info.address.clone(), info.port);
                    let mut m = mgr.borrow_mut();
                    if let Some(ref mut manager) = *m {
                        let _ = manager.add_host(host);
                    } else {
                        tracing::warn!("Host manager not available during discovery");
                    }
                }
            }
            Err(e) => tracing::warn!("Discovery failed: {:?}", e),
        }
    });

    let list = widgets.list.clone();
    let client = state.client.clone();
    let mgr = state.host_manager.clone();
    widgets.btn_connect.connect_clicked(move |_| {
        let m = mgr.borrow();
        let host = list.selected_row()
            .and_then(|row| m.as_ref()
                .and_then(|manager| manager.hosts().get(row.index() as usize)));
        if let Some(host) = host {
            let kodi_client = KodiClient::from_host(host);
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::warn!("Failed to create runtime: {}", e);
                    return;
                }
            };
            if rt.block_on(kodi_client.ping()).is_ok() {
                client.replace(Some(kodi_client));
                tracing::info!("Connected to {}", host.name);
            } else {
                tracing::warn!("Failed to ping {}", host.name);
            }
        } else {
            tracing::warn!("No host selected or host not found");
        }
    });

    let list = widgets.list.clone();
    let mgr = state.host_manager.clone();
    widgets.btn_add.connect_clicked(move |_| {
        show_add_host_dialog(&list, mgr.clone());
    });

    let list = widgets.list.clone();
    let mgr = state.host_manager.clone();
    widgets.btn_edit.connect_clicked(move |_| {
        let m = mgr.borrow();
        let host = list.selected_row()
            .and_then(|row| m.as_ref()
                .and_then(|manager| manager.hosts().get(row.index() as usize)));
        if let Some(host) = host {
            show_edit_host_dialog(host, mgr.clone());
        } else {
            tracing::warn!("No host selected or host not found");
        }
    });

    let list = widgets.list.clone();
    let mgr = state.host_manager.clone();
    widgets.btn_del.connect_clicked(move |_| {
        if let Some(row) = list.selected_row() {
            let host_id = {
                let m = mgr.borrow();
                list.selected_row()
                    .and_then(|r| m.as_ref()
                        .and_then(|manager| manager.hosts().get(r.index() as usize)))
                    .map(|h| h.id.clone())
            };
            if let Some(id) = host_id {
                if let Ok(mut mm) = mgr.try_borrow_mut() {
                    if let Some(ref mut manager) = *mm {
                        let _ = manager.remove_host(&id);
                    } else {
                        tracing::warn!("Host manager not available during delete");
                    }
                } else {
                    tracing::warn!("Could not borrow host manager mutably during delete");
                }
            } else {
                tracing::warn!("Host not found during delete");
            }
            list.remove(&row);
        } else {
            tracing::debug!("No row selected for delete");
        }
    });

    let edit = widgets.btn_edit.clone();
    let del = widgets.btn_del.clone();
    widgets.list.connect_row_selected(move |_, row| {
        let selected = row.is_some();
        edit.set_sensitive(selected);
        del.set_sensitive(selected);
    });

    let dialog = dialog.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk::ResponseType::Close {
            dlg.close();
        }
    });
}
