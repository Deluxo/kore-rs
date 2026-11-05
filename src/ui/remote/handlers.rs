use gtk::prelude::*;

use crate::io::kodi::actions::{self, TransportAction};
use crate::io::kodi::types::InputAction;
use crate::io::kodi::{with_client, ClientCell, KodiClient};

use super::layout::{RemoteState, RemoteWidgets};

pub fn connect_remote_handlers(widgets: &RemoteWidgets, state: &RemoteState) {
    let client = state.client();

    connect(&widgets.btn_prev, client, |c| {
        actions::transport(c, TransportAction::Previous)
    });
    connect(&widgets.btn_play, client, |c| {
        actions::transport(c, TransportAction::PlayPause)
    });
    connect(&widgets.btn_stop, client, |c| {
        actions::transport(c, TransportAction::Stop)
    });
    connect(&widgets.btn_next, client, |c| {
        actions::transport(c, TransportAction::Next)
    });

    connect(&widgets.btn_back, client, |c| actions::input(c, InputAction::Back));
    connect(&widgets.btn_home, client, |c| actions::input(c, InputAction::Home));
    connect(&widgets.btn_info, client, |c| actions::input(c, InputAction::Info));
    connect(&widgets.btn_up, client, |c| actions::input(c, InputAction::Up));
    connect(&widgets.btn_left, client, |c| actions::input(c, InputAction::Left));
    connect(&widgets.btn_ok, client, |c| actions::input(c, InputAction::Select));
    connect(&widgets.btn_right, client, |c| actions::input(c, InputAction::Right));
    connect(&widgets.btn_down, client, |c| actions::input(c, InputAction::Down));

    connect(&widgets.btn_play_pause, client, |c| {
        actions::transport(c, TransportAction::PlayPause)
    });
    connect(&widgets.btn_mute, client, actions::toggle_mute);
    connect(&widgets.btn_voldown, client, |c| actions::adjust_volume(c, -2));
    connect(&widgets.btn_volup, client, |c| actions::adjust_volume(c, 2));
}

fn connect<F>(button: &gtk::Button, client: &ClientCell, action: F)
where
    F: Fn(&KodiClient) + 'static,
{
    let client = client.clone();
    button.connect_clicked(move |_| {
        with_client(&client, &action);
    });
}
