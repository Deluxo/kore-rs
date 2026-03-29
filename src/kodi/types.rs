use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub mac_address: Option<String>,
    pub wol_port: u16,
}

impl HostInfo {
    pub fn new(name: String, address: String, port: u16) -> Self {
        Self {
            name,
            address,
            port,
            mac_address: None,
            wol_port: 9,
        }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub version: String,
    pub build_date: String,
    pub hostname: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerItem {
    pub id: i32,
    pub r#type: String,
    pub title: Option<String>,
    pub artist: Option<Vec<String>>,
    pub album: Option<String>,
    pub showtitle: Option<String>,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub file: Option<String>,
    pub thumbnail: Option<String>,
    pub fanart: Option<String>,
    pub year: Option<i32>,
    pub runtime: Option<i32>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProperty {
    pub speed: i32,
    pub time: i64,
    pub duration: i64,
    pub playlistid: i32,
    pub currentaudiostream: Option<AudioStream>,
    pub subtitleenabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub index: i32,
    pub language: String,
    pub name: String,
    pub codec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub playlistid: i32,
    pub position: i32,
    pub mediaid: Option<i32>,
    pub r#type: Option<String>,
    pub title: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub file: String,
    pub filetype: Option<String>,
    pub title: Option<String>,
    pub label: String,
    pub thumbnail: Option<String>,
    pub fanart: Option<String>,
    pub r#type: String,
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
    pub name: String,
    pub version: ApplicationVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationVersion {
    pub major: i32,
    pub minor: i32,
    pub revision: String,
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
        static mut REQUEST_ID: i32 = 1;
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
            id: unsafe {
                REQUEST_ID += 1;
                REQUEST_ID
            },
        }
    }

    pub fn with_params<P: Serialize>(mut self, params: P) -> Self {
        self.params = Some(serde_json::to_value(params).ok()).flatten();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: String,
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "result")]
    pub result: Option<T>,
    #[serde(rename = "error")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerId {
    #[default]
    Active,
    Video,
    Audio,
}

impl std::fmt::Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerId::Active => write!(f, "activeplayer"),
            PlayerId::Video => write!(f, "playerid"),
            PlayerId::Audio => write!(f, "playerid"),
        }
    }
}
