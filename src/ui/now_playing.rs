use relm4::gtk;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt,
    view,
};

#[derive(Debug, Clone)]
pub enum NowPlayingMsg {
    Refresh,
    PlayPause,
    Stop,
    Next,
    Previous,
    Seek(i64),
}

#[derive(Debug, Default)]
pub struct NowPlayingModel {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration: i64,
    pub current_time: i64,
    pub is_playing: bool,
}

#[relm4::component]
impl SimpleComponent for NowPlayingModel {
    type Init = ();
    type Input = NowPlayingMsg;
    type Output = ();
    type Widgets = NowPlayingWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_spacing: 20,
            set_margin_all: 20,

            #[name = "thumbnail"]
            gtk::Image {
                set_width_request: 300,
                set_height_request: 300,
                set_icon_name: Some("audio-x-generic"),
                set_pixel_size: 200,
            },

            #[name = "title_label"]
            gtk::Label {
                set_label: "No media playing",
                set_markup: Some("<span size='large' weight='bold'>No media playing</span>"),
                set_halign: gtk::Align::Center,
            },

            #[name = "artist_label"]
            gtk::Label {
                set_label: "",
                set_markup: Some("<span size='medium'>-</span>"),
                set_halign: gtk::Align::Center,
            },

            #[name = "album_label"]
            gtk::Label {
                set_label: "",
                set_markup: Some("<span size='small' foreground='gray'>-</span>"),
                set_halign: gtk::Align::Center,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 20,

                #[name = "time_current"]
                gtk::Label {
                    set_label: "0:00",
                    set_width_request: 50,
                },

                #[name = "progress_scale"]
                gtk::Scale {
                    set_draw_value: false,
                    set_adjustment: &gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 5.0, 0.0),
                    set_hexpand: true,
                    set_width_request: 300,
                },

                #[name = "time_duration"]
                gtk::Label {
                    set_label: "0:00",
                    set_width_request: 50,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 20,

                #[name = "btn_previous"]
                gtk::Button {
                    set_label: "⏮",
                    set_width_request: 50,
                    set_height_request: 50,
                    connect_clicked => NowPlayingMsg::Previous,
                },

                #[name = "btn_play_pause"]
                gtk::Button {
                    set_label: "▶",
                    set_width_request: 60,
                    set_height_request: 60,
                    connect_clicked => NowPlayingMsg::PlayPause,
                },

                #[name = "btn_stop"]
                gtk::Button {
                    set_label: "⏹",
                    set_width_request: 50,
                    set_height_request: 50,
                    connect_clicked => NowPlayingMsg::Stop,
                },

                #[name = "btn_next"]
                gtk::Button {
                    set_label: "⏭",
                    set_width_request: 50,
                    set_height_request: 50,
                    connect_clicked => NowPlayingMsg::Next,
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                set_halign: gtk::Align::Center,
                set_margin_top: 20,

                gtk::Button {
                    set_label: "🔊-",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "🔇",
                    set_width_request: 50,
                },
                gtk::Button {
                    set_label: "🔊+",
                    set_width_request: 50,
                },
            },
        }
    }

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = NowPlayingModel::default();
        let widgets = NowPlayingWidgets::from_builder(&root, ());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NowPlayingMsg::Refresh => {
                tracing::debug!("Refreshing now playing info");
            }
            NowPlayingMsg::PlayPause => {
                self.is_playing = !self.is_playing;
                tracing::debug!("Play/Pause: {}", self.is_playing);
            }
            NowPlayingMsg::Stop => {
                self.is_playing = false;
                tracing::debug!("Stop");
            }
            NowPlayingMsg::Next => {
                tracing::debug!("Next track");
            }
            NowPlayingMsg::Previous => {
                tracing::debug!("Previous track");
            }
            NowPlayingMsg::Seek(time) => {
                self.current_time = time;
                tracing::debug!("Seek to: {}s", time);
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        if let Some(title) = &self.title {
            widgets.title_label.set_markup(&format!(
                "<span size='large' weight='bold'>{}</span>",
                title
            ));
        }

        if let Some(artist) = &self.artist {
            widgets.artist_label.set_markup(&format!(
                "<span size='medium'>{}</span>",
                artist
            ));
        }

        widgets.btn_play_pause.set_label(if self.is_playing { "⏸" } else { "▶" });
    }
}

#[relm4::macros::widget_struct]
pub struct NowPlayingWidgets {
    pub thumbnail: gtk::Image,
    pub title_label: gtk::Label,
    pub artist_label: gtk::Label,
    pub album_label: gtk::Label,
    pub time_current: gtk::Label,
    pub time_duration: gtk::Label,
    pub progress_scale: gtk::Scale,
    pub btn_previous: gtk::Button,
    pub btn_play_pause: gtk::Button,
    pub btn_stop: gtk::Button,
    pub btn_next: gtk::Button,
}
