use std::time::{Duration, Instant};

use super::types::InputAction;
use super::KodiClient;

#[derive(Debug, Clone, Copy)]
pub enum TransportAction {
    PlayPause,
    Play,
    Pause,
    Stop,
    Next,
    Previous,
}

fn runtime() -> Option<tokio::runtime::Runtime> {
    match tokio::runtime::Runtime::new() {
        Ok(rt) => Some(rt),
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            None
        }
    }
}

/// Perform a transport action on the first active Kodi player.
pub fn transport(client: &KodiClient, action: TransportAction) {
    let Some(rt) = runtime() else {
        return;
    };

    let Ok(players) = rt.block_on(client.get_active_players()) else {
        tracing::warn!("Failed to get active players");
        return;
    };
    let Some(player) = players.first() else {
        tracing::debug!("No active players for transport action");
        return;
    };
    let player_id = player.playerid;

    match action {
        TransportAction::PlayPause => {
            if let Err(e) = rt.block_on(client.play_pause(player_id)) {
                tracing::warn!("play_pause error: {:?}", e);
            }
        }
        TransportAction::Play => {
            if let Err(e) = rt.block_on(client.play_pause_with(player_id, true)) {
                tracing::warn!("play error: {:?}", e);
            }
        }
        TransportAction::Pause => {
            if let Err(e) = rt.block_on(client.play_pause_with(player_id, false)) {
                tracing::warn!("pause error: {:?}", e);
            }
        }
        TransportAction::Next => {
            let _ = rt.block_on(client.go_to(player_id, "next"));
        }
        TransportAction::Previous => {
            let _ = rt.block_on(client.go_to(player_id, "previous"));
        }
        TransportAction::Stop => {
            let _ = rt.block_on(client.stop(player_id));
            // Kodi can keep the player in GetActivePlayers with speed=0 after
            // Stop; wait briefly so the follow-up poll sees it disappear.
            let deadline = Instant::now() + Duration::from_millis(1000);
            while Instant::now() < deadline {
                if let Ok(players) = rt.block_on(client.get_active_players()) {
                    if !players.iter().any(|p| p.playerid == player_id) {
                        break;
                    }
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Seek the active player to an absolute position.
pub fn seek_seconds(client: &KodiClient, seconds: i64) {
    let Some(rt) = runtime() else {
        return;
    };
    let Ok(players) = rt.block_on(client.get_active_players()) else {
        return;
    };
    let Some(player) = players.first() else {
        return;
    };
    if let Err(e) = rt.block_on(client.seek(player.playerid, seconds.max(0))) {
        tracing::warn!("seek error: {:?}", e);
    }
}

pub fn set_volume(client: &KodiClient, volume: i32) {
    let Some(rt) = runtime() else {
        return;
    };
    if let Err(e) = rt.block_on(client.set_volume(volume.clamp(0, 100))) {
        tracing::warn!("set_volume error: {:?}", e);
    }
}

pub fn adjust_volume(client: &KodiClient, delta: i32) {
    let Some(rt) = runtime() else {
        return;
    };
    if let Ok(props) = rt.block_on(client.get_application_properties()) {
        let _ = rt.block_on(client.set_volume((props.volume + delta).clamp(0, 100)));
    } else {
        tracing::warn!("Failed to get application properties for volume change");
    }
}

pub fn toggle_mute(client: &KodiClient) {
    let Some(rt) = runtime() else {
        return;
    };
    if let Ok(props) = rt.block_on(client.get_application_properties()) {
        let _ = rt.block_on(client.set_mute(!props.muted));
    } else {
        tracing::warn!("Failed to get application properties for mute toggle");
    }
}

pub fn input(client: &KodiClient, action: InputAction) {
    let Some(rt) = runtime() else {
        return;
    };
    let _ = rt.block_on(client.input(action));
}
