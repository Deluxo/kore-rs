/// Pure snapshot of player state at a point in time.
/// No IO dependencies — just data.
#[derive(Debug, Clone)]
pub struct PlayerSnapshot {
    pub title: String,
    pub description: String,
    pub plot: String,
    pub current_time: i64,
    pub duration: i64,
    pub speed: f64,
    pub has_player: bool,
    pub thumbnail: Option<String>,
}

impl PlayerSnapshot {
    /// State when no media is playing.
    pub fn idle() -> Self {
        Self {
            title: "No media playing".to_string(),
            description: String::new(),
            plot: String::new(),
            current_time: 0,
            duration: 0,
            speed: 0.0,
            has_player: false,
            thumbnail: None,
        }
    }

    /// Seeker position as a percentage of duration.
    pub fn percentage(&self) -> f64 {
        if self.duration > 0 {
            (self.current_time as f64 / self.duration as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Advance current_time by wall-clock elapsed seconds × playback speed.
    /// Same logic as the original advance_locally but without any Instant dependency.
    pub fn advance_locally(&mut self, elapsed_secs: f64) {
        if !self.has_player || self.speed == 0.0 {
            return;
        }
        let new_time = self.current_time as f64 + elapsed_secs * self.speed;
        self.current_time = (new_time as i64).clamp(0, self.duration);
    }
}
