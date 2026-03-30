use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::KodiClient;
use crate::kodi::client::InputAction;
use crate::kodi::discovery::DiscoveryService;

pub struct App {
    app: Application,
    hosts: Vec<Host>,
    host_manager: Rc<RefCell<Option<HostManager>>>,
}

impl App {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id("org.korers.app")
            .build();
        Self { app, hosts: Vec::new(), host_manager: Rc::new(RefCell::new(None)) }
    }

    pub fn init_logging(self) -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into()),
            )
            .init();
        tracing::info!("Starting Korers");
        self
    }

    pub fn load_hosts(mut self) -> Self {
        if let Ok(mgr) = HostManager::new() {
            self.hosts = mgr.hosts().to_vec();
            *self.host_manager.borrow_mut() = Some(mgr);
        }
        self
    }

    pub fn show_window(self) -> Self {
        let hosts = self.hosts.clone();
        let host_manager = self.host_manager.clone();
        
        self.app.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::builder()
                .application(app)
                .title("Korers")
                .default_width(400)
                .default_height(700)
                .build();

            // Header
            let header = gtk::HeaderBar::builder()
                .title_widget(&gtk::Label::new(Some("Korers")))
                .show_title_buttons(true)
                .build();
            let menu_btn = gtk::MenuButton::builder().icon_name("open-menu").build();
            header.pack_start(&menu_btn);

            // Popover for hosts
            let popover = gtk::Popover::new();
            let client: Rc<RefCell<Option<KodiClient>>> = Rc::new(RefCell::new(None));
            
            let host_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
            host_box.set_width_request(320);
            host_box.set_margin_start(12); host_box.set_margin_end(12); 
            host_box.set_margin_top(12); host_box.set_margin_bottom(12);

            let label = gtk::Label::new(Some("Hosts"));
            label.set_halign(gtk::Align::Start);
            host_box.append(&label);

            let list = gtk::ListBox::new();
            for host in &hosts {
                add_host_to_list(&list, host);
            }
            host_box.append(&list);

            let btns = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let discover = gtk::Button::with_label("Discover");
            let connect = gtk::Button::with_label("Connect");
            let add = gtk::Button::with_label("Add");
            let edit = gtk::Button::with_label("Edit");
            let del = gtk::Button::with_label("Del");
            btns.append(&discover); btns.append(&connect); btns.append(&add); btns.append(&edit); btns.append(&del);
            host_box.append(&btns);

            let list = list.clone();
            let client = client.clone();
            let mgr = host_manager.clone();
            discover.connect_clicked(move |_| {
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

            let connect = connect.clone();
            let list_conn = list.clone();
            let client_conn = client.clone();
            let mgr_conn = host_manager.clone();
            connect.connect_clicked(move |_| {
                if let Some(row) = list_conn.selected_row() {
                    let idx = row.index() as usize;
                    let m = mgr_conn.borrow();
                    if let Some(manager) = m.as_ref() {
                        if let Some(host) = manager.hosts().get(idx) {
                            let kodi_client = KodiClient::from_host(host);
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            if rt.block_on(kodi_client.ping()).is_ok() {
                                client_conn.replace(Some(kodi_client));
                                tracing::info!("Connected to {}", host.name);
                            } else {
                                tracing::warn!("Failed to ping {}", host.name);
                            }
                        }
                    }
                }
            });

            let add = add.clone();
            let list_for_add = list.clone();
            add.connect_clicked(move |_| {
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
                vb.set_margin_start(12); vb.set_margin_end(12);
                vb.append(&name_entry);
                vb.append(&addr_entry);
                vb.append(&port_entry);
                vb.append(&user_entry);
                vb.append(&pass_entry);

                let content = dialog.content_area();
                content.append(&vb);
                dialog.show();

                let list = list_for_add.clone();
                dialog.connect_response(move |dlg, resp| {
                    if resp == gtk::ResponseType::Accept {
                        let name = name_entry.text().to_string();
                        let addr = addr_entry.text().to_string();
                        let port: u16 = port_entry.text().parse().unwrap_or(8080);
                        let user = if user_entry.text().is_empty() { None } else { Some(user_entry.text().to_string()) };
                        let pass = if pass_entry.text().is_empty() { None } else { Some(pass_entry.text().to_string()) };
                        if !name.is_empty() && !addr.is_empty() {
                            let host = Host::new_with_credentials(name, addr, port, user, pass);
                            let row = gtk::ListBoxRow::new();
                            let hb = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                            hb.set_margin_start(12); hb.set_margin_end(12);
                            hb.set_margin_top(8); hb.set_margin_bottom(8);
                            let icon = gtk::Image::from_icon_name("computer");
                            hb.append(&icon);
                            let vb = gtk::Box::new(gtk::Orientation::Vertical, 2);
                            let lbl = gtk::Label::new(Some(&host.name));
                            lbl.set_halign(gtk::Align::Start); lbl.set_hexpand(true);
                            let addr_lbl = gtk::Label::new(Some(&format!("{}:{}", host.address, host.port)));
                            addr_lbl.set_halign(gtk::Align::Start);
                            vb.append(&lbl); vb.append(&addr_lbl);
                            hb.append(&vb);
                            row.set_child(Some(&hb));
                            list.append(&row);
                        }
                    }
                    dlg.close();
                });
            });

            let list2 = list.clone();
            let edit2 = edit.clone();
            let del2 = del.clone();
            list2.connect_row_selected(move |_, row| {
                let selected = row.is_some();
                edit2.set_sensitive(selected);
                del2.set_sensitive(selected);
            });

            let list_edit = list.clone();
            let mgr_edit = host_manager.clone();
            edit.connect_clicked(move |_| {
                if let Some(row) = list_edit.selected_row() {
                    let idx = row.index() as usize;
                    let m = mgr_edit.borrow();
                    if let Some(manager) = m.as_ref() {
                        if let Some(host) = manager.hosts().get(idx) {
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
                            vb.set_margin_start(12); vb.set_margin_end(12);
                            vb.append(&name_entry);
                            vb.append(&addr_entry);
                            vb.append(&port_entry);
                            vb.append(&user_entry);
                            vb.append(&pass_entry);

                            let content = dialog.content_area();
                            content.append(&vb);
                            dialog.show();

                            let host_id = host.id.clone();
                            let mgr_resp = mgr_edit.clone();
                            dialog.connect_response(move |dlg, resp| {
                                if resp == gtk::ResponseType::Accept {
                                    let name = name_entry.text().to_string();
                                    let addr = addr_entry.text().to_string();
                                    let port: u16 = port_entry.text().parse().unwrap_or(8080);
                                    let user = if user_entry.text().is_empty() { None } else { Some(user_entry.text().to_string()) };
                                    let pass = if pass_entry.text().is_empty() { None } else { Some(pass_entry.text().to_string()) };
                                    if !name.is_empty() && !addr.is_empty() {
                                        let new_host = Host::new_with_credentials(name, addr, port, user, pass);
                                        let mut m = mgr_resp.borrow_mut();
                                        if let Some(ref mut manager) = *m {
                                            let _ = manager.update_host(new_host);
                                        }
                                    }
                                }
                                dlg.close();
                            });
                        }
                    }
                }
            });

            let list3 = list.clone();
            let del3 = del.clone();
            let mgr3 = host_manager.clone();
            del3.connect_clicked(move |_| {
                if let Some(row) = list3.selected_row() {
                    let idx = row.index() as usize;
                    let m = mgr3.borrow();
                    if let Some(manager) = m.as_ref() {
                        if let Some(hosts) = manager.hosts().get(idx) {
                            let host_id = hosts.id.clone();
                            drop(m);
                            if let Ok(mut mm) = mgr3.try_borrow_mut() {
                                if let Some(ref mut manager) = *mm {
                                    let _ = manager.remove_host(&host_id);
                                }
                            }
                        }
                    }
                    list3.remove(&row);
                }
            });

            popover.set_child(Some(&host_box));
            menu_btn.set_popover(Some(&popover));

            // Main content
            let content = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .hexpand(true)
                .vexpand(true)
                .build();

            // Now Playing
            let now_playing = create_now_playing_box(client.clone());
            content.append(&now_playing);

            // Remote
            let remote = create_remote_box(client.clone());
            content.append(&remote);

            window.set_titlebar(Some(&header));
            window.set_child(Some(&content));
            window.show();
        });
        self
    }

    pub fn show_host_selection(self) -> Self { self }
    pub fn run(self) { self.app.run(); }
}

