//! MPRIS (Media Player Remote Interfacing Specification) service.
//!
//! Exposes Kodi's current player on the session bus as
//! `org.mpris.MediaPlayer2.korers`, so desktop shells and mobile lock screens
//! (Phosh, GNOME, KDE, playerctl, ...) can display and control playback.
//!
//! This module does not talk to Kodi. The GTK ticker publishes snapshots here
//! and drains the command queue, keeping all Kodi I/O on the existing on-demand
//! path.

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};

use zbus::fdo::Properties;
use zbus::names::InterfaceName;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, Value};

use crate::io::kodi::types::PlayerItem;

const BUS_NAME: &str = "org.mpris.MediaPlayer2.korers";
const OBJECT_PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

/// Everything MPRIS exposes, built from `NowPlayingState` by the GTK ticker.
#[derive(Clone, Debug)]
pub struct MprisSnapshot {
    pub playback_status: String,
    pub position_micros: i64,
    pub duration_micros: i64,
    pub speed: f64,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub art_url: String,
    pub track_id: String,
    pub has_player: bool,
    /// Kodi volume, 0.0..=1.0.
    pub volume: f64,
    pub updated: Instant,
}

impl Default for MprisSnapshot {
    fn default() -> Self {
        Self {
            playback_status: "Stopped".to_string(),
            position_micros: 0,
            duration_micros: 0,
            speed: 0.0,
            title: String::new(),
            artists: Vec::new(),
            album: String::new(),
            art_url: String::new(),
            track_id: String::new(),
            has_player: false,
            volume: 0.5,
            updated: Instant::now(),
        }
    }
}

impl MprisSnapshot {
    /// Position extrapolated to "now" while playing.
    fn position_now(&self) -> i64 {
        if self.playback_status != "Playing" || self.speed <= 0.0 {
            return self.position_micros;
        }
        let elapsed = self.updated.elapsed().as_secs_f64();
        let position = self.position_micros + (elapsed * self.speed * 1_000_000.0) as i64;
        if self.duration_micros > 0 {
            position.min(self.duration_micros)
        } else {
            position
        }
    }

    fn set_position(&mut self, micros: i64) {
        self.position_micros = micros.max(0);
        self.updated = Instant::now();
    }

    fn metadata(&self) -> HashMap<String, Value<'static>> {
        let mut metadata = HashMap::new();
        if !self.has_player {
            return metadata;
        }

        let track_id = if self.track_id.is_empty() {
            "/org/mpris/MediaPlayer2/korers/track/none"
        } else {
            &self.track_id
        };
        metadata.insert(
            "mpris:trackid".to_string(),
            Value::from(ObjectPath::from_string_unchecked(track_id.to_string())),
        );

        if self.duration_micros > 0 {
            metadata.insert("mpris:length".to_string(), Value::from(self.duration_micros));
        }
        if !self.title.is_empty() {
            metadata.insert("xesam:title".to_string(), Value::from(self.title.clone()));
        }
        if !self.artists.is_empty() {
            metadata.insert(
                "xesam:artist".to_string(),
                Value::from(self.artists.clone()),
            );
        }
        if !self.album.is_empty() {
            metadata.insert("xesam:album".to_string(), Value::from(self.album.clone()));
        }
        if !self.art_url.is_empty() {
            metadata.insert(
                "mpris:artUrl".to_string(),
                Value::from(self.art_url.clone()),
            );
        }

