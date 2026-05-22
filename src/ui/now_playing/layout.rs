use gtk::pango;
use gtk::prelude::*;
use gtk::Scale;
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::kodi::KodiClient;

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
    player_info: Rc<RefCell<Option<(i32, i64)>>>,
    pub thumbnail: Option<String>,
    pub last_thumbnail: Option<String>,
    pub cached_speed: f64,
    last_poll_time: std::time::Instant,
    polled_time: i64,
    has_player: bool,
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
            player_info: Rc::new(RefCell::new(None)),
            thumbnail: None,
            last_thumbnail: None,
            cached_speed: 0.0,
            last_poll_time: std::time::Instant::now(),
            polled_time: 0,
            has_player: false,
        }
    }

    pub fn reset(&mut self) {
        self.title = "No media playing".to_string();
        self.description = String::new();
        self.plot = String::new();
        self.thumbnail = None;
        self.last_thumbnail = None;
        self.current_time = 0;
        self.duration = 0;
        self.cached_speed = 0.0;
        self.has_player = false;
        *self.player_info.borrow_mut() = None;
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
        let c = if let Some(ref c) = *client {
            c
        } else {
            tracing::debug!("No Kodi client available for poll");
            return;
        };
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!("Failed to create runtime in poll: {}", e);
                return;
            }
        };
        let players = rt.block_on(c.get_active_players());
        tracing::debug!("Poll - players: {:?}", players);

        if let Ok(players) = players {
            if let Some(p) = players.first() {
                tracing::info!("Found player with id: {}", p.playerid);
                *self.player_info.borrow_mut() = Some((p.playerid, self.duration));

                if let Ok(props) = rt.block_on(c.get_player_properties(p.playerid)) {
                    tracing::info!("Player properties - time: {:?}, totaltime: {:?}, speed: {}", props.time, props.totaltime, props.speed);
                    self.current_time = props.time.to_seconds();
                    self.duration = props.totaltime.to_seconds();
                    self.cached_speed = props.speed as f64;
                    self.has_player = true;
                    self.polled_time = self.current_time;
                    self.last_poll_time = std::time::Instant::now();

                    if props.speed == 0 {
                        if let Ok(recheck) = rt.block_on(c.get_active_players()) {
                            if recheck.is_empty() {
                                tracing::info!("Player stopped (re-check after speed=0)");
                                self.title = "No media playing".to_string();
                                self.description = String::new();
                                self.plot = String::new();
                                self.thumbnail = None;
                                self.current_time = 0;
                                self.duration = 0;
                                self.cached_speed = 0.0;
                                self.has_player = false;
                                *self.player_info.borrow_mut() = None;
                                return;
                            }
                        } else {
                            tracing::warn!("Failed to re-check active players after speed=0");
                        }
                    }
                } else {
                    self.cached_speed = 0.0;
                    self.has_player = false;
                }

                let item = rt.block_on(c.get_current_item(p.playerid));
                tracing::debug!("Poll - item: {:?}", item);

                if let Ok(item) = item {
                    use crate::core::transform;

                    let title = transform::format_media_title(
                        item.title.as_deref(),
                        item.label.as_deref(),
                        item.file.as_deref(),
                    );

                    let desc = transform::format_media_description(
                        item.r#type.as_deref().unwrap_or("unknown"),
                        item.showtitle.as_deref(),
                        item.season,
                        item.episode,
                        item.artist.as_deref(),
                        item.album.as_deref(),
                    );

                    self.description = desc;
                    self.title = title;
                    self.plot = item.plot.clone().unwrap_or_else(|| item.tagline.clone().unwrap_or_default());
                    self.thumbnail = transform::resolve_thumbnail(
                        item.art.as_ref().and_then(|art| art.poster.as_deref()),
                        item.thumbnail.as_deref(),
                    );
                }
            } else {
                self.title = "No media playing".to_string();
                self.description = String::new();
                self.plot = String::new();
                self.thumbnail = None;
                self.current_time = 0;
                self.duration = 0;
                self.cached_speed = 0.0;
                self.has_player = false;
                *self.player_info.borrow_mut() = None;
            }
        }
    }

    pub fn advance_locally(&mut self) {
        if !self.has_player || self.cached_speed == 0.0 {
            return;
        }
        let elapsed = self.last_poll_time.elapsed().as_secs_f64();
        let new_time = self.polled_time as f64 + elapsed * self.cached_speed;
        self.current_time = (new_time as i64).clamp(0, self.duration);
    }
}

pub struct SeekerContext {
    pub adjustment: gtk::Adjustment,
    pub player_info: Rc<RefCell<Option<(i32, i64)>>>,
    pub client: Rc<RefCell<Option<KodiClient>>>,
    pub prev_val: Rc<RefCell<f64>>,
}

pub fn create_now_playing(client: Rc<RefCell<Option<KodiClient>>>) -> (gtk::Box, NowPlayingWidgets, NowPlayingState, SeekerContext) {
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
    poster_placeholder.set_size_request(100, 100);
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

    // Seeker
    let seeker_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    seeker_box.set_hexpand(true);

    let seeker_adj = gtk::Adjustment::new(0.0, 0.0, 100.0, 0.1, 1.0, 0.0);
    let seeker = Scale::new(gtk::Orientation::Horizontal, Some(&seeker_adj));
    seeker.set_hexpand(true);
    seeker.set_draw_value(false);
    seeker_box.append(&seeker);

    let seeker_ctx = SeekerContext {
        adjustment: seeker_adj.clone(),
        player_info: state.player_info.clone(),
        client: client.clone(),
        prev_val: Rc::new(RefCell::new(seeker_adj.value())),
    };

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

    (box_, widgets, state, seeker_ctx)
}
