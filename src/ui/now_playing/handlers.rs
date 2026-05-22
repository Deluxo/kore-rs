use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::core::transform::format_time;
use crate::io::kodi::KodiClient;

use super::layout::{NowPlayingState, NowPlayingWidgets};

pub fn update_now_playing(widgets: &mut NowPlayingWidgets, state: &mut NowPlayingState, client: &Rc<RefCell<Option<KodiClient>>>) {
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

    if thumbnail_changed {
        let show_placeholder = match &state.thumbnail {
            Some(url) => {
                tracing::debug!("=== FETCHING: {} ===", url);

                if let Some(ref c) = *client.borrow() {
                    let rt = match tokio::runtime::Runtime::new() {
                        Ok(rt) => rt,
                        Err(e) => {
                            tracing::warn!("Failed to create runtime: {}", e);
                            return;
                        }
                    };
                    let fetch_result = rt.block_on(async {
                        tokio::time::timeout(std::time::Duration::from_secs(5), c.get_thumbnail(url)).await
                    });
                    match fetch_result {
                        Ok(Ok(bytes)) => {
                            tracing::debug!("=== SUCCESS: {} bytes ===", bytes.len());
                            if bytes.len() > 1000 {
                                match image::load_from_memory(&bytes) {
                                    Ok(img) => {
                                        tracing::debug!("=== DECODED: {}x{} ===", img.width(), img.height());
                                        let temp_dir = std::env::temp_dir();
                                        let temp_path = temp_dir.join("korers_thumb.png");
                                        if img.save_with_format(&temp_path, image::ImageFormat::Png).is_ok() {
                                            if let Some(path_str) = temp_path.to_str() {
                                                tracing::debug!("=== SETTING PIC: {} ===", path_str);
                                                widgets.poster_picture.set_filename(Some(path_str));
                                                widgets.poster_picture.set_visible(true);
                                                widgets.poster_placeholder.set_visible(false);
                                                tracing::debug!("=== DONE - showing pic ===");
                                                return;
                                            }
                                        }
                                    }
                                    Err(e) => tracing::warn!("=== DECODE ERR: {} ===", e),
                                }
                            } else {
                                tracing::warn!("=== TOO SMALL: {} ===", bytes.len());
                            }
                        }
                        Ok(Err(e)) => tracing::warn!("=== FETCH ERR: {} ===", e),
                        Err(e) => tracing::warn!("=== TIMEOUT: {} ===", e),
                    }
                }
                true
            }
            None => true,
        };

        if show_placeholder {
            tracing::debug!("=== SHOW PLACEHOLDER ===");
            widgets.poster_picture.set_visible(false);
            widgets.poster_placeholder.set_visible(true);
        }
    }
}

pub fn start_local_advancement(
    widgets: Rc<RefCell<NowPlayingWidgets>>,
    state: Rc<RefCell<NowPlayingState>>,
    client: Rc<RefCell<Option<KodiClient>>>,
    prev_val: Rc<RefCell<f64>>,
) {
    glib::source::timeout_add_local(
        std::time::Duration::from_millis(250),
        move || {
            let mut s = state.borrow_mut();
            s.advance_locally();
            *prev_val.borrow_mut() = s.percentage();
            let mut w = widgets.borrow_mut();
            update_now_playing(&mut *w, &mut *s, &client);
            glib::ControlFlow::Continue
        }
    );
}

pub fn trigger_kodi_poll(
    widgets: &Rc<RefCell<NowPlayingWidgets>>,
    state: &Rc<RefCell<NowPlayingState>>,
    client: &Rc<RefCell<Option<KodiClient>>>,
    prev_val: &Rc<RefCell<f64>>,
) {
    let mut s = state.borrow_mut();
    s.poll();
    *prev_val.borrow_mut() = s.percentage();
    let mut w = widgets.borrow_mut();
    update_now_playing(&mut *w, &mut *s, client);
}