        metadata
    }

    fn same_metadata(&self, other: &Self) -> bool {
        self.title == other.title
            && self.artists == other.artists
            && self.album == other.album
            && self.art_url == other.art_url
            && self.duration_micros == other.duration_micros
            && self.track_id == other.track_id
            && self.has_player == other.has_player
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MprisCommand {
    PlayPause,
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    /// Absolute position in microseconds.
    SeekTo(i64),
    /// Volume in the MPRIS 0.0..=1.0 range.
    SetVolume(f64),
}

struct MprisData {
    snapshot: Mutex<MprisSnapshot>,
    commands: tokio::sync::mpsc::UnboundedSender<MprisCommand>,
}

/// GTK-side handle for publishing state and draining queued commands.
#[derive(Clone)]
pub struct MprisHandle {
    snapshots: tokio::sync::mpsc::UnboundedSender<MprisSnapshot>,
    commands: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<MprisCommand>>>,
}

impl MprisHandle {
    pub fn publish(&self, snapshot: MprisSnapshot) {
        let _ = self.snapshots.send(snapshot);
    }

    pub fn next_command(&self) -> Option<MprisCommand> {
        let mut commands = self.commands.lock().ok()?;
        commands.try_recv().ok()
    }
}

pub struct MprisService {
    data: Arc<MprisData>,
    snapshots: tokio::sync::mpsc::UnboundedReceiver<MprisSnapshot>,
}

static STARTED: Once = Once::new();

impl MprisService {
    pub fn new() -> (Self, MprisHandle) {
        let (snapshot_tx, snapshot_rx) = tokio::sync::mpsc::unbounded_channel();
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

        let data = Arc::new(MprisData {
            snapshot: Mutex::new(MprisSnapshot::default()),
            commands: command_tx,
        });

        (
            Self {
                data,
                snapshots: snapshot_rx,
            },
            MprisHandle {
                snapshots: snapshot_tx,
                commands: Arc::new(Mutex::new(command_rx)),
            },
        )
    }

    /// Start the MPRIS service on a dedicated thread/runtime.
    ///
    /// Calling this more than once is a no-op. If there is no session bus the
    /// error is logged and the rest of the application keeps working.
    pub fn start(self) {
        STARTED.call_once(move || {
            let spawned = std::thread::Builder::new()
                .name("korers-mpris".to_string())
                .spawn(move || {
                    let runtime = match tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(runtime) => runtime,
                        Err(e) => {
                            tracing::warn!("Failed to create MPRIS runtime: {}", e);
                            return;
                        }
                    };

                    runtime.block_on(async move {
                        if let Err(e) = self.run().await {
                            tracing::warn!("MPRIS service unavailable: {}", e);
                        }
                    });
                });

            if let Err(e) = spawned {
                tracing::warn!("Failed to spawn MPRIS thread: {}", e);
            }
        });
    }

    async fn run(self) -> Result<(), zbus::Error> {
        let connection = zbus::connection::Builder::session()?.build().await?;

        connection
            .object_server()
            .at(OBJECT_PATH, MprisRoot)
            .await?;
        connection
            .object_server()
            .at(OBJECT_PATH, MprisPlayer(self.data.clone()))
            .await?;

        // Register the well-known name only after the objects exist, so clients
        // can never see the name without its interfaces.
        connection.request_name(BUS_NAME).await?;
        tracing::info!("MPRIS service registered as {}", BUS_NAME);

        let emitter = SignalEmitter::new(&connection, OBJECT_PATH)?.into_owned();
        let data = self.data.clone();
        tokio::spawn(Self::publish_loop(data, self.snapshots, emitter));

        // Keep the connection (and therefore the service) alive.
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    }

    async fn publish_loop(
        data: Arc<MprisData>,
        mut snapshots: tokio::sync::mpsc::UnboundedReceiver<MprisSnapshot>,
        emitter: SignalEmitter<'static>,
    ) {
        let mut last = MprisSnapshot::default();

        while let Some(next) = snapshots.recv().await {
            let changed = Self::diff(&last, &next);

            if let Ok(mut snapshot) = data.snapshot.lock() {
                *snapshot = next.clone();
            }

            Self::emit_changed(&emitter, changed).await;
            last = next;
        }
    }

    fn diff(
        old: &MprisSnapshot,
        new: &MprisSnapshot,
    ) -> HashMap<&'static str, Value<'static>> {
        let mut changed = HashMap::new();

        if old.playback_status != new.playback_status {
            changed.insert(
                "PlaybackStatus",
                Value::from(new.playback_status.clone()),
            );
        }

        if !old.same_metadata(new) {
            changed.insert("Metadata", Value::from(new.metadata()));
        }

        if (old.volume - new.volume).abs() > 0.001 {
            changed.insert("Volume", Value::from(new.volume));
        }

        if old.has_player != new.has_player {
            for property in [
                "CanGoNext",
                "CanGoPrevious",
                "CanPlay",
                "CanPause",
                "CanSeek",
            ] {
                changed.insert(property, Value::from(new.has_player));
            }
        }

        changed
    }

    async fn emit_changed(
        emitter: &SignalEmitter<'static>,
        changed: HashMap<&'static str, Value<'static>>,
    ) {
        if changed.is_empty() {
            return;
        }

        if let Err(e) = Properties::properties_changed(
            emitter,
            InterfaceName::from_static_str_unchecked(PLAYER_INTERFACE),
            changed,
            Cow::Borrowed(&[]),
        )
        .await
        {
            tracing::debug!("Failed to emit MPRIS PropertiesChanged: {}", e);
        }
    }
}

