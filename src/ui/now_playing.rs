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

impl NowPlayingModel {
    pub fn update(&mut self, msg: NowPlayingMsg) {
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
