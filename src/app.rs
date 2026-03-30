use gtk::prelude::*;
use gtk::Application;

use std::cell::RefCell;
use std::rc::Rc;

use crate::host::{Host, manager::HostManager};
use crate::kodi::discovery::DiscoveryService;
use crate::kodi::client::InputAction;
use crate::kodi::KodiClient;

pub struct App {
    app: Application,
    hosts: Vec<Host>,
}

impl App {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id("org.korers.app")
            .build();

        Self {
            app,
            hosts: Vec::new(),
        }
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
        if let Ok(manager) = HostManager::new() {
            self.hosts = manager.hosts().to_vec();
        }
        self
    }

    pub fn show_window(self) -> Self {
        let hosts = self.hosts.clone();
        
        self.app.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::builder()
                .application(app)
                .title("Korers - Kodi Remote")
                .default_width(800)
                .default_height(600)
                .build();

            let vbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

            let header = gtk::HeaderBar::builder()
                .title_widget(&gtk::Label::new(Some("Korers")))
                .show_title_buttons(true)
                .build();

            let main_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .hexpand(true)
                .vexpand(true)
                .build();

            let sidebar = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .width_request(250)
                .build();

            let hosts_label = gtk::Label::builder()
                .label("Hosts")
                .halign(gtk::Align::Start)
                .margin_start(12)
                .margin_top(12)
                .margin_bottom(12)
                .build();

            let host_list = gtk::ListBox::builder()
                .vexpand(true)
                .margin_start(8)
                .margin_end(8)
                .build();

            for host in &hosts {
                add_host_to_list(&host_list, host);
            }

            let button_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .margin_start(8)
                .margin_end(8)
                .margin_top(8)
                .margin_bottom(8)
                .build();

            let discover_button = gtk::Button::builder()
                .label("Discover")
                .build();

            let add_button = gtk::Button::builder()
                .label("Add")
                .hexpand(true)
                .build();

            let edit_button = gtk::Button::builder()
                .label("Edit")
                .build();

            let delete_button = gtk::Button::builder()
                .label("Delete")
                .build();

            let status_label = gtk::Label::builder()
                .label("Ready")
                .halign(gtk::Align::Start)
                .margin_start(8)
                .margin_end(8)
                .build();

            let connection_status = gtk::Label::builder()
                .label("Disconnected")
                .halign(gtk::Align::Start)
                .margin_start(8)
                .margin_end(8)
                .margin_bottom(8)
                .build();

            let sep = gtk::Separator::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();

            let main_stack = gtk::Stack::builder()
                .vexpand(true)
                .hexpand(true)
                .build();

            let welcome_label = gtk::Label::new(Some("Select a host to get started"));
            main_stack.add_titled(&welcome_label, Some("welcome"), "Welcome");

            // Build Remote Control UI
            let remote_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(16)
                .hexpand(true)
                .vexpand(true)
                .build();

            // D-pad section
            let dpad_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(4)
                .build();

            let dpad_up = gtk::Button::builder()
                .label("▲")
                .build();
            let dpad_row = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(4)
                .build();
            let dpad_left = gtk::Button::builder()
                .label("◀")
                .build();
            let dpad_select = gtk::Button::builder()
                .label("OK")
                .build();
            let dpad_right = gtk::Button::builder()
                .label("▶")
                .build();
            let dpad_down = gtk::Button::builder()
                .label("▼")
                .build();

            dpad_row.append(&dpad_left);
            dpad_row.append(&dpad_select);
            dpad_row.append(&dpad_right);
            dpad_box.append(&dpad_up);
            dpad_box.append(&dpad_row);
            dpad_box.append(&dpad_down);

            // Navigation buttons
            let nav_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .build();
            let back_btn = gtk::Button::builder()
                .label("Back")
                .build();
            let home_btn = gtk::Button::builder()
                .label("Home")
                .build();
            let info_btn = gtk::Button::builder()
                .label("Info")
                .build();
            nav_box.append(&back_btn);
            nav_box.append(&home_btn);
            nav_box.append(&info_btn);

            // Transport controls
            let transport_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .build();
            let prev_btn = gtk::Button::builder()
                .label("⏮")
                .build();
            let play_btn = gtk::Button::builder()
                .label("▶/⏸")
                .build();
            let stop_btn = gtk::Button::builder()
                .label("⏹")
                .build();
            let next_btn = gtk::Button::builder()
                .label("⏭")
                .build();
            transport_box.append(&prev_btn);
            transport_box.append(&play_btn);
            transport_box.append(&stop_btn);
            transport_box.append(&next_btn);

            // Volume controls
            let volume_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .build();
            let mute_btn = gtk::Button::builder()
                .label("🔇")
                .build();
            let vol_down_btn = gtk::Button::builder()
                .label("🔉")
                .build();
            let vol_label = gtk::Label::new(Some("Volume"));
            let vol_up_btn = gtk::Button::builder()
                .label("🔊")
                .build();
            volume_box.append(&mute_btn);
            volume_box.append(&vol_down_btn);
            volume_box.append(&vol_label);
            volume_box.append(&vol_up_btn);

            remote_box.append(&dpad_box);
            remote_box.append(&nav_box);
            remote_box.append(&transport_box);
            remote_box.append(&volume_box);

            main_stack.add_titled(&remote_box, Some("remote"), "Remote");

            // Store for connected client
            let client: Rc<RefCell<Option<KodiClient>>> = Rc::new(RefCell::new(None));

            // Wire up D-pad buttons
            let client_for_dpad = client.clone();
            dpad_up.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_dpad.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Up));
                }
            });

            let client_for_dpad = client.clone();
            dpad_down.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_dpad.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Down));
                }
            });

            let client_for_dpad = client.clone();
            dpad_left.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_dpad.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Left));
                }
            });

            let client_for_dpad = client.clone();
            dpad_right.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_dpad.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Right));
                }
            });

            let client_for_dpad = client.clone();
            dpad_select.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_dpad.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Select));
                }
            });

            // Navigation buttons
            let client_for_nav = client.clone();
            back_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_nav.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Back));
                }
            });

            let client_for_nav = client.clone();
            home_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_nav.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Home));
                }
            });

            let client_for_nav = client.clone();
            info_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_nav.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.input(InputAction::Info));
                }
            });

            // Transport controls
            let client_for_transport = client.clone();
            play_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_transport.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.play_pause(player.playerid));
                        }
                    }
                }
            });

            let client_for_transport = client.clone();
            stop_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_transport.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.stop(player.playerid));
                        }
                    }
                }
            });

            let client_for_transport = client.clone();
            prev_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_transport.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.go_to(player.playerid, "previous"));
                        }
                    }
                }
            });

            let client_for_transport = client.clone();
            next_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_transport.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.go_to(player.playerid, "next"));
                        }
                    }
                }
            });

            // Volume controls
            let client_for_vol = client.clone();
            vol_up_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_vol.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(props) = rt.block_on(c.get_application_properties()) {
                        let new_vol = (props.volume + 10).min(100);
                        let _ = rt.block_on(c.set_volume(new_vol));
                    }
                }
            });

            let client_for_vol = client.clone();
            vol_down_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_vol.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(props) = rt.block_on(c.get_application_properties()) {
                        let new_vol = (props.volume - 10).max(0);
                        let _ = rt.block_on(c.set_volume(new_vol));
                    }
                }
            });

            let client_for_vol = client.clone();
            mute_btn.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_vol.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(props) = rt.block_on(c.get_application_properties()) {
                        let _ = rt.block_on(c.set_mute(!props.muted));
                    }
                }
            });

            // Build Now Playing UI
            let nowplaying_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(16)
                .hexpand(true)
                .vexpand(true)
                .build();

            let np_thumbnail = gtk::Image::from_icon_name("audio-x-generic");

            let np_title = gtk::Label::new(Some("No media playing"));
            np_title.set_markup("<big><b>No media playing</b></big>");
            np_title.set_halign(gtk::Align::Center);

            let np_artist = gtk::Label::new(Some(""));
            np_artist.set_halign(gtk::Align::Center);

            let np_album = gtk::Label::new(Some(""));
            np_album.set_halign(gtk::Align::Center);

            let np_progress = gtk::Scale::builder()
                .orientation(gtk::Orientation::Horizontal)
                .hexpand(true)
                .build();
            np_progress.set_range(0.0, 100.0);
            np_progress.set_value(0.0);

            let np_time = gtk::Label::new(Some("0:00 / 0:00"));
            np_time.set_halign(gtk::Align::Center);

            // Transport controls for Now Playing
            let np_transport = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(16)
                .build();
            let np_prev = gtk::Button::builder().label("⏮").build();
            let np_play = gtk::Button::builder().label("▶/⏸").build();
            let np_stop = gtk::Button::builder().label("⏹").build();
            let np_next = gtk::Button::builder().label("⏭").build();
            np_transport.append(&np_prev);
            np_transport.append(&np_play);
            np_transport.append(&np_stop);
            np_transport.append(&np_next);

            nowplaying_box.append(&np_thumbnail);
            nowplaying_box.append(&np_title);
            nowplaying_box.append(&np_artist);
            nowplaying_box.append(&np_album);
            nowplaying_box.append(&np_progress);
            nowplaying_box.append(&np_time);
            nowplaying_box.append(&np_transport);

            main_stack.add_titled(&nowplaying_box, Some("nowplaying"), "Now Playing");

            // Wire up Now Playing transport buttons
            let client_for_np = client.clone();
            np_play.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_np.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.play_pause(player.playerid));
                        }
                    }
                }
            });

            let client_for_np = client.clone();
            np_stop.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_np.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.stop(player.playerid));
                        }
                    }
                }
            });

            let client_for_np = client.clone();
            np_prev.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_np.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.go_to(player.playerid, "previous"));
                        }
                    }
                }
            });

            let client_for_np = client.clone();
            np_next.connect_clicked(move |_| {
                if let Some(ref c) = *client_for_np.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    if let Ok(players) = rt.block_on(c.get_active_players()) {
                        if let Some(player) = players.first() {
                            let _ = rt.block_on(c.go_to(player.playerid, "next"));
                        }
                    }
                }
            });

            let settings_label = gtk::Label::new(Some("Settings"));
            main_stack.add_titled(&settings_label, Some("settings"), "Settings");

            // Host selection handler - auto-connect when host is clicked
            let hosts_for_selection = hosts.clone();
            let connection_status_clone = connection_status.clone();
            let main_stack_clone = main_stack.clone();
            let client_clone = client.clone();
            
            host_list.connect_row_selected(move |_list, row| {
                if let Some(row) = row {
                    let index = row.index() as usize;
                    if let Some(host) = hosts_for_selection.get(index) {
                        connection_status_clone.set_label("Connecting...");
                        
                        tracing::info!("Connecting to host: {} at {}:{} with username: {:?}", 
                            host.name, host.address, host.port, host.username);
                        let new_client = KodiClient::from_host(host);
                        
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        match rt.block_on(new_client.ping()) {
                            Ok(response) => {
                                tracing::info!("Connected to {}: {}", host.name, response);
                                connection_status_clone.set_label(&format!("Connected to {}", host.name));
                                *client_clone.borrow_mut() = Some(new_client);
                                main_stack_clone.set_visible_child_name("remote");
                            }
                            Err(e) => {
                                tracing::error!("Connection failed: {:?}", e);
                                let error_detail = format!("{:?}", e);
                                connection_status_clone.set_label(&format!("Error: {}", e));
                                let debug_info = format!(
                                    "Failed to connect to {}\n\n\
                                    Address: {}:{}\n\n\
                                    Error: {}\n\n\
                                    Possible causes:\n\
                                    - Wrong IP address or port\n\
                                    - Firewall blocking the connection\n\
                                    - Kodi not running or not enabled for HTTP control",
                                    host.name, host.address, host.port, error_detail
                                );
                                show_error_dialog(&debug_info);
                            }
                        }
                    }
                }
            });

            sidebar.append(&hosts_label);
            sidebar.append(&host_list);
            sidebar.append(&button_box);
            sidebar.append(&connection_status);
            sidebar.append(&status_label);

            button_box.append(&discover_button);
            button_box.append(&add_button);
            button_box.append(&edit_button);
            button_box.append(&delete_button);

            main_box.append(&sidebar);
            main_box.append(&sep);
            main_box.append(&main_stack);

            vbox.append(&header);
            vbox.append(&main_box);

            window.set_child(Some(&vbox));

            let host_list_for_discovery = host_list.clone();
            let status_label_for_discovery = status_label.clone();
            let status_label_for_add = status_label.clone();
            
            discover_button.connect_clicked(move |btn| {
                let host_list = host_list_for_discovery.clone();
                
                status_label_for_discovery.set_label("Discovering...");
                btn.set_sensitive(false);
                
                let discovery = DiscoveryService::new();
                match discovery.discover_all(5) {
                    Ok(discovered) => {
                        tracing::info!("Discovery found {} hosts", discovered.len());
                        
                        for host_info in &discovered {
                            let host = Host::new(
                                host_info.name.clone(),
                                host_info.address.clone(),
                                host_info.port,
                            );
                            
                            if let Ok(mut manager) = HostManager::new() {
                                let _ = manager.add_host(host.clone());
                            }
                            
                            add_host_to_list(&host_list, &host);
                        }
                        
                        let host_count = discovered.len();
                        let status = if host_count > 0 {
                            format!("Found {} hosts", host_count)
                        } else {
                            "No hosts found".to_string()
                        };
                        status_label_for_discovery.set_label(&status);
                    }
                    Err(e) => {
                        tracing::error!("Discovery failed: {}", e);
                        status_label_for_discovery.set_label(&format!("Error: {}", e));
                    }
                }
                btn.set_sensitive(true);
            });

            let host_list_for_add = host_list.clone();
            
            add_button.connect_clicked(move |_| {
                if let Some((dialog, name_entry, address_entry, port_spin, username_entry, password_entry)) = show_add_host_dialog() {
                    let host_list = host_list_for_add.clone();
                    let status_label = status_label_for_add.clone();
                    
                    dialog.connect_response(move |dialog, response| {
                        if response == gtk::ResponseType::Ok {
                            let name = name_entry.text().to_string();
                            let address = address_entry.text().to_string();
                            let port = port_spin.value() as u16;
                            let username = username_entry.text().to_string();
                            let password = password_entry.text().to_string();
                            
                            if !name.is_empty() && !address.is_empty() {
                                let host = Host::new_with_credentials(
                                    name.clone(),
                                    address,
                                    port,
                                    if username.is_empty() { None } else { Some(username) },
                                    if password.is_empty() { None } else { Some(password) },
                                );
                                
                                add_host_to_list(&host_list, &host);
                                status_label.set_label(&format!("Added {}", name));
                                
                                if let Ok(mut manager) = HostManager::new() {
                                    let _ = manager.add_host(host);
                                }
                            }
                        }
                        dialog.destroy();
                    });
                    
                    dialog.show();
                }
            });

            // Edit button - edit selected host
            let hosts_for_edit = hosts.clone();
            let host_list_for_edit = host_list.clone();
            let status_label_for_edit = status_label.clone();
            
            edit_button.connect_clicked(move |_| {
                // Get selected row
                if let Some(selected_row) = host_list_for_edit.selected_row() {
                    let index = selected_row.index() as usize;
                    if let Some(host) = hosts_for_edit.get(index) {
                        if let Some((dialog, name_entry, address_entry, port_spin, username_entry, password_entry)) = show_edit_host_dialog(host) {
                            let host_list = host_list_for_edit.clone();
                            let status_label = status_label_for_edit.clone();
                            let original_host = host.clone();
                            
                            dialog.connect_response(move |dialog, response| {
                                if response == gtk::ResponseType::Ok {
                                    let name = name_entry.text().to_string();
                                    let address = address_entry.text().to_string();
                                    let port = port_spin.value() as u16;
                                    let username = username_entry.text().to_string();
                                    let password = password_entry.text().to_string();
                                    
                                    if !name.is_empty() && !address.is_empty() {
                                        let updated_host = Host::new_with_credentials(
                                            name.clone(),
                                            address,
                                            port,
                                            if username.is_empty() { None } else { Some(username) },
                                            if password.is_empty() { None } else { Some(password) },
                                        );
                                        
                                        // Update in manager
                                        if let Ok(mut manager) = HostManager::new() {
                                            let _ = manager.remove_host(&original_host.id);
                                            let _ = manager.add_host(updated_host.clone());
                                        }
                                        
                                        // Refresh list - rebuild all hosts
                                        while let Some(child) = host_list.first_child() {
                                            host_list.remove(&child);
                                        }
                                        if let Ok(manager) = HostManager::new() {
                                            for h in manager.hosts() {
                                                add_host_to_list(&host_list, h);
                                            }
                                        }
                                        
                                        status_label.set_label(&format!("Updated {}", name));
                                    }
                                }
                                dialog.destroy();
                            });
                            
                            dialog.show();
                        }
                    }
                } else {
                    show_error_dialog("Please select a host to edit.");
                }
            });

            // Delete button - delete selected host
            let host_list_for_delete = host_list.clone();
            let status_label_for_delete = status_label.clone();
            
            delete_button.connect_clicked(move |_| {
                if let Some(selected_row) = host_list_for_delete.selected_row() {
                    let index = selected_row.index() as usize;
                    
                    // Get fresh list of hosts from manager
                    if let Ok(manager) = HostManager::new() {
                        let hosts = manager.hosts().to_vec();
                        if let Some(host) = hosts.get(index) {
                            let name = host.name.clone();
                            let id = host.id.clone();
                            
                            // Remove from manager
                            if let Ok(mut mgr) = HostManager::new() {
                                match mgr.remove_host(&id) {
                                    Ok(()) => {
                                        // Refresh list
                                        while let Some(child) = host_list_for_delete.first_child() {
                                            host_list_for_delete.remove(&child);
                                        }
                                        if let Ok(manager) = HostManager::new() {
                                            for h in manager.hosts() {
                                                add_host_to_list(&host_list_for_delete, h);
                                            }
                                        }
                                        status_label_for_delete.set_label(&format!("Deleted {}", name));
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to delete host: {}", e);
                                        status_label_for_delete.set_label(&format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    show_error_dialog("Please select a host to delete.");
                }
            });

            window.show();
        });
        self
    }

    pub fn show_host_selection(self) -> Self {
        self
    }

    pub fn run(self) {
        self.app.run();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn add_host_to_list(list: &gtk::ListBox, host: &Host) {
    let row = gtk::ListBoxRow::new();
    
    let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    box_.set_margin_start(12);
    box_.set_margin_end(12);
    box_.set_margin_top(8);
    box_.set_margin_bottom(8);

    let icon = gtk::Image::from_icon_name("computer");
    box_.append(&icon);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    
    let name_label = gtk::Label::new(Some(&host.name));
    name_label.set_halign(gtk::Align::Start);
    name_label.set_hexpand(true);
    vbox.append(&name_label);

    let addr_label = gtk::Label::new(Some(&format!("{}:{}", host.address, host.port)));
    addr_label.set_halign(gtk::Align::Start);
    vbox.append(&addr_label);

    box_.append(&vbox);

    row.set_child(Some(&box_));
    list.append(&row);
}

fn show_add_host_dialog() -> Option<(gtk::Dialog, gtk::Entry, gtk::Entry, gtk::SpinButton, gtk::Entry, gtk::Entry)> {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Add Host"));
    dialog.set_modal(true);
    dialog.set_default_size(350, 280);

    let content = dialog.content_area();
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_spacing(12);

    let form = gtk::Grid::new();
    form.set_row_spacing(8);
    form.set_column_spacing(8);

    let name_label = gtk::Label::new(Some("Name:"));
    name_label.set_halign(gtk::Align::End);
    let name_entry = gtk::Entry::new();
    name_entry.set_placeholder_text(Some("Kodi @ 192.168.1.100"));

    let address_label = gtk::Label::new(Some("Address:"));
    address_label.set_halign(gtk::Align::End);
    let address_entry = gtk::Entry::new();
    address_entry.set_placeholder_text(Some("192.168.1.100"));

    let port_label = gtk::Label::new(Some("Port:"));
    port_label.set_halign(gtk::Align::End);
    let port_adjustment = gtk::Adjustment::new(8080.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
    let port_spin = gtk::SpinButton::new(Some(&port_adjustment), 1.0, 0);

    let username_label = gtk::Label::new(Some("Username:"));
    username_label.set_halign(gtk::Align::End);
    let username_entry = gtk::Entry::new();
    username_entry.set_placeholder_text(Some("kodi (optional)"));

    let password_label = gtk::Label::new(Some("Password:"));
    password_label.set_halign(gtk::Align::End);
    let password_entry = gtk::Entry::new();
    password_entry.set_placeholder_text(Some("password (optional)"));
    password_entry.set_visibility(false);

    form.attach(&name_label, 0, 0, 1, 1);
    form.attach(&name_entry, 1, 0, 1, 1);
    form.attach(&address_label, 0, 1, 1, 1);
    form.attach(&address_entry, 1, 1, 1, 1);
    form.attach(&port_label, 0, 2, 1, 1);
    form.attach(&port_spin, 1, 2, 1, 1);
    form.attach(&username_label, 0, 3, 1, 1);
    form.attach(&username_entry, 1, 3, 1, 1);
    form.attach(&password_label, 0, 4, 1, 1);
    form.attach(&password_entry, 1, 4, 1, 1);

    content.append(&form);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Add", gtk::ResponseType::Ok);

    Some((dialog, name_entry, address_entry, port_spin, username_entry, password_entry))
}

fn show_edit_host_dialog(host: &Host) -> Option<(gtk::Dialog, gtk::Entry, gtk::Entry, gtk::SpinButton, gtk::Entry, gtk::Entry)> {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Edit Host"));
    dialog.set_modal(true);
    dialog.set_default_size(350, 280);

    let content = dialog.content_area();
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_spacing(12);

    let form = gtk::Grid::new();
    form.set_row_spacing(8);
    form.set_column_spacing(8);

    let name_label = gtk::Label::new(Some("Name:"));
    name_label.set_halign(gtk::Align::End);
    let name_entry = gtk::Entry::new();
    name_entry.set_text(&host.name);

    let address_label = gtk::Label::new(Some("Address:"));
    address_label.set_halign(gtk::Align::End);
    let address_entry = gtk::Entry::new();
    address_entry.set_text(&host.address);

    let port_label = gtk::Label::new(Some("Port:"));
    port_label.set_halign(gtk::Align::End);
    let port_adjustment = gtk::Adjustment::new(host.port as f64, 1.0, 65535.0, 1.0, 10.0, 0.0);
    let port_spin = gtk::SpinButton::new(Some(&port_adjustment), 1.0, 0);

    let username_label = gtk::Label::new(Some("Username:"));
    username_label.set_halign(gtk::Align::End);
    let username_entry = gtk::Entry::new();
    if let Some(ref user) = host.username {
        username_entry.set_text(user);
    }
    username_entry.set_placeholder_text(Some("kodi (optional)"));

    let password_label = gtk::Label::new(Some("Password:"));
    password_label.set_halign(gtk::Align::End);
    let password_entry = gtk::Entry::new();
    if let Some(ref pass) = host.password {
        password_entry.set_text(pass);
    }
    password_entry.set_placeholder_text(Some("password (optional)"));
    password_entry.set_visibility(false);

    form.attach(&name_label, 0, 0, 1, 1);
    form.attach(&name_entry, 1, 0, 1, 1);
    form.attach(&address_label, 0, 1, 1, 1);
    form.attach(&address_entry, 1, 1, 1, 1);
    form.attach(&port_label, 0, 2, 1, 1);
    form.attach(&port_spin, 1, 2, 1, 1);
    form.attach(&username_label, 0, 3, 1, 1);
    form.attach(&username_entry, 1, 3, 1, 1);
    form.attach(&password_label, 0, 4, 1, 1);
    form.attach(&password_entry, 1, 4, 1, 1);

    content.append(&form);

    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Ok);

    Some((dialog, name_entry, address_entry, port_spin, username_entry, password_entry))
}

fn show_error_dialog(message: &str) {
    let dialog = gtk::Dialog::new();
    dialog.set_title(Some("Connection Error"));
    dialog.set_modal(true);
    dialog.set_default_size(400, 150);

    let content = dialog.content_area();
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    let label = gtk::Label::new(Some(message));
    label.set_wrap(true);
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Start);
    content.append(&label);

    dialog.add_button("OK", gtk::ResponseType::Ok);

    dialog.connect_response(|dialog, _| {
        dialog.destroy();
    });

    dialog.show();
}
