use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::kodi::client::InputAction;
use crate::kodi::KodiClient;

pub struct RemoteWidgets {
    pub btn_prev: gtk::Button,
    pub btn_play: gtk::Button,
    pub btn_stop: gtk::Button,
    pub btn_next: gtk::Button,
    pub btn_back: gtk::Button,
    pub btn_home: gtk::Button,
    pub btn_info: gtk::Button,
    pub btn_up: gtk::Button,
    pub btn_left: gtk::Button,
    pub btn_ok: gtk::Button,
    pub btn_right: gtk::Button,
    pub btn_down: gtk::Button,
    pub btn_play_pause: gtk::Button,
    pub btn_mute: gtk::Button,
    pub btn_voldown: gtk::Button,
    pub btn_volup: gtk::Button,
}

pub struct RemoteState {
    client: Rc<RefCell<Option<KodiClient>>>,
}

impl RemoteState {
    pub fn new(client: Rc<RefCell<Option<KodiClient>>>) -> Self {
        Self { client }
    }
}

pub fn create_remote(client: Rc<RefCell<Option<KodiClient>>>) -> (gtk::Box, RemoteWidgets, RemoteState) {
    let state = RemoteState::new(client);
    
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 16);
    box_.set_margin_start(12);
    box_.set_margin_end(12);
    box_.set_margin_top(12);
    box_.set_margin_bottom(12);
    box_.set_hexpand(true);
    box_.set_vexpand(true);

    // Transport row
    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transport.set_hexpand(true);

    let btn_prev = gtk::Button::builder().label("⏮").hexpand(true).build();
    let btn_play = gtk::Button::builder().label("▶/⏸").hexpand(true).build();
    let btn_stop = gtk::Button::builder().label("⏹").hexpand(true).build();
    let btn_next = gtk::Button::builder().label("⏭").hexpand(true).build();

    transport.append(&btn_prev);
    transport.append(&btn_play);
    transport.append(&btn_stop);
    transport.append(&btn_next);
    box_.append(&transport);

    // D-pad with integrated nav buttons (5 columns x 3 rows)
    // Grid layout:
    //   [Home]   [Up]   [Info]
    //   [Back]  [OK]  [Right]
    //   [Left] [Down]
    let dpad = gtk::Grid::new();
    dpad.set_column_homogeneous(true);
    dpad.set_row_homogeneous(true);
    dpad.set_hexpand(true);
    dpad.set_vexpand(true);

    // Row 0: Home, Up, Info
    let btn_home = gtk::Button::builder().label("Home").hexpand(true).vexpand(true).build();
    let btn_up = gtk::Button::builder().label("▲").hexpand(true).vexpand(true).build();
    let btn_info = gtk::Button::builder().label("Info").hexpand(true).vexpand(true).build();

    // Row 1: Left, OK, Right
    let btn_left = gtk::Button::builder().label("◀").hexpand(true).vexpand(true).build();
    let btn_ok = gtk::Button::builder().label("OK").hexpand(true).vexpand(true).build();
    let btn_right = gtk::Button::builder().label("▶").hexpand(true).vexpand(true).build();

    // Row 2: Back, Down, Play/Pause
    let btn_back = gtk::Button::builder().label("Back").hexpand(true).vexpand(true).build();
    let btn_down = gtk::Button::builder().label("▼").hexpand(true).vexpand(true).build();
    let btn_play_pause = gtk::Button::builder().label("▶/⏸").hexpand(true).vexpand(true).build();

    // Attach to grid: column, row, width, height
    dpad.attach(&btn_home, 0, 0, 1, 1);
    dpad.attach(&btn_up,     1, 0, 1, 1);
    dpad.attach(&btn_info,  2, 0, 1, 1);
    
    dpad.attach(&btn_left,  0, 1, 1, 1);
    dpad.attach(&btn_ok,    1, 1, 1, 1);
    dpad.attach(&btn_right, 2, 1, 1, 1);
    
    dpad.attach(&btn_back,      0, 2, 1, 1);
    dpad.attach(&btn_down,       1, 2, 1, 1);
    dpad.attach(&btn_play_pause, 2, 2, 1, 1);
    
    box_.append(&dpad);

    // Volume row
    let volume = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    volume.set_hexpand(true);

    let btn_mute = gtk::Button::builder().label("🔇").hexpand(true).build();
    let btn_voldown = gtk::Button::builder().label("🔉").hexpand(true).build();
    let btn_volup = gtk::Button::builder().label("🔊").hexpand(true).build();

    volume.append(&btn_mute);
    volume.append(&btn_voldown);
    volume.append(&btn_volup);
    box_.append(&volume);

    let widgets = RemoteWidgets {
        btn_prev,
        btn_play,
        btn_stop,
        btn_next,
        btn_back,
        btn_home,
        btn_info,
        btn_up,
        btn_left,
        btn_ok,
        btn_right,
        btn_down,
        btn_play_pause,
        btn_mute,
        btn_voldown,
        btn_volup,
    };

    (box_, widgets, state)
}

pub fn connect_remote_handlers(widgets: &RemoteWidgets, state: &RemoteState) {
    let client = state.client.clone();
    widgets.btn_prev.connect_clicked(move |_| transport_action(&client, "previous"));
    
    let client = state.client.clone();
    widgets.btn_play.connect_clicked(move |_| transport_action(&client, "play_pause"));
    
    let client = state.client.clone();
    widgets.btn_stop.connect_clicked(move |_| transport_action(&client, "stop"));
    
    let client = state.client.clone();
    widgets.btn_next.connect_clicked(move |_| transport_action(&client, "next"));

    let client = state.client.clone();
    widgets.btn_back.connect_clicked(move |_| send_input(&client, InputAction::Back));
    
    let client = state.client.clone();
    widgets.btn_home.connect_clicked(move |_| send_input(&client, InputAction::Home));
    
    let client = state.client.clone();
    widgets.btn_info.connect_clicked(move |_| send_input(&client, InputAction::Info));

    let client = state.client.clone();
    widgets.btn_up.connect_clicked(move |_| send_input(&client, InputAction::Up));
    
    let client = state.client.clone();
    widgets.btn_left.connect_clicked(move |_| send_input(&client, InputAction::Left));
    
    let client = state.client.clone();
    widgets.btn_ok.connect_clicked(move |_| send_input(&client, InputAction::Select));
    
    let client = state.client.clone();
    widgets.btn_right.connect_clicked(move |_| send_input(&client, InputAction::Right));
    
    let client = state.client.clone();
    widgets.btn_down.connect_clicked(move |_| send_input(&client, InputAction::Down));

    let client = state.client.clone();
    widgets.btn_play_pause.connect_clicked(move |_| transport_action(&client, "play_pause"));

    let client = state.client.clone();
    widgets.btn_mute.connect_clicked(move |_| volume_mute(&client));
    
    let client = state.client.clone();
    widgets.btn_voldown.connect_clicked(move |_| volume_change(&client, -10));
    
    let client = state.client.clone();
    widgets.btn_volup.connect_clicked(move |_| volume_change(&client, 10));
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
                    "stop" => {
                        let _ = rt.block_on(c.stop(p.playerid));
                    }
                    "previous" => {
                        let _ = rt.block_on(c.go_to(p.playerid, "previous"));
                    }
                    "next" => {
                        let _ = rt.block_on(c.go_to(p.playerid, "next"));
                    }
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
