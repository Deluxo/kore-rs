use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

use crate::host::Host;
use crate::host::manager::HostManager;
use crate::kodi::KodiClient;
use crate::ui::host_list::{create_host_manager_dialog, connect_host_dialog_handlers, HostListState};
use crate::ui::now_playing::{create_now_playing, start_local_advancement, trigger_kodi_poll};
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

    content.set_homogeneous(true);

    // Create now-playing component (seeker not yet connected)
    let (now_playing, np_widgets, np_state, seeker_ctx) = create_now_playing(client.clone());
    let np_widgets = Rc::new(RefCell::new(np_widgets));
    let np_state = Rc::new(RefCell::new(np_state));

    // Connect seeker: on user drag -> seek Kodi + trigger Kodi poll
    let prev_val = seeker_ctx.prev_val.clone();
    let prev_val_for_initial = prev_val.clone();
    let player_info = seeker_ctx.player_info.clone();
    let client_seeker = seeker_ctx.client.clone();
    let poll_w = np_widgets.clone();
    let poll_s = np_state.clone();
    let poll_c = client.clone();
    let poll_pv = prev_val.clone();
    seeker_ctx.adjustment.connect_value_changed(move |adj| {
        let curr = adj.value();
        let prev = *poll_pv.borrow();
        if (curr - prev).abs() > 0.5 {
            *poll_pv.borrow_mut() = curr;
            if let Some((player_id, _)) = *player_info.borrow() {
                if let Some(ref c) = *client_seeker.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.seek_percentage(player_id, curr));
                }
            }
            trigger_kodi_poll(&poll_w, &poll_s, &poll_c, &poll_pv);
        }
    });

    // Start lightweight local advancement timer (no Kodi calls)
    // Pass prev_val so the timer syncs it before set_value, avoiding re-entrancy.
    let adv_pv = prev_val.clone();
    start_local_advancement(np_widgets.clone(), np_state.clone(), client.clone(), adv_pv);

    // Initial Kodi poll to populate now-playing UI on startup
    trigger_kodi_poll(&np_widgets, &np_state, &client, &prev_val_for_initial);
    content.append(&now_playing);

    // Remote controls
    let (remote, r_widgets, r_state) = create_remote(client.clone());
    connect_remote_handlers(&r_widgets, &r_state);

    // Attach Kodi poll trigger to all remote buttons
    let buttons: [&gtk::Button; 16] = [
        &r_widgets.btn_prev,
        &r_widgets.btn_play,
        &r_widgets.btn_stop,
        &r_widgets.btn_next,
        &r_widgets.btn_back,
        &r_widgets.btn_home,
        &r_widgets.btn_info,
        &r_widgets.btn_up,
        &r_widgets.btn_left,
        &r_widgets.btn_ok,
        &r_widgets.btn_right,
        &r_widgets.btn_down,
        &r_widgets.btn_play_pause,
        &r_widgets.btn_mute,
        &r_widgets.btn_voldown,
        &r_widgets.btn_volup,
    ];
    let btn_pv = prev_val;
    for btn in buttons {
        let w = np_widgets.clone();
        let s = np_state.clone();
        let c = client.clone();
        let pv = btn_pv.clone();
        btn.connect_clicked(move |_| {
            trigger_kodi_poll(&w, &s, &c, &pv);
        });
    }

    content.append(&remote);
    content
}