impl Default for App { fn default() -> Self { Self::new() } }

fn create_now_playing_box(client: Rc<RefCell<Option<KodiClient>>>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.set_margin_start(12); box_.set_margin_end(12); box_.set_margin_top(12); box_.set_margin_bottom(12);
    box_.set_hexpand(true);

    let title = gtk::Label::new(Some("<big><b>No media playing</b></big>"));
    title.set_halign(gtk::Align::Center);
    title.set_use_markup(true);
    box_.append(&title);

    let artist = gtk::Label::new(Some(""));
    artist.set_halign(gtk::Align::Center);
    box_.append(&artist);

    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transport.set_halign(gtk::Align::Center);

    let prev = gtk::Button::with_label("⏮");
    let play = gtk::Button::with_label("▶/⏸");
    let stop = gtk::Button::with_label("⏹");
    let next = gtk::Button::with_label("⏭");

    let c = client.clone();
    prev.connect_clicked(move |_| transport_action(&c, "previous"));
    let c = client.clone();
    play.connect_clicked(move |_| transport_action(&c, "play_pause"));
    let c = client.clone();
    stop.connect_clicked(move |_| transport_action(&c, "stop"));
    let c = client.clone();
    next.connect_clicked(move |_| transport_action(&c, "next"));

    transport.append(&prev); transport.append(&play); transport.append(&stop); transport.append(&next);
    box_.append(&transport);

    box_
}

