use gtk::prelude::*;
use relm4::{
    ComponentParts, ComponentSender, SimpleComponent,
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

pub struct RootWidgets;

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
    type Widgets = RootWidgets;

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = NowPlayingModel::default();
        ComponentParts { model, widgets: () }
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
}
