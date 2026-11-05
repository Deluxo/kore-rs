use gtk::prelude::*;

use crate::io::kodi::ClientCell;

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
    client: ClientCell,
}

impl RemoteState {
    pub fn new(client: ClientCell) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &ClientCell {
        &self.client
    }
}

pub fn create_remote(client: ClientCell) -> (gtk::Box, RemoteWidgets, RemoteState) {
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

    // D-pad with integrated nav buttons (3 columns x 3 rows)
    let dpad = gtk::Grid::new();
    dpad.set_column_homogeneous(true);
    dpad.set_row_homogeneous(true);
    dpad.set_column_spacing(8);
    dpad.set_row_spacing(8);
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