fn create_remote_box(client: Rc<RefCell<Option<KodiClient>>>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 16);
    box_.set_margin_start(12); box_.set_margin_end(12); box_.set_margin_top(12); box_.set_margin_bottom(12);
    box_.set_hexpand(true); box_.set_vexpand(true);

    // Nav
    let nav = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    nav.set_halign(gtk::Align::Center);
    let back = gtk::Button::with_label("Back");
    let home = gtk::Button::with_label("Home");
    let info = gtk::Button::with_label("Info");
    let c = client.clone();
    back.connect_clicked(move |_| send_input(&c, InputAction::Back));
    let c = client.clone();
    home.connect_clicked(move |_| send_input(&c, InputAction::Home));
    let c = client.clone();
    info.connect_clicked(move |_| send_input(&c, InputAction::Info));
    nav.append(&back); nav.append(&home); nav.append(&info);
    box_.append(&nav);

    // D-pad
    let dpad = gtk::Box::new(gtk::Orientation::Vertical, 4);
    dpad.set_halign(gtk::Align::Center);
    let up = gtk::Button::with_label("▲");
    let mid = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let left = gtk::Button::with_label("◀");
    let ok = gtk::Button::with_label("OK");
    let right = gtk::Button::with_label("▶");
    let down = gtk::Button::with_label("▼");
    
    let c = client.clone();
    up.connect_clicked(move |_| send_input(&c, InputAction::Up));
    let c = client.clone();
    left.connect_clicked(move |_| send_input(&c, InputAction::Left));
    let c = client.clone();
    ok.connect_clicked(move |_| send_input(&c, InputAction::Select));
    let c = client.clone();
    right.connect_clicked(move |_| send_input(&c, InputAction::Right));
    let c = client.clone();
    down.connect_clicked(move |_| send_input(&c, InputAction::Down));

    mid.append(&left); mid.append(&ok); mid.append(&right);
    dpad.append(&up); dpad.append(&mid); dpad.append(&down);
    box_.append(&dpad);

    // Transport
    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transport.set_halign(gtk::Align::Center);
    let t_prev = gtk::Button::with_label("⏮");
    let t_play = gtk::Button::with_label("▶/⏸");
    let t_stop = gtk::Button::with_label("⏹");
    let t_next = gtk::Button::with_label("⏭");

    let c = client.clone();
    t_prev.connect_clicked(move |_| transport_action(&c, "previous"));
    let c = client.clone();
    t_play.connect_clicked(move |_| transport_action(&c, "play_pause"));
    let c = client.clone();
    t_stop.connect_clicked(move |_| transport_action(&c, "stop"));
    let c = client.clone();
    t_next.connect_clicked(move |_| transport_action(&c, "next"));

    transport.append(&t_prev); transport.append(&t_play); transport.append(&t_stop); transport.append(&t_next);
    box_.append(&transport);

    // Volume
    let volume = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    volume.set_halign(gtk::Align::Center);
    let mute = gtk::Button::with_label("🔇");
    let v_down = gtk::Button::with_label("🔉");
    let v_up = gtk::Button::with_label("🔊");

    let c = client.clone();
    mute.connect_clicked(move |_| volume_mute(&c));
    let c = client.clone();
    v_down.connect_clicked(move |_| volume_change(&c, -10));
    let c = client.clone();
    v_up.connect_clicked(move |_| volume_change(&c, 10));

    volume.append(&mute); volume.append(&v_down); volume.append(&v_up);
    box_.append(&volume);

    box_
}

