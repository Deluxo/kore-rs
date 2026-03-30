use gtk::prelude::*;
use gtk::Application;

use crate::effects::Effects;
use crate::host::{Host, manager::HostManager};

pub struct App {
    effects: Effects,
    app: Application,
    window: Option<gtk::ApplicationWindow>,
    hosts: Vec<Host>,
}

impl App {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id("org.korers.app")
            .build();

        Self {
            effects: Effects::new(),
            app,
            window: None,
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
        self.app.connect_activate(|app| {
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
