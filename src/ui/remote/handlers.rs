use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::kodi::types::InputAction;
use crate::io::kodi::KodiClient;

use super::layout::{RemoteState, RemoteWidgets};

pub fn connect_remote_handlers(widgets: &RemoteWidgets, state: &RemoteState) {
    let client = state.client().clone();
    widgets.btn_prev.connect_clicked(move |_| transport_action(&client, "previous"));

    let client = state.client().clone();
    widgets.btn_play.connect_clicked(move |_| transport_action(&client, "play_pause"));

    let client = state.client().clone();
    widgets.btn_stop.connect_clicked(move |_| transport_action(&client, "stop"));

    let client = state.client().clone();
    widgets.btn_next.connect_clicked(move |_| transport_action(&client, "next"));

    let client = state.client().clone();
    widgets.btn_back.connect_clicked(move |_| send_input(&client, InputAction::Back));

    let client = state.client().clone();
    widgets.btn_home.connect_clicked(move |_| send_input(&client, InputAction::Home));

    let client = state.client().clone();
    widgets.btn_info.connect_clicked(move |_| send_input(&client, InputAction::Info));

    let client = state.client().clone();
    widgets.btn_up.connect_clicked(move |_| send_input(&client, InputAction::Up));

    let client = state.client().clone();
    widgets.btn_left.connect_clicked(move |_| send_input(&client, InputAction::Left));

    let client = state.client().clone();
    widgets.btn_ok.connect_clicked(move |_| send_input(&client, InputAction::Select));

    let client = state.client().clone();
    widgets.btn_right.connect_clicked(move |_| send_input(&client, InputAction::Right));

    let client = state.client().clone();
    widgets.btn_down.connect_clicked(move |_| send_input(&client, InputAction::Down));

    let client = state.client().clone();
    widgets.btn_play_pause.connect_clicked(move |_| transport_action(&client, "play_pause"));

    let client = state.client().clone();
    widgets.btn_mute.connect_clicked(move |_| volume_mute(&client));

    let client = state.client().clone();
    widgets.btn_voldown.connect_clicked(move |_| volume_change(&client, -2));

    let client = state.client().clone();
    widgets.btn_volup.connect_clicked(move |_| volume_change(&client, 2));
}

fn transport_action(client: &Rc<RefCell<Option<KodiClient>>>, action: &str) {
    let binding = client.borrow();
    let c = if let Some(ref c) = *binding {
        c
    } else {
        tracing::debug!("No Kodi client for transport action");
        return;
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            return;
        }
    };
    let players = rt.block_on(c.get_active_players());
    if let Ok(players) = players {
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
                    let target_id = p.playerid;
                    let _ = rt.block_on(c.stop(target_id));
                    let deadline = std::time::Instant::now()
                        + std::time::Duration::from_millis(1000);
                    while std::time::Instant::now() < deadline {
                        if let Ok(players) = rt.block_on(c.get_active_players()) {
                            if !players.iter().any(|ap| ap.playerid == target_id) {
                                break;
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                }
                "previous" => {
                    let _ = rt.block_on(c.go_to(p.playerid, "previous"));
                }
                "next" => {
                    let _ = rt.block_on(c.go_to(p.playerid, "next"));
                }
                _ => {}
            }
        } else {
            tracing::debug!("No active players for transport action");
        }
    } else {
        tracing::warn!("Failed to get active players");
    }
}

fn send_input(client: &Rc<RefCell<Option<KodiClient>>>, action: InputAction) {
    let binding = client.borrow();
    let c = if let Some(ref c) = *binding {
        c
    } else {
        tracing::debug!("No Kodi client for input action");
        return;
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            return;
        }
    };
    let _ = rt.block_on(c.input(action));
}

fn volume_change(client: &Rc<RefCell<Option<KodiClient>>>, delta: i32) {
    let binding = client.borrow();
    let c = if let Some(ref c) = *binding {
        c
    } else {
        tracing::debug!("No Kodi client for volume change");
        return;
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            return;
        }
    };
    if let Ok(p) = rt.block_on(c.get_application_properties()) {
        let new_vol = (p.volume + delta).clamp(0, 100);
        let _ = rt.block_on(c.set_volume(new_vol));
    } else {
        tracing::warn!("Failed to get application properties for volume change");
    }
}

fn volume_mute(client: &Rc<RefCell<Option<KodiClient>>>) {
    let binding = client.borrow();
    let c = if let Some(ref c) = *binding {
        c
    } else {
        tracing::debug!("No Kodi client for volume mute");
        return;
    };
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            return;
        }
    };
    if let Ok(p) = rt.block_on(c.get_application_properties()) {
        let _ = rt.block_on(c.set_mute(!p.muted));
    } else {
        tracing::warn!("Failed to get application properties for mute toggle");
    }
}
