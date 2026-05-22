use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::host::Host;
use crate::io::host::manager::HostManager;

pub struct HostListWidgets {
    pub list: gtk::ListBox,
    pub btn_discover: gtk::Button,
    pub btn_connect: gtk::Button,
    pub btn_add: gtk::Button,
    pub btn_edit: gtk::Button,
    pub btn_del: gtk::Button,
}

pub fn create_host_list(hosts: &[Host]) -> gtk::ListBox {
    let list = gtk::ListBox::new();
    for host in hosts {
        add_host_to_list(&list, host);
    }
    list
}

pub fn create_host_manager_dialog(
    hosts: &[Host],
) -> (gtk::Dialog, HostListWidgets) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Hosts"),
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        &[
            ("Close", gtk::ResponseType::Close),
        ],
    );

    let host_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
    host_box.set_margin_start(12);
    host_box.set_margin_end(12);
    host_box.set_margin_top(12);
    host_box.set_margin_bottom(12);

    let label = gtk::Label::new(Some("Hosts"));
    label.set_halign(gtk::Align::Start);
    host_box.append(&label);

    let list = create_host_list(hosts);
    host_box.append(&list);

    let btns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let btn_discover = gtk::Button::with_label("Discover");
    let btn_connect = gtk::Button::with_label("Connect");
    let btn_add = gtk::Button::with_label("Add");
    let btn_edit = gtk::Button::with_label("Edit");
    let btn_del = gtk::Button::with_label("Del");

    btn_edit.set_sensitive(false);
    btn_del.set_sensitive(false);

    btns.append(&btn_discover);
    btns.append(&btn_connect);
    btns.append(&btn_add);
    btns.append(&btn_edit);
    btns.append(&btn_del);
    host_box.append(&btns);

    dialog.content_area().append(&host_box);

    let widgets = HostListWidgets {
        list,
        btn_discover,
        btn_connect,
        btn_add,
        btn_edit,
        btn_del,
    };

    (dialog, widgets)
}

pub fn show_add_host_dialog(
    list: &gtk::ListBox,
    host_manager: Rc<RefCell<Option<HostManager>>>,
) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Add Host"),
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        &[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Add", gtk::ResponseType::Accept),
        ],
    );
    let name_entry = gtk::Entry::new();
    name_entry.set_placeholder_text(Some("Name"));
    let addr_entry = gtk::Entry::new();
    addr_entry.set_placeholder_text(Some("Address (e.g., 192.168.1.100)"));
    let port_entry = gtk::Entry::new();
    port_entry.set_placeholder_text(Some("Port (default: 8080)"));
    port_entry.set_text("8080");
    let user_entry = gtk::Entry::new();
    user_entry.set_placeholder_text(Some("Username (optional)"));
    let pass_entry = gtk::Entry::new();
    pass_entry.set_placeholder_text(Some("Password (optional)"));
    pass_entry.set_visibility(false);

    let vb = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vb.set_margin_start(12);
    vb.set_margin_end(12);
    vb.append(&name_entry);
    vb.append(&addr_entry);
    vb.append(&port_entry);
    vb.append(&user_entry);
    vb.append(&pass_entry);

    let content = dialog.content_area();
    content.append(&vb);
    dialog.show();

    let list = list.clone();
    let mgr = host_manager.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk::ResponseType::Accept {
            let name = name_entry.text().to_string();
            let addr = addr_entry.text().to_string();
            let port: u16 = port_entry.text().parse().unwrap_or(8080);
            let user = if user_entry.text().is_empty() { None } else { Some(user_entry.text().to_string()) };
            let pass = if pass_entry.text().is_empty() { None } else { Some(pass_entry.text().to_string()) };

            if !name.is_empty() && !addr.is_empty() {
                let host = Host::new_with_credentials(name, addr, port, user, pass);
                add_host_to_list(&list, &host);
                let mut m = mgr.borrow_mut();
                if let Some(ref mut manager) = *m {
                    let _ = manager.add_host(host);
                } else {
                    tracing::warn!("Host manager not available when adding host");
                }
            }
        }
        dlg.close();
    });
}

pub fn show_edit_host_dialog(host: &Host, host_manager: Rc<RefCell<Option<HostManager>>>) {
    let dialog = gtk::Dialog::with_buttons(
        Some("Edit Host"),
        None::<&gtk::Window>,
        gtk::DialogFlags::MODAL,
        &[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Save", gtk::ResponseType::Accept),
        ],
    );
    let name_entry = gtk::Entry::new();
    name_entry.set_text(&host.name);
    let addr_entry = gtk::Entry::new();
    addr_entry.set_text(&host.address);
    let port_entry = gtk::Entry::new();
    port_entry.set_text(&host.port.to_string());
    let user_entry = gtk::Entry::new();
    user_entry.set_text(host.username.as_deref().unwrap_or(""));
    user_entry.set_placeholder_text(Some("Username (optional)"));
    let pass_entry = gtk::Entry::new();
    pass_entry.set_text(host.password.as_deref().unwrap_or(""));
    pass_entry.set_placeholder_text(Some("Password (optional)"));
    pass_entry.set_visibility(false);

    let vb = gtk::Box::new(gtk::Orientation::Vertical, 8);
    vb.set_margin_start(12);
    vb.set_margin_end(12);
    vb.append(&name_entry);
    vb.append(&addr_entry);
    vb.append(&port_entry);
    vb.append(&user_entry);
    vb.append(&pass_entry);

    let content = dialog.content_area();
    content.append(&vb);
    dialog.show();

    let mgr = host_manager.clone();
    dialog.connect_response(move |dlg, resp| {
        if resp == gtk::ResponseType::Accept {
            let name = name_entry.text().to_string();
            let addr = addr_entry.text().to_string();
            let port: u16 = port_entry.text().parse().unwrap_or(8080);
            let user = if user_entry.text().is_empty() { None } else { Some(user_entry.text().to_string()) };
            let pass = if pass_entry.text().is_empty() { None } else { Some(pass_entry.text().to_string()) };

            if !name.is_empty() && !addr.is_empty() {
                let new_host = Host::new_with_credentials(name, addr, port, user, pass);
                let mut m = mgr.borrow_mut();
                if let Some(ref mut manager) = *m {
                    let _ = manager.update_host(new_host);
                } else {
                    tracing::warn!("Host manager not available when updating host");
                }
            }
        }
        dlg.close();
    });
}

pub fn add_host_to_list(list: &gtk::ListBox, host: &Host) {
    let row = gtk::ListBoxRow::new();
    let hb = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hb.set_margin_start(12);
    hb.set_margin_end(12);
    hb.set_margin_top(8);
    hb.set_margin_bottom(8);
    let icon = gtk::Image::from_icon_name("computer");
    hb.append(&icon);
    let vb = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let name = gtk::Label::new(Some(&host.name));
    name.set_halign(gtk::Align::Start);
    name.set_hexpand(true);
    let addr = gtk::Label::new(Some(&format!("{}:{}", host.address, host.port)));
    addr.set_halign(gtk::Align::Start);
    vb.append(&name);
    vb.append(&addr);
    hb.append(&vb);
    row.set_child(Some(&hb));
    list.append(&row);
}
