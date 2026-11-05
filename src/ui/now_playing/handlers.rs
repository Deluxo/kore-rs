use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::transform::format_time;
use crate::io::kodi::actions::{self, TransportAction};
use crate::io::kodi::{with_client, ClientCell, KodiClient};
use crate::io::mpris::{MprisCommand, MprisHandle};

use super::layout::{NowPlayingState, NowPlayingWidgets};

pub fn update_now_playing(
    widgets: &mut NowPlayingWidgets,
    state: &mut NowPlayingState,
    client: &ClientCell,
) {
    widgets.title_label.set_markup(&format!("<big><b>{}</b></big>", state.title));
    widgets.description_label.set_text(&state.description);
    widgets.plot_label.set_text(&state.plot);

    widgets.seeker.set_value(state.percentage());

    let remaining = state.duration - state.current_time;
    let end_ts = chrono::Local::now() + chrono::Duration::seconds(remaining);

    widgets.time_current.set_text(&format_time(state.current_time));
    widgets.time_remaining.set_text(&format!("-{}", format_time(remaining)));
    widgets.time_ends.set_text(&format!("ends {}", end_ts.format("%H:%M")));

    let thumbnail_changed = state.thumbnail != state.last_thumbnail;
    state.last_thumbnail = state.thumbnail.clone();

    if !thumbnail_changed {
        return;
    }

    let thumbnail = state.thumbnail.clone();
    let show_placeholder = match &thumbnail {
        Some(url) => !load_thumbnail(widgets, state, client, url),
        None => true,
    };

    if show_placeholder {
        state.art_url = None;
        widgets.poster_picture.set_visible(false);
        widgets.poster_placeholder.set_visible(true);
    }
}

/// Download artwork once, cache it locally and store a `file://` URL for MPRIS.
fn load_thumbnail(
    widgets: &mut NowPlayingWidgets,
    state: &mut NowPlayingState,
    client: &ClientCell,
    url: &str,
) -> bool {
    tracing::debug!("=== FETCHING: {} ===", url);

    let Ok(client) = client.try_borrow() else {
        return false;
    };
    let Some(c) = client.as_ref() else {
        return false;
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!("Failed to create runtime: {}", e);
            return false;
        }
    };

    let fetch_result = rt.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(5), c.get_thumbnail(url)).await
    });

    let bytes = match fetch_result {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(e)) => {
            tracing::warn!("=== FETCH ERR: {} ===", e);
            return false;
        }
        Err(e) => {
            tracing::warn!("=== TIMEOUT: {} ===", e);
            return false;
        }
    };

    if bytes.len() <= 1000 {
        tracing::warn!("=== TOO SMALL: {} ===", bytes.len());
        return false;
    }

    let img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(e) => {
            tracing::warn!("=== DECODE ERR: {} ===", e);
            return false;
        }
    };

    let hash = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    };
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("korers");
    let _ = std::fs::create_dir_all(&cache_dir);
    let art_path = cache_dir.join(format!("thumb_{:016x}.png", hash));

    if img
        .save_with_format(&art_path, image::ImageFormat::Png)
        .is_err()
    {
        return false;
    }

    let Some(path_str) = art_path.to_str() else {
        return false;
    };

    tracing::debug!("=== SETTING PIC: {} ===", path_str);
    widgets.poster_picture.set_filename(Some(path_str));
    widgets.poster_picture.set_visible(true);
    widgets.poster_placeholder.set_visible(false);
    state.art_url = url::Url::from_file_path(&art_path)
        .ok()
        .map(|url| url.to_string());
    true
}

pub fn start_local_advancement(
    widgets: Rc<RefCell<NowPlayingWidgets>>,
    state: Rc<RefCell<NowPlayingState>>,
    client: ClientCell,
    prev_val: Rc<RefCell<f64>>,
    mpris: MprisHandle,
) {
    glib::source::timeout_add_local(
        std::time::Duration::from_millis(250),
        move || {
            drain_mpris_commands(&state, &client, &mpris);

            let mut s = state.borrow_mut();
            s.advance_locally();
            *prev_val.borrow_mut() = s.percentage();
            let mut w = widgets.borrow_mut();
            update_now_playing(&mut *w, &mut *s, &client);
            mpris.publish(s.snapshot());
            glib::ControlFlow::Continue
        },
    );
}

/// Execute queued MPRIS actions on the GTK thread, using the same client and
/// action helpers as the on-screen remote.
fn drain_mpris_commands(
    state: &Rc<RefCell<NowPlayingState>>,
    client: &ClientCell,
    mpris: &MprisHandle,
) {
    while let Some(command) = mpris.next_command() {
        let mut current = state.borrow_mut();
        if with_client(client, |c| execute_mpris(c, &mut *current, command)).is_none() {
            break;
        }
    }
}

fn execute_mpris(client: &KodiClient, state: &mut NowPlayingState, command: MprisCommand) {
    match command {
        MprisCommand::PlayPause => actions::transport(client, TransportAction::PlayPause),
        MprisCommand::Play => actions::transport(client, TransportAction::Play),
        MprisCommand::Pause => actions::transport(client, TransportAction::Pause),
        MprisCommand::Next => actions::transport(client, TransportAction::Next),
        MprisCommand::Previous => actions::transport(client, TransportAction::Previous),
        MprisCommand::Stop => {
            actions::transport(client, TransportAction::Stop);
            state.reset();
            return;
        }
        MprisCommand::SeekTo(micros) => actions::seek_seconds(client, micros / 1_000_000),
        MprisCommand::SetVolume(volume) => {
            actions::set_volume(client, (volume * 100.0).round() as i32)
        }
    }

    // Refresh metadata, status and volume from Kodi after the action, exactly
    // like a UI button press would.
    state.poll();
}

pub fn trigger_kodi_poll(
    widgets: &Rc<RefCell<NowPlayingWidgets>>,
    state: &Rc<RefCell<NowPlayingState>>,
    client: &ClientCell,
    prev_val: &Rc<RefCell<f64>>,
) {
    let mut s = state.borrow_mut();
    s.poll();
    *prev_val.borrow_mut() = s.percentage();
    let mut w = widgets.borrow_mut();
    update_now_playing(&mut *w, &mut *s, client);
}
