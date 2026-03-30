use gtk::prelude::*;
use gtk::Application;
use gtk::glib;
use gtk::pango;
use gtk::{Scale, GestureClick};
use std::cell::RefCell;
use std::rc::Rc;

use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::DiscoveryService;
use crate::kodi::KodiClient;
use crate::kodi::client::InputAction;

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
                .default_width(600)
                .default_height(900)
                .resizable(true)
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
            
            // Auto-connect to first host
            if let Some(first_host) = hosts.first() {
                let kodi_client = KodiClient::from_host(first_host);
                let rt = tokio::runtime::Runtime::new().unwrap();
                if rt.block_on(kodi_client.ping()).is_ok() {
                    client.replace(Some(kodi_client));
                    tracing::info!("Auto-connected to {}", first_host.name);
                }
            }
            
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

                            let _host_id = host.id.clone();
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
    box_.set_hexpand(true); box_.set_vexpand(true);

    // Poster (aspect ratio box)
    let poster = gtk::AspectFrame::new(0.5, 0.5, 0.666, false);
    poster.set_hexpand(false);
    let poster_image = gtk::Image::from_icon_name("media-video");
    poster_image.set_icon_size(gtk::IconSize::Large);
    poster.set_child(Some(&poster_image));
    box_.append(&poster);

    // Title
    let title = gtk::Label::new(Some("<big><b>No media playing</b></big>"));
    title.set_halign(gtk::Align::Center);
    title.set_use_markup(true);
    title.set_hexpand(true);
    title.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&title);

    // Description (artist / show info)
    let description = gtk::Label::new(Some(""));
    description.set_halign(gtk::Align::Center);
    description.set_hexpand(true);
    description.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&description);

    // Seeker
    let seeker_adj = gtk::Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);
    let seeker = Scale::new(gtk::Orientation::Horizontal, Some(&seeker_adj));
    seeker.set_hexpand(true);
    seeker.set_draw_value(false);
    
    // For seek - store current player info
    let player_info: Rc<RefCell<Option<(i32, i64)>>> = Rc::new(RefCell::new(None));
    let previous_poll_value = Rc::new(RefCell::new(0.0));
    let was_seeking = Rc::new(RefCell::new(false));
    
    // Track when user manually adjusts the scale
    let adjustment_clone = seeker_adj.clone();
    let prev_value_clone = previous_poll_value.clone();
    let was_seeking_clone = was_seeking.clone();
    let player_info_clone = player_info.clone();
    let client_clone = client.clone();
    let seeker_clone = seeker.clone();
    seeker_clone.connect_value_changed(move |_scale| {
        let curr = adjustment_clone.value();
        let prev = *prev_value_clone.borrow();
        
        // Only trigger seek if value changed significantly (> 1%)
        // This prevents polling from triggering seeks
        if (curr - prev).abs() > 1.0 {
            tracing::info!("Slider moved by user! {} -> {} (diff: {:.1})", prev, curr, curr - prev);
            *was_seeking_clone.borrow_mut() = true;
            if let Some((player_id, _)) = *player_info_clone.borrow() {
                if let Some(ref c) = *client_clone.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.seek_percentage(player_id, curr));
                }
            }
        }
    });
    
    // Initially set to prevent immediate seek on first poll
    *previous_poll_value.borrow_mut() = seeker_adj.value();
    
    // Visual feedback hint
    let seek_hint = gtk::Label::new(Some("👆 tap to seek"));
    seek_hint.set_halign(gtk::Align::Center);
    seek_hint.set_margin_top(4);
    seek_hint.set_hexpand(true);
    seek_hint.set_css_classes(&["secondary"]);
    
    let seek_hint_clone = seek_hint.clone();
    let was_seeking_clone2 = was_seeking.clone();
    seeker_clone.connect_value_changed(move |_scale| {
        let is_seeking = *was_seeking_clone2.borrow();
        if is_seeking {
            seek_hint_clone.set_label("✨ SEEKING... ✨");
            *was_seeking_clone2.borrow_mut() = false;
        }
    });
    
    box_.append(&seeker);
    box_.append(&seek_hint);

    // Time labels: current | remaining | ends at
    let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    time_box.set_hexpand(true);
    let time_current = gtk::Label::new(Some("0:00"));
    time_current.set_halign(gtk::Align::Start);
    time_current.set_hexpand(true);
    
    let time_remaining = gtk::Label::new(Some("-0:00"));
    time_remaining.set_halign(gtk::Align::Center);
    time_remaining.set_hexpand(true);
    
    let time_ends = gtk::Label::new(Some("ends 0:00"));
    time_ends.set_halign(gtk::Align::End);
    time_ends.set_hexpand(true);
    
    time_box.append(&time_current);
    time_box.append(&time_remaining);
    time_box.append(&time_ends);
    box_.append(&time_box);

    // Polling
    let title_clone = title.clone();
    let desc_clone = description.clone();
    let seeker_clone = seeker.clone();
    let time_cur_clone = time_current.clone();
    let time_rem_clone = time_remaining.clone();
    let time_ends_clone = time_ends.clone();
    let player_info_poll = player_info.clone();
    let client_poll = client.clone();
    let seek_hint_clone = seek_hint.clone();
    glib::source::timeout_add_seconds_local(2, move || {
        let title2 = title_clone.clone();
        let desc2 = desc_clone.clone();
        let seeker2 = seeker_clone.clone();
        let time_cur2 = time_cur_clone.clone();
        let time_rem2 = time_rem_clone.clone();
        let time_ends2 = time_ends_clone.clone();
        
        if let Some(ref c) = *client_poll.borrow() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let players = rt.block_on(c.get_active_players());
            if let Ok(players) = players {
                if let Some(p) = players.first() {
                    tracing::debug!("  player_id: {}", p.playerid);
                    let item = rt.block_on(c.get_current_item(p.playerid));
                    let props = rt.block_on(c.get_player_properties(p.playerid));
                    
                    // Store player info for seek
                    if let Ok(ref props_ok) = props {
                        let dur = props_ok.totaltime.to_seconds();
                        player_info_poll.replace(Some((p.playerid, dur)));
                    }
                    
                    if let Ok(item) = item {
                        let label = item.title.clone()
                            .or(item.label.clone())
                            .or(item.file.clone())
                            .unwrap_or_else(|| "Playing".to_string());
                        
                        let mut desc_parts = Vec::new();
                        if let Some(artist) = item.artist {
                            desc_parts.push(artist.join(", "));
                        }
                        if let Some(album) = item.album {
                            desc_parts.push(album);
                        }
                        if let Some(show) = item.showtitle {
                            if let (Some(season), Some(episode)) = (item.season, item.episode) {
                                desc_parts.push(format!("S{:02}E{:02}", season, episode));
                            }
                            desc_parts.push(show);
                        }
                        let desc = desc_parts.join(" • ");
                        
                        let (cur_time, duration, percent): (i64, i64, f64) = {
                            match props {
                                Ok(p) => {
                                    let ct = p.time.to_seconds();
                                    let dur = p.totaltime.to_seconds();
                                    let pct = if dur > 0 { (ct as f64 / dur as f64) * 100.0 } else { 50.0 };
                                    (ct, dur, pct.clamp(0.0, 100.0))
                                }
                                Err(e) => {
                                    eprintln!("SEEKER: props error: {:?}", e);
                                    (0, 0, 50.0)
                                }
                            }
                        };
                        
                        tracing::debug!("NOW PLAYING: label={}, time={}/{}", label, cur_time, duration);
                        
                        let t2 = title2.clone();
                         let d2 = desc2.clone();
                         let s2 = seeker2.clone();
                         let tc2 = time_cur2.clone();
                         let tr2 = time_rem2.clone();
                         let te2 = time_ends2.clone();
                         let sh2 = seek_hint_clone.clone();
                         glib::idle_add_local_once(move || {
                             t2.set_markup(&format!("<big><b>{}</b></big>", label));
                             d2.set_text(&desc);
                             s2.set_value(percent);
                             sh2.set_label("👆 tap to seek");
                             tc2.set_text(&format_time(cur_time));
                             tr2.set_text(&format!("-{}", format_time(duration - cur_time)));
                             // Calculate end time as clock time
                             let end_time = chrono::Local::now() + chrono::Duration::seconds((duration - cur_time) as i64);
                             te2.set_text(&format!("ends {}", end_time.format("%H:%M")));
                         });
                         return glib::ControlFlow::Continue;
                    }
                }
            }
        }
        let t2 = title2.clone();
        let d2 = desc2.clone();
        glib::idle_add_local_once(move || {
            t2.set_markup("<big><b>No media playing</b></big>");
            d2.set_text("");
        });
        glib::ControlFlow::Continue
    });

    box_
}

