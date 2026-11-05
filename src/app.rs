use gtk::prelude::*;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::host::Host;
use crate::io::host::manager::HostManager;
use crate::io::kodi::{ClientCell, KodiClient};
use crate::io::mpris::{MprisHandle, MprisService};
use crate::ui::host_list::{create_host_manager_dialog, connect_host_dialog_handlers, HostListState};
use crate::ui::now_playing::{create_now_playing, start_local_advancement, trigger_kodi_poll, update_now_playing};
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
        } else {
            tracing::warn!("Failed to load hosts manager");
        }
        self
    }

    pub fn show_window(self) -> Self {
        let hosts = self.hosts.clone();
        let host_manager = self.host_manager.clone();
        let client = auto_connect_first_host(&hosts);

        // MPRIS is driven by the local GTK ticker: the D-Bus thread only
        // publishes snapshots and queues commands, never talks to Kodi.
        let (mpris, mpris_handle) = MprisService::new();
        mpris.start();

        self.app.connect_activate(move |app| {
            if let Some(settings) = gtk::Settings::default() {
                settings.set_property("gtk-application-prefer-dark-theme", true);
            } else {
                tracing::debug!("No GTK settings available");
            }

            let window = create_main_window(app);
            let header = create_header_bar(&host_manager, &hosts, client.clone());
            let content = create_main_content(
                client.clone(),
                &window,
                mpris_handle.clone(),
            );

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
    client: ClientCell,
) -> gtk::HeaderBar {
    let header = gtk::HeaderBar::builder()
        .title_widget(&gtk::Label::new(Some("Korers")))
        .show_title_buttons(true)
        .build();

    let hosts_btn = gtk::Button::builder()
        .icon_name("computer")
        .tooltip_text("Manage hosts")
        .build();

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

fn auto_connect_first_host(hosts: &[Host]) -> ClientCell {
    let client: ClientCell = Rc::new(RefCell::new(None));

    if let Some(first_host) = hosts.first() {
        let kodi_client = KodiClient::from_host(first_host);
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("Failed to create runtime for auto-connect: {}", e);
                return client;
            }
        };
        if rt.block_on(kodi_client.ping()).is_ok() {
            *client.borrow_mut() = Some(kodi_client);
            tracing::info!("Auto-connected to {}", first_host.name);
        } else {
            tracing::warn!("Failed to ping {} for auto-connect", first_host.name);
        }
    } else {
        tracing::debug!("No hosts available for auto-connect");
    }

    client
}

fn create_main_content(
    client: ClientCell,
    window: &gtk::ApplicationWindow,
    mpris: MprisHandle,
) -> gtk::Box {
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
                if let Ok(client) = client_seeker.try_borrow() {
                    if let Some(c) = client.as_ref() {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let _ = rt.block_on(c.seek_percentage(player_id, curr));
                    }
                }
            }
            trigger_kodi_poll(&poll_w, &poll_s, &poll_c, &poll_pv);
        }
    });

    // Start the local ticker. It advances playback, updates the UI, executes
    // queued MPRIS commands and publishes MPRIS state. No background Kodi I/O.
    let adv_pv = prev_val.clone();
    start_local_advancement(
        np_widgets.clone(),
        np_state.clone(),
        client.clone(),
        adv_pv,
        mpris,
    );

    // Initial Kodi poll to populate now-playing UI on startup
    trigger_kodi_poll(&np_widgets, &np_state, &client, &prev_val_for_initial);
    content.append(&now_playing);

    // Remote controls
    let (remote, r_widgets, r_state) = create_remote(client.clone());
    connect_remote_handlers(&r_widgets, &r_state);

    // Attach Kodi poll trigger to all remote buttons (except stop — handled separately)
    let buttons: [&gtk::Button; 15] = [
        &r_widgets.btn_prev,
        &r_widgets.btn_play,
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

    // Stop button: force-reset now-playing state regardless of Kodi's response.
    // Kodi may keep the player in GetActivePlayers with speed=0 after stop,
    // which would leave the seeker at the last position instead of 0.
    let stop_s = np_state.clone();
    let stop_w = np_widgets.clone();
    let stop_c = client.clone();
    let stop_pv = btn_pv.clone();
    r_widgets.btn_stop.connect_clicked(move |_| {
        let mut s = stop_s.borrow_mut();
        s.reset();
        *stop_pv.borrow_mut() = s.percentage();
        let mut w = stop_w.borrow_mut();
        update_now_playing(&mut *w, &mut *s, &stop_c);
    });

    // Poll Kodi once when window becomes active (e.g. user switches back to this window).
    // Uses EventControllerFocus (not connect_notify on is-active) because Rc<RefCell<...>>
    // does not implement Send+Sync required by property notification callbacks.
    // Works on both desktop (GNOME/KDE) and mobile (Phosh/phoc).
    let focus_ctrl = gtk::EventControllerFocus::new();
    let fc_w = np_widgets.clone();
    let fc_s = np_state.clone();
    let fc_c = client.clone();
    let fc_pv = btn_pv.clone();
    focus_ctrl.connect_enter(move |_| {
        trigger_kodi_poll(&fc_w, &fc_s, &fc_c, &fc_pv);
    });
    window.add_controller(focus_ctrl);

    content.append(&remote);
    content
}
