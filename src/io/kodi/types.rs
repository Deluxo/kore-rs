use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub mac_address: Option<String>,
    pub wol_port: Option<u16>,
}

impl HostInfo {
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: Option<String>,
    pub version: Option<serde_json::Value>,
    pub build_date: Option<String>,
    pub hostname: Option<String>,
    pub os: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Art {
    pub poster: Option<String>,
    pub fanart: Option<String>,
    pub banner: Option<String>,
    pub thumb: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerItem {
    pub id: Option<i32>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub title: Option<String>,
    pub label: Option<String>,
    pub artist: Option<Vec<String>>,
    pub album: Option<String>,
    pub showtitle: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub file: Option<String>,
    pub thumbnail: Option<String>,
    pub fanart: Option<String>,
    pub art: Option<Art>,
    pub year: Option<i32>,
    pub runtime: Option<i32>,
    pub duration: Option<i32>,
    pub plot: Option<String>,
    pub tagline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetItemResponse {
    pub item: PlayerItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTime {
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub milliseconds: i64,
}

impl PlayerTime {
    pub fn to_seconds(&self) -> i64 {
        self.hours * 3600 + self.minutes * 60 + self.seconds
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProperty {
    pub speed: i32,
    pub time: PlayerTime,
    pub totaltime: PlayerTime,
    pub playlistid: Option<i32>,
    #[serde(default)]
    pub currentaudiostream: Option<AudioStream>,
    #[serde(default)]
    pub subtitleenabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub index: i32,
    pub language: Option<String>,
    pub name: Option<String>,
    pub codec: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePlayer {
    pub playerid: i32,
    #[serde(rename = "type")]
    pub r#type: String,
    pub playertype: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub playlistid: Option<i32>,
    pub position: Option<i32>,
    pub mediaid: Option<i32>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    pub title: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub file: String,
    pub filetype: String,
    pub title: Option<String>,
    pub label: Option<String>,
    pub thumbnail: Option<String>,
    pub fanart: Option<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetails {
    pub title: Option<String>,
    pub plot: Option<String>,
    pub plotoutline: Option<String>,
    pub tagline: Option<String>,
    pub genre: Option<Vec<String>>,
    pub year: Option<i32>,
    pub rating: Option<f64>,
    pub votes: Option<i32>,
    pub runtime: Option<i32>,
    pub mpaa: Option<String>,
    pub director: Option<Vec<String>>,
    pub studio: Option<Vec<String>>,
    pub premiered: Option<String>,
    pub streamurl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicDetails {
    pub title: Option<String>,
    pub artist: Option<Vec<String>>,
    pub album: Option<String>,
    pub albumartist: Option<Vec<String>>,
    pub genre: Option<Vec<String>>,
    pub year: Option<i32>,
    pub track: Option<i32>,
    pub disc: Option<i32>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationProperty {
    pub volume: i32,
    pub muted: bool,
    pub name: Option<String>,
    pub version: Option<ApplicationVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationVersion {
    pub major: i32,
    pub minor: i32,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: i32,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
            id: next_id(),
        }
    }

    pub fn with_params(mut self, params: impl Serialize) -> Self {
        self.params = Some(serde_json::to_value(params).unwrap());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: Option<String>,
    pub id: Option<i32>,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favourite {
    pub title: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub path: Option<String>,
    pub thumbnail: Option<String>,
    pub window: Option<String>,
    pub windowparameter: Option<String>,
}

#[derive(Debug, Clone)]
pub enum InputAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Home,
    Info,
    ContextMenu,
    PreviousMenu,
}

impl std::fmt::Display for InputAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputAction::Up => write!(f, "up"),
            InputAction::Down => write!(f, "down"),
            InputAction::Left => write!(f, "left"),
            InputAction::Right => write!(f, "right"),
            InputAction::Select => write!(f, "select"),
            InputAction::Back => write!(f, "back"),
            InputAction::Home => write!(f, "home"),
            InputAction::Info => write!(f, "info"),
            InputAction::ContextMenu => write!(f, "contextmenu"),
            InputAction::PreviousMenu => write!(f, "previousmenu"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerId {
    Active,
    Video,
    Audio,
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerId::Active => write!(f, "activeplayers"),
            PlayerId::Video => write!(f, "video"),
            PlayerId::Audio => write!(f, "audio"),
        }
    }
}

static mut ID_COUNTER: i32 = 0;

fn next_id() -> i32 {
    unsafe {
        ID_COUNTER += 1;
        ID_COUNTER
    }
}
