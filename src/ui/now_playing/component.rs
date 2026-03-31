use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use gtk::Scale;
use std::cell::RefCell;
use std::rc::Rc;

use crate::kodi::KodiClient;

pub struct NowPlayingWidgets {
    pub title_label: gtk::Label,
    pub description_label: gtk::Label,
    pub plot_label: gtk::Label,
    pub seeker: Scale,
    pub time_current: gtk::Label,
    pub time_remaining: gtk::Label,
    pub time_ends: gtk::Label,
    pub poster_picture: gtk::Picture,
    pub poster_placeholder: gtk::Image,
}

pub struct NowPlayingState {
    client: Rc<RefCell<Option<KodiClient>>>,
    pub title: String,
    pub description: String,
    pub plot: String,
    pub current_time: i64,
    pub duration: i64,
    pub is_seeking: Rc<RefCell<bool>>,
    player_info: Rc<RefCell<Option<(i32, i64)>>>,
    pub thumbnail: Option<String>,
    last_thumbnail: Option<String>,
}

impl NowPlayingState {
    pub fn new(client: Rc<RefCell<Option<KodiClient>>>) -> Self {
        Self {
            client,
            title: "No media playing".to_string(),
            description: String::new(),
            plot: String::new(),
            current_time: 0,
            duration: 0,
            is_seeking: Rc::new(RefCell::new(false)),
            player_info: Rc::new(RefCell::new(None)),
            thumbnail: None,
            last_thumbnail: None,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.duration > 0 {
            (self.current_time as f64 / self.duration as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn poll(&mut self) {
        let client = self.client.borrow();
        if let Some(ref c) = *client {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let players = rt.block_on(c.get_active_players());
            tracing::debug!("Poll - players: {:?}", players);
            
            if let Ok(players) = players {
                if let Some(p) = players.first() {
                    tracing::info!("Found player with id: {}", p.playerid);
                    *self.player_info.borrow_mut() = Some((p.playerid, self.duration));

                    let item = rt.block_on(c.get_current_item(p.playerid));
                    tracing::debug!("Poll - item: {:?}", item);

                    if let Ok(item) = item {
                        let title = item.title.clone()
                            .or(item.label.clone())
                            .or(item.file.clone())
                            .unwrap_or_else(|| "Playing".to_string());

                        let mut desc_parts = Vec::new();
                        let media_type = item.r#type.as_deref().unwrap_or("unknown");
                        
                        if media_type == "episode" {
                            // TV show - show SxxExx and show title
                            if let Some(show) = item.showtitle {
                                if let (Some(season), Some(episode)) = (item.season, item.episode) {
                                    desc_parts.push(format!("S{:02}E{:02}", season, episode));
                                }
                                desc_parts.push(show);
                            }
                        } else if media_type == "song" || media_type == "unknown" {
                            // Music - show artist + album
                            if let Some(artist) = item.artist {
                                desc_parts.push(artist.join(", "));
                            }
                            if let Some(album) = item.album {
                                desc_parts.push(album);
                            }
                        }
                        // For movies ("movie"), don't add anything to description - title is enough
                        
                        self.description = desc_parts.join(" • ");
                        self.title = title;
                        self.plot = item.plot.clone().unwrap_or_else(|| item.tagline.clone().unwrap_or_default());

                        tracing::debug!("item.thumbnail: {:?}", item.thumbnail);
                        tracing::debug!("item.art: {:?}", item.art);
                        tracing::debug!("item.plot: {:?}", item.plot);
                        
                        // If art exists, use poster from it (fanart.tv), otherwise use thumbnail
                        if let Some(art) = &item.art {
                            if let Some(poster) = &art.poster {
                                if poster.contains("fanart.tv") || poster.contains("assets.fanart.tv") {
                                    self.thumbnail = Some(poster.clone());
                                }
                            }
                        }
                        
                        // If still no thumbnail, try item.thumbnail
                        if self.thumbnail.is_none() {
                            self.thumbnail = item.thumbnail.clone();
                        }
                        
                        // If thumbnail exists but is a local path (video@), try to get poster from art anyway
                        if self.thumbnail.is_none() || self.thumbnail.as_ref().map(|s| s.contains("video@")).unwrap_or(false) {
                            if let Some(art) = &item.art {
                                if let Some(poster) = &art.poster {
                                    if poster.contains("fanart.tv") || poster.contains("http") {
                                        self.thumbnail = Some(poster.clone());
                                    }
                                }
                            }
                        }
                        
                        tracing::info!("Final thumbnail: {:?}", self.thumbnail);

                        if let Ok(props) = rt.block_on(c.get_player_properties(p.playerid)) {
                            self.current_time = props.time.to_seconds();
                            self.duration = props.totaltime.to_seconds();
                        }
                    }
                } else {
                    // No active player - reset state
                    self.title = "No media playing".to_string();
                    self.description = String::new();
                    self.plot = String::new();
                    self.thumbnail = None;
                    self.current_time = 0;
                    self.duration = 0;
                    *self.player_info.borrow_mut() = None;
                }
            }
        }
    }
}

pub fn create_now_playing(client: Rc<RefCell<Option<KodiClient>>>) -> (gtk::Box, NowPlayingWidgets, NowPlayingState) {
    let state = NowPlayingState::new(client.clone());
    
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.set_margin_start(12);
    box_.set_margin_end(12);
    box_.set_margin_top(12);
    box_.set_margin_bottom(12);
    box_.set_hexpand(true);
    box_.set_vexpand(true);

    let poster = gtk::Frame::new(None);
    poster.set_valign(gtk::Align::Center);
    poster.set_halign(gtk::Align::Center);
    poster.set_hexpand(true);
    poster.set_vexpand(true);
    
    let poster_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    poster_box.set_halign(gtk::Align::Center);
    poster_box.set_valign(gtk::Align::Center);
    
    let poster_picture = gtk::Picture::new();
    poster_picture.set_hexpand(true);
    poster_picture.set_vexpand(true);
    poster_picture.set_size_request(300, -1);
    poster_picture.set_visible(false);
    
    let poster_placeholder = gtk::Image::from_icon_name("media-video");
    poster_placeholder.set_icon_size(gtk::IconSize::Large);
    poster_placeholder.set_size_request(300, 300);
    poster_placeholder.set_hexpand(true);
    poster_placeholder.set_vexpand(true);
    
    poster_box.append(&poster_picture);
    poster_box.append(&poster_placeholder);
    poster.set_child(Some(&poster_box));
    box_.append(&poster);

    let title_label = gtk::Label::new(Some("<big><b>No media playing</b></big>"));
    title_label.set_halign(gtk::Align::Center);
    title_label.set_use_markup(true);
    title_label.set_hexpand(true);
    title_label.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&title_label);

    let description_label = gtk::Label::new(Some(""));
    description_label.set_halign(gtk::Align::Start);
    description_label.set_hexpand(true);
    description_label.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&description_label);

    let plot_label = gtk::Label::new(Some(""));
    plot_label.set_halign(gtk::Align::Center);
    plot_label.set_wrap(true);
    plot_label.set_wrap_mode(pango::WrapMode::Word);
    plot_label.set_hexpand(true);
    plot_label.set_size_request(300, -1);
    plot_label.set_margin_start(8);
    plot_label.set_margin_end(8);
    plot_label.set_lines(4);
    box_.append(&plot_label);

    // Seeker - wrap in box for gesture handling
    let seeker_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    seeker_box.set_hexpand(true);
    
    let seeker_adj = gtk::Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);
    let seeker = Scale::new(gtk::Orientation::Horizontal, Some(&seeker_adj));
    seeker.set_hexpand(true);
    seeker.set_draw_value(false);
    seeker_box.append(&seeker);
    
    // Track user interaction state
    let player_info = state.player_info.clone();
    let client_for_adj = client.clone();
    let previous_value = Rc::new(RefCell::new(0.0f64));
    
    // Store initial value
    *previous_value.borrow_mut() = seeker_adj.value();
    
    // Use scale's value-changed to detect user interaction
    let player_info_adj = player_info.clone();
    let client_adj = client_for_adj.clone();
    let prev_val = previous_value.clone();
    seeker_adj.connect_value_changed(move |adj| {
        let curr = adj.value();
        let prev = *prev_val.borrow();
        
        // Only seek if value actually changed (user dragged)
        if (curr - prev).abs() > 0.5 {
            *prev_val.borrow_mut() = curr;
            
            if let Some((player_id, _)) = *player_info_adj.borrow() {
                if let Some(ref c) = *client_adj.borrow() {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let _ = rt.block_on(c.seek_percentage(player_id, curr));
                }
            }
        }
    });
    
    box_.append(&seeker_box);

    let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    time_box.set_hexpand(true);

    let time_current = gtk::Label::new(Some("0:00"));
    time_current.set_halign(gtk::Align::Start);
    time_current.set_hexpand(true);

    let time_remaining = gtk::Label::new(Some("-0:00"));
    time_remaining.set_halign(gtk::Align::Center);
    time_remaining.set_hexpand(true);

    let time_ends = gtk::Label::new(Some("ends 0:00"));
    time_ends.set_halign(gtk::Align::End);
    time_ends.set_hexpand(true);

    time_box.append(&time_current);
    time_box.append(&time_remaining);
    time_box.append(&time_ends);
    box_.append(&time_box);

    let widgets = NowPlayingWidgets {
        title_label,
        description_label,
        plot_label,
        seeker,
        time_current,
        time_remaining,
        time_ends,
        poster_picture,
        poster_placeholder,
    };

    (box_, widgets, state)
}

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
                true // Show placeholder on any failure
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

pub fn start_now_playing_polling(
    widgets: NowPlayingWidgets,
    state: NowPlayingState,
) {
    let client = state.client.clone();
    let mut widgets = widgets;
    let mut state = state;

    glib::source::timeout_add_seconds_local(1, move || {
        state.poll();
        update_now_playing(&mut widgets, &mut state, &client);
        glib::ControlFlow::Continue
    });
}

fn format_time(secs: i64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, s)
    } else {
        format!("{}:{:02}", mins, s)
    }
}
