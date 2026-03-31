use gtk::glib;
use gtk::pango;
use gtk::prelude::*;
use gtk::{Scale, Adjustment};
use std::cell::RefCell;
use std::rc::Rc;

use crate::kodi::KodiClient;

pub struct NowPlayingWidgets {
    pub title_label: gtk::Label,
    pub description_label: gtk::Label,
    pub seeker: Scale,
    pub seek_hint: gtk::Label,
    pub time_current: gtk::Label,
    pub time_remaining: gtk::Label,
    pub time_ends: gtk::Label,
}

pub struct NowPlayingState {
    client: Rc<RefCell<Option<KodiClient>>>,
    pub title: String,
    pub description: String,
    pub current_time: i64,
    pub duration: i64,
    pub is_seeking: bool,
    player_id: Option<i32>,
}

impl NowPlayingState {
    pub fn new(client: Rc<RefCell<Option<KodiClient>>>) -> Self {
        Self {
            client,
            title: "No media playing".to_string(),
            description: String::new(),
            current_time: 0,
            duration: 0,
            is_seeking: false,
            player_id: None,
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
            if let Ok(players) = rt.block_on(c.get_active_players()) {
                if let Some(p) = players.first() {
                    self.player_id = Some(p.playerid);

                    if let Ok(item) = rt.block_on(c.get_current_item(p.playerid)) {
                        let title = item.title.clone()
                            .or(item.label.clone())
                            .or(item.file.clone())
                            .unwrap_or_else(|| "Playing".to_string());

                        let mut desc_parts = Vec::new();
                        if let Some(artist) = item.artist {
                            desc_parts.push(artist.join(", "));
                        }
                        if let Some(album) = item.album {
                            desc_parts.push(album);
                        }
                        if let Some(show) = item.showtitle {
                            if let (Some(season), Some(episode)) = (item.season, item.episode) {
                                desc_parts.push(format!("S{:02}E{:02}", season, episode));
                            }
                            desc_parts.push(show);
                        }
                        self.description = desc_parts.join(" • ");
                        self.title = title;
                    }

                    if let Ok(props) = rt.block_on(c.get_player_properties(p.playerid)) {
                        self.current_time = props.time.to_seconds();
                        self.duration = props.totaltime.to_seconds();
                    }
                }
            }
        }
    }

    pub fn seek(&self, percent: f64) {
        let client = self.client.borrow();
        if let (Some(ref c), Some(player_id)) = (&*client, self.player_id) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let _ = rt.block_on(c.seek_percentage(player_id, percent));
        }
    }
}

pub fn create_now_playing(client: Rc<RefCell<Option<KodiClient>>>) -> (gtk::Box, NowPlayingWidgets, NowPlayingState) {
    let state = NowPlayingState::new(client);
    
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
    box_.set_margin_start(12);
    box_.set_margin_end(12);
    box_.set_margin_top(12);
    box_.set_margin_bottom(12);
    box_.set_hexpand(true);
    box_.set_vexpand(true);

    let poster = gtk::AspectFrame::new(0.5, 0.5, 0.666, false);
    let poster_image = gtk::Image::from_icon_name("media-video");
    poster_image.set_icon_size(gtk::IconSize::Large);
    poster.set_child(Some(&poster_image));
    box_.append(&poster);

    let title_label = gtk::Label::new(Some("<big><b>No media playing</b></big>"));
    title_label.set_halign(gtk::Align::Center);
    title_label.set_use_markup(true);
    title_label.set_hexpand(true);
    title_label.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&title_label);

    let description_label = gtk::Label::new(Some(""));
    description_label.set_halign(gtk::Align::Center);
    description_label.set_hexpand(true);
    description_label.set_ellipsize(pango::EllipsizeMode::End);
    box_.append(&description_label);

    let seeker_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    seeker_box.set_hexpand(true);

    let seeker_adj = Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);
    let seeker = Scale::new(gtk::Orientation::Horizontal, Some(&seeker_adj));
    seeker.set_hexpand(true);
    seeker.set_draw_value(false);
    seeker_box.append(&seeker);

    let seek_hint = gtk::Label::new(Some("👆 tap to seek"));
    seek_hint.set_halign(gtk::Align::Center);
    seek_hint.set_margin_top(4);
    seek_hint.set_hexpand(true);
    seek_hint.set_css_classes(&["secondary"]);
    seeker_box.append(&seek_hint);
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
        seeker,
        seek_hint,
        time_current,
        time_remaining,
        time_ends,
    };

    (box_, widgets, state)
}

pub fn update_now_playing(widgets: &mut NowPlayingWidgets, state: &mut NowPlayingState) {
    widgets.title_label.set_markup(&format!("<big><b>{}</b></big>", state.title));
    widgets.description_label.set_text(&state.description);

    if !state.is_seeking {
        widgets.seeker.set_value(state.percentage());
    }

    let remaining = state.duration - state.current_time;
    let end_ts = chrono::Local::now() + chrono::Duration::seconds(remaining);

    widgets.time_current.set_text(&format_time(state.current_time));
    widgets.time_remaining.set_text(&format!("-{}", format_time(remaining)));
    widgets.time_ends.set_text(&format!("ends {}", end_ts.format("%H:%M")));
    widgets.seek_hint.set_label("👆 tap to seek");
}

pub fn start_now_playing_polling(
    widgets: NowPlayingWidgets,
    state: NowPlayingState,
) {
    let mut widgets = widgets;
    let mut state = state;

    glib::source::timeout_add_seconds_local(1, move || {
        state.poll();
        update_now_playing(&mut widgets, &mut state);
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
