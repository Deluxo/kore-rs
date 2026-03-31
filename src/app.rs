use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::KodiClient;
use crate::ui::host_list::{create_host_manager_dialog, connect_host_dialog_handlers, HostListState};
use crate::ui::now_playing::{create_now_playing, start_now_playing_polling};
use crate::ui::remote::{create_remote, connect_remote_handlers};

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
        
        Self {
            app,
            hosts: Vec::new(),
            host_manager: Rc::new(RefCell::new(None)),
        }
    }

    pub fn init_logging(self) -> Self {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::DEBUG.into()),
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
            if let Some(settings) = gtk::Settings::default() {
                settings.set_property("gtk-application-prefer-dark-theme", true);
            }
            
            let window = create_main_window(app);
            let header = create_header_bar(&host_manager, &hosts);
            let client = auto_connect_first_host(&hosts);
            let content = create_main_content(client.clone());

            window.set_titlebar(Some(&header));
            window.set_child(Some(&content));
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

fn create_main_window(app: &gtk::Application) -> gtk::ApplicationWindow {
    gtk::ApplicationWindow::builder()
        .application(app)
        .title("Korers")
        .default_width(600)
        .default_height(900)
        .resizable(true)
        .build()
}

fn create_header_bar(
    host_manager: &Rc<RefCell<Option<HostManager>>>,
    hosts: &[Host],
) -> gtk::HeaderBar {
    let header = gtk::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("Korers")))
        .show_title_buttons(true)
        .build();

    let hosts_btn = gtk::Button::builder()
        .icon_name("computer")
        .tooltip_text("Manage hosts")
        .build();

    let client: Rc<RefCell<Option<KodiClient>>> = Rc::new(RefCell::new(None));
    let (dialog, widgets) = create_host_manager_dialog(hosts);
    let state = HostListState::new(host_manager.clone(), client);
    connect_host_dialog_handlers(&dialog, &widgets, &state);

    let dialog = dialog.clone();
    hosts_btn.connect_clicked(move |_| {
        dialog.show();
        dialog.present();
    });
    
    header.pack_start(&hosts_btn);
    header
}

fn auto_connect_first_host(hosts: &[Host]) -> Rc<RefCell<Option<KodiClient>>> {
    let client: Rc<RefCell<Option<KodiClient>>> = Rc::new(RefCell::new(None));

    if let Some(first_host) = hosts.first() {
        let kodi_client = KodiClient::from_host(first_host);
        let rt = tokio::runtime::Runtime::new().unwrap();
        if rt.block_on(kodi_client.ping()).is_ok() {
            client.replace(Some(kodi_client));
            tracing::info!("Auto-connected to {}", first_host.name);
        }
    }

    client
}

fn create_main_content(client: Rc<RefCell<Option<KodiClient>>>) -> gtk::Box {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .build();

    let (now_playing, np_widgets, np_state) = create_now_playing(client.clone());
    start_now_playing_polling(np_widgets, np_state);
    content.append(&now_playing);

    let (remote, r_widgets, r_state) = create_remote(client.clone());
    connect_remote_handlers(&r_widgets, &r_state);
    content.append(&remote);

    content
}
