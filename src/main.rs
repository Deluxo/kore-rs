mod kodi;
mod host;
mod ui;

use gtk::prelude::*;
use gtk::Application;

pub use host::Host;
pub use host::manager::HostManager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Korers");

    let app = Application::builder()
        .application_id("org.korers.app")
        .build();
    
    app.connect_activate(|app| {
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

        let status_label = gtk::Label::builder()
            .label("Ready")
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

        let remote_label = gtk::Label::new(Some("Remote Control"));
        main_stack.add_titled(&remote_label, Some("remote"), "Remote");

        let now_playing_label = gtk::Label::new(Some("Now Playing"));
        main_stack.add_titled(&now_playing_label, Some("nowplaying"), "Now Playing");

        let settings_label = gtk::Label::new(Some("Settings"));
        main_stack.add_titled(&settings_label, Some("settings"), "Settings");

        sidebar.append(&hosts_label);
        sidebar.append(&host_list);
        sidebar.append(&button_box);
        sidebar.append(&status_label);

        button_box.append(&discover_button);
        button_box.append(&add_button);

        main_box.append(&sidebar);
        main_box.append(&sep);
        main_box.append(&main_stack);

        vbox.append(&header);
        vbox.append(&main_box);

        window.set_child(Some(&vbox));

        let mut hosts: Vec<Host> = Vec::new();
        if let Ok(manager) = HostManager::new() {
            hosts = manager.hosts().to_vec();
        }

        for host in &hosts {
            add_host_to_list(&host_list, host);
        }

        let host_list_for_discovery = host_list.clone();
        
        discover_button.clone().connect_clicked(move |_| {
            status_label.set_label("Discovering...");
            discover_button.set_sensitive(false);
            
            let discovery = kodi::DiscoveryService::new();
            match discovery.discover_all(5) {
                Ok(discovered) => {
                    tracing::info!("Discovery found {} hosts", discovered.len());
                    
                    for host_info in discovered {
                        let host = Host::new(host_info.name.clone(), host_info.address.clone(), host_info.port);
                        add_host_to_list(&host_list_for_discovery, &host);
                        
                        if let Ok(mut manager) = HostManager::new() {
                            let _ = manager.add_host(host);
                        }
                    }
                    
                    status_label.set_label("Discovery complete");
                }
                Err(e) => {
                    tracing::error!("Discovery failed: {}", e);
                    status_label.set_label("Discovery failed");
                }
            }
            discover_button.set_sensitive(true);
        });

        window.show();
    });
    
    app.run();
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