/// Stable MPRIS track id for a Kodi item.
pub fn track_id(item: &PlayerItem) -> String {
    if let Some(id) = item.id {
        let kind: String = item
            .r#type
            .as_deref()
            .unwrap_or("item")
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        return format!(
            "/org/mpris/MediaPlayer2/korers/track/{}_{}",
            kind,
            id.unsigned_abs()
        );
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    item.title.hash(&mut hasher);
    item.file.hash(&mut hasher);
    item.album.hash(&mut hasher);
    format!(
        "/org/mpris/MediaPlayer2/korers/track/{:016x}",
        hasher.finish()
    )
}

// --- org.mpris.MediaPlayer2 (root interface) ---

struct MprisRoot;

#[zbus::interface(name = "org.mpris.MediaPlayer2")]
impl MprisRoot {
    #[zbus(property)]
    fn can_quit(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn identity(&self) -> String {
        "Korers".to_string()
    }

    #[zbus(property)]
    fn desktop_entry(&self) -> String {
        "kore-rs".to_string()
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        Vec::new()
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        Vec::new()
    }
}

// --- org.mpris.MediaPlayer2.Player ---

struct MprisPlayer(Arc<MprisData>);

#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl MprisPlayer {
    async fn next(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::Next);
        Ok(())
    }

    async fn previous(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::Previous);
        Ok(())
    }

    async fn pause(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::Pause);
        Ok(())
    }

    async fn play_pause(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::PlayPause);
        Ok(())
    }

    async fn stop(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::Stop);
        Ok(())
    }

    async fn play(&mut self) -> zbus::fdo::Result<()> {
        self.send(MprisCommand::Play);
        Ok(())
    }

    async fn seek(
        &mut self,
        offset: i64,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        let snapshot = self.snapshot();
        if !snapshot.has_player {
            return Ok(());
        }

        let target = (snapshot.position_now() + offset).max(0);
        self.send(MprisCommand::SeekTo(target));
        if let Ok(mut snapshot) = self.0.snapshot.lock() {
            snapshot.set_position(target);
        }
        let _ = emitter.seeked(target).await;
        Ok(())
    }

    async fn set_position(
        &mut self,
        track_id: ObjectPath<'_>,
        position: i64,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        let snapshot = self.snapshot();
        if !snapshot.has_player {
            return Ok(());
        }
        // The spec says a stale track id must be ignored.
        if !snapshot.track_id.is_empty()
            && track_id.as_str() != snapshot.track_id.as_str()
        {
            return Ok(());
        }

        let target = position.max(0);
        self.send(MprisCommand::SeekTo(target));
        if let Ok(mut snapshot) = self.0.snapshot.lock() {
            snapshot.set_position(target);
        }
        let _ = emitter.seeked(target).await;
        Ok(())
    }

    async fn open_uri(&mut self, _uri: String) -> zbus::fdo::Result<()> {
        Err(zbus::fdo::Error::NotSupported(
            "Korers does not support OpenUri".to_string(),
        ))
    }

    #[zbus(property)]
    fn playback_status(&self) -> String {
        self.snapshot().playback_status
    }

    #[zbus(property)]
    fn loop_status(&self) -> String {
        "None".to_string()
    }

    #[zbus(property)]
    fn set_loop_status(&mut self, _status: String) {}

    #[zbus(property)]
    fn rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn set_rate(&mut self, _rate: f64) {}

    #[zbus(property)]
    fn shuffle(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn set_shuffle(&mut self, _shuffle: bool) {}

    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, Value<'static>> {
        self.snapshot().metadata()
    }

    #[zbus(property)]
    fn volume(&self) -> f64 {
        self.snapshot().volume.clamp(0.0, 1.0)
    }

    #[zbus(property)]
    async fn set_volume(&mut self, volume: f64) -> zbus::fdo::Result<()> {
        let volume = volume.clamp(0.0, 1.0);
        self.send(MprisCommand::SetVolume(volume));
        if let Ok(mut snapshot) = self.0.snapshot.lock() {
            snapshot.volume = volume;
        }
        Ok(())
    }

    #[zbus(property)]
    fn position(&self) -> i64 {
        self.snapshot().position_now()
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        self.has_player()
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        self.has_player()
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        self.has_player()
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        self.has_player()
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        self.has_player()
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(signal)]
    async fn seeked(emitter: &SignalEmitter<'_>, position: i64) -> zbus::Result<()>;
}

impl MprisPlayer {
    fn snapshot(&self) -> MprisSnapshot {
        match self.0.snapshot.lock() {
            Ok(snapshot) => snapshot.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    fn has_player(&self) -> bool {
        self.snapshot().has_player
    }

    fn send(&self, command: MprisCommand) {
        let _ = self.0.commands.send(command);
    }
}