fn add_host_to_list(list: &gtk::ListBox, host: &Host) {
    let row = gtk::ListBoxRow::new();
    let hb = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    hb.set_margin_start(12); hb.set_margin_end(12); hb.set_margin_top(8); hb.set_margin_bottom(8);
    let icon = gtk::Image::from_icon_name("computer");
    hb.append(&icon);
    let vb = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let name = gtk::Label::new(Some(&host.name));
    name.set_halign(gtk::Align::Start); name.set_hexpand(true);
    let addr = gtk::Label::new(Some(&format!("{}:{}", host.address, host.port)));
    addr.set_halign(gtk::Align::Start);
    vb.append(&name); vb.append(&addr);
    hb.append(&vb);
    row.set_child(Some(&hb));
    list.append(&row);
}

fn transport_action(client: &Rc<RefCell<Option<KodiClient>>>, action: &str) {
    if let Some(ref c) = *client.borrow() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(players) = rt.block_on(c.get_active_players()) {
            if let Some(p) = players.first() {
                match action {
                    "play_pause" => { let _ = rt.block_on(c.play_pause(p.playerid)); }
                    "stop" => { let _ = rt.block_on(c.stop(p.playerid)); }
                    "previous" => { let _ = rt.block_on(c.go_to(p.playerid, "previous")); }
                    "next" => { let _ = rt.block_on(c.go_to(p.playerid, "next")); }
                    _ => {}
                }
            }
        }
    }
}

fn send_input(client: &Rc<RefCell<Option<KodiClient>>>, action: InputAction) {
    if let Some(ref c) = *client.borrow() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(c.input(action));
    }
}

fn volume_change(client: &Rc<RefCell<Option<KodiClient>>>, delta: i32) {
    if let Some(ref c) = *client.borrow() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(p) = rt.block_on(c.get_application_properties()) {
            let new_vol = (p.volume + delta).clamp(0, 100);
            let _ = rt.block_on(c.set_volume(new_vol));
        }
    }
}

fn volume_mute(client: &Rc<RefCell<Option<KodiClient>>>) {
    if let Some(ref c) = *client.borrow() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Ok(p) = rt.block_on(c.get_application_properties()) {
            let _ = rt.block_on(c.set_mute(!p.muted));
        }
    }
}