fn format_time(secs: i64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, s)
    } else {
        format!("{}:{:02}", mins, s)
    }
}

fn create_remote_box(client: Rc<RefCell<Option<KodiClient>>>) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 16);
    box_.set_margin_start(12); box_.set_margin_end(12); box_.set_margin_top(12); box_.set_margin_bottom(12);
    box_.set_hexpand(true); box_.set_vexpand(true);

    // Transport controls
    let transport = gtk::Grid::new();
    transport.set_column_homogeneous(true);
    transport.set_row_homogeneous(true);
    transport.set_hexpand(true);

    let prev = gtk::Button::builder().label("⏮").hexpand(true).build();
    let play = gtk::Button::builder().label("▶/⏸").hexpand(true).build();
    let stop = gtk::Button::builder().label("⏹").hexpand(true).build();
    let next = gtk::Button::builder().label("⏭").hexpand(true).build();

    let c = client.clone();
    prev.connect_clicked(move |_| transport_action(&c, "previous"));
    let c = client.clone();
    play.connect_clicked(move |_| transport_action(&c, "play_pause"));
    let c = client.clone();
    stop.connect_clicked(move |_| transport_action(&c, "stop"));
    let c = client.clone();
    next.connect_clicked(move |_| transport_action(&c, "next"));

    transport.attach(&prev, 0, 0, 1, 1);
    transport.attach(&play, 1, 0, 1, 1);
    transport.attach(&stop, 2, 0, 1, 1);
    transport.attach(&next, 3, 0, 1, 1);
    box_.append(&transport);

    // Nav buttons - top row
    let nav = gtk::Grid::new();
    nav.set_column_homogeneous(true);
    nav.set_row_homogeneous(true);
    nav.set_hexpand(true);
    let back = gtk::Button::builder().label("Back").hexpand(true).build();
    let home = gtk::Button::builder().label("Home").hexpand(true).build();
    let info = gtk::Button::builder().label("Info").hexpand(true).build();
    let c = client.clone();
    back.connect_clicked(move |_| send_input(&c, InputAction::Back));
    let c = client.clone();
    home.connect_clicked(move |_| send_input(&c, InputAction::Home));
    let c = client.clone();
    info.connect_clicked(move |_| send_input(&c, InputAction::Info));
    nav.attach(&back, 0, 0, 1, 1);
    nav.attach(&home, 1, 0, 1, 1);
    nav.attach(&info, 2, 0, 1, 1);
    box_.append(&nav);

    // D-pad - 3x3 grid
    let dpad = gtk::Grid::new();
    dpad.set_column_homogeneous(true);
    dpad.set_row_homogeneous(true);
    dpad.set_hexpand(true);
    dpad.set_vexpand(true);
    let up = gtk::Button::builder().label("▲").hexpand(true).vexpand(true).build();
    let left = gtk::Button::builder().label("◀").hexpand(true).vexpand(true).build();
    let ok = gtk::Button::builder().label("OK").hexpand(true).vexpand(true).build();
    let right = gtk::Button::builder().label("▶").hexpand(true).vexpand(true).build();
    let down = gtk::Button::builder().label("▼").hexpand(true).vexpand(true).build();
    
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

    dpad.attach(&up, 1, 0, 1, 1);
    dpad.attach(&left, 0, 1, 1, 1);
    dpad.attach(&ok, 1, 1, 1, 1);
    dpad.attach(&right, 2, 1, 1, 1);
    dpad.attach(&down, 1, 2, 1, 1);
    box_.append(&dpad);

    // Volume - 3 buttons
    let volume = gtk::Grid::new();
    volume.set_column_homogeneous(true);
    volume.set_row_homogeneous(true);
    volume.set_hexpand(true);
    let mute = gtk::Button::builder().label("🔇").hexpand(true).build();
    let v_down = gtk::Button::builder().label("🔉").hexpand(true).build();
    let v_up = gtk::Button::builder().label("🔊").hexpand(true).build();

    let c = client.clone();
    mute.connect_clicked(move |_| volume_mute(&c));
    let c = client.clone();
    v_down.connect_clicked(move |_| volume_change(&c, -10));
    let c = client.clone();
    v_up.connect_clicked(move |_| volume_change(&c, 10));

    volume.attach(&mute, 0, 0, 1, 1);
    volume.attach(&v_down, 1, 0, 1, 1);
    volume.attach(&v_up, 2, 0, 1, 1);
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
                    "play_pause" => {
                        tracing::debug!("play_pause: player_id={}", p.playerid);
                        match rt.block_on(c.play_pause(p.playerid)) {
                            Ok(speed) => tracing::debug!("  speed={}", speed),
                            Err(e) => tracing::warn!("  play_pause error: {:?}", e),
                        }
                    }
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
