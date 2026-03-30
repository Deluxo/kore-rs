use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::*;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Kodi error: {code} {message}")]
    Kodi { code: i32, message: String },
    #[error("Not connected")]
    NotConnected,
    #[error("Request error: {0}")]
    Request(String),
}

#[derive(Clone)]
pub struct KodiClient {
    client: Client,
    base_url: String,
}

impl KodiClient {
    pub fn new(host: &HostInfo) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: host.url(),
        }
    }

    pub fn from_url(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.into(),
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    async fn call<R: for<'de> Deserialize<'de> + Sized>(
        &self,
        request: JsonRpcRequest,
    ) -> Result<R, ClientError> {
        let response = self
            .client
            .post(format!("{}/jsonrpc", self.base_url))
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse<R> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(ClientError::Kodi {
                code: error.code,
                message: error.message,
            });
        }

        rpc_response
            .result
            .ok_or_else(|| ClientError::Request("No result in response".to_string()))
    }

    pub async fn ping(&self) -> Result<String, ClientError> {
        #[derive(Deserialize)]
        struct PingResult(String);

        let result: PingResult = self.call(JsonRpcRequest::new("JSONRPC.Ping")).await?;
        Ok(result.0)
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, ClientError> {
        self.call(JsonRpcRequest::new("System.GetInfo")).await
    }

    pub async fn get_application_properties(
        &self,
    ) -> Result<ApplicationProperty, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<String>,
        }

        self.call(JsonRpcRequest::new("Application.GetProperties").with_params(Params {
            properties: vec![
                "volume".to_string(),
                "muted".to_string(),
                "name".to_string(),
                "version".to_string(),
            ],
        }))
        .await
    }

    pub async fn set_volume(&self, volume: i32) -> Result<i32, ClientError> {
        #[derive(Serialize)]
        struct Params {
            volume: i32,
        }

        #[derive(Deserialize)]
        struct VolumeResult(i32);

        let result: VolumeResult = self
            .call(JsonRpcRequest::new("Application.SetVolume").with_params(Params { volume }))
            .await?;
        Ok(result.0)
    }

    pub async fn set_mute(&self, mute: bool) -> Result<bool, ClientError> {
        #[derive(Serialize)]
        struct Params {
            mute: bool,
        }

        #[derive(Deserialize)]
        struct MuteResult(bool);

        let result: MuteResult = self
            .call(JsonRpcRequest::new("Application.SetMute").with_params(Params { mute }))
            .await?;
        Ok(result.0)
    }

    pub async fn get_active_players(&self) -> Result<Vec<ActivePlayer>, ClientError> {
        self.call(JsonRpcRequest::new("Player.GetActivePlayers")).await
    }

    pub async fn get_player_properties(
        &self,
        player_id: i32,
    ) -> Result<PlayerProperty, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            properties: Vec<String>,
        }

        self.call(
            JsonRpcRequest::new("Player.GetProperties").with_params(Params {
                playerid: player_id,
                properties: vec![
                    "speed".to_string(),
                    "time".to_string(),
                    "duration".to_string(),
                    "playlistid".to_string(),
                    "currentaudiostream".to_string(),
                    "subtitleenabled".to_string(),
                ],
            }),
        )
        .await
    }

    pub async fn get_current_item(&self, player_id: i32) -> Result<PlayerItem, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            properties: Vec<String>,
        }

        #[derive(Deserialize)]
        struct ItemResult {
            item: PlayerItem,
        }

        let result: ItemResult = self
            .call(
                JsonRpcRequest::new("Player.GetItem").with_params(Params {
                    playerid: player_id,
                    properties: vec![
                        "title".to_string(),
                        "artist".to_string(),
                        "album".to_string(),
                        "showtitle".to_string(),
                        "season".to_string(),
                        "episode".to_string(),
                        "file".to_string(),
                        "thumbnail".to_string(),
                        "fanart".to_string(),
                        "year".to_string(),
                        "runtime".to_string(),
                        "duration".to_string(),
                    ],
                }),
            )
            .await?;
        Ok(result.item)
    }

    pub async fn play_pause(&self, player_id: i32) -> Result<i32, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
        }

        #[derive(Deserialize)]
        struct PlayPauseResult {
            speed: i32,
        }

        let result: PlayPauseResult = self
            .call(
                JsonRpcRequest::new("Player.PlayPause")
                    .with_params(Params { playerid: player_id }),
            )
            .await?;
        Ok(result.speed)
    }

    pub async fn stop(&self, player_id: i32) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
        }

        #[derive(Deserialize)]
        struct StopResult;

        let _: StopResult = self
            .call(
                JsonRpcRequest::new("Player.Stop")
                    .with_params(Params { playerid: player_id }),
            )
            .await?;
        Ok(())
    }

    pub async fn seek(&self, player_id: i32, time_secs: i64) -> Result<PlayerProperty, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            value: i64,
        }

        self.call(
            JsonRpcRequest::new("Player.Seek")
                .with_params(Params { playerid: player_id, value: time_secs }),
        )
        .await
    }

    pub async fn open(&self, file: Option<&str>) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            item: OpenItem,
        }

        #[derive(Serialize)]
        #[serde(untagged)]
        enum OpenItem {
            File { file: String },
            Partymode { partymode: String },
        }

        let params = if let Some(f) = file {
            serde_json::to_value(Params {
                item: OpenItem::File {
                    file: f.to_string(),
                },
            })
            .ok()
        } else {
            serde_json::to_value(Params {
                item: OpenItem::Partymode {
                    partymode: "music".to_string(),
                },
            })
            .ok()
        };

        #[derive(Deserialize)]
        struct OpenResult;

        let _: OpenResult = self
            .call(JsonRpcRequest::new("Player.Open").with_params(params))
            .await?;
        Ok(())
    }

    pub async fn go_to(&self, player_id: i32, to: &str) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            to: String,
        }

        #[derive(Deserialize)]
        struct GoToResult;

        let _: GoToResult = self
            .call(
                JsonRpcRequest::new("Player.GoTo")
                    .with_params(Params { playerid: player_id, to: to.to_string() }),
            )
            .await?;
        Ok(())
    }

    pub async fn get_playlist(&self, playlist_id: i32) -> Result<Vec<PlaylistItem>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playlistid: i32,
            properties: Vec<String>,
        }

        #[derive(Deserialize)]
        struct PlaylistResult {
            items: Vec<PlaylistItem>,
        }

        let result: PlaylistResult = self
            .call(
                JsonRpcRequest::new("Playlist.GetItems")
                    .with_params(Params {
                        playlistid: playlist_id,
                        properties: vec![
                            "title".to_string(),
                            "file".to_string(),
                            "type".to_string(),
                        ],
                    }),
            )
            .await?;
        Ok(result.items)
    }

    pub async fn add_to_playlist(&self, playlist_id: i32, item: &str) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            playlistid: i32,
            item: ItemToAdd,
        }

        #[derive(Serialize)]
        #[serde(untagged)]
        enum ItemToAdd {
            File { file: String },
        }

        #[derive(Deserialize)]
        struct AddResult;

        let _: AddResult = self
            .call(
                JsonRpcRequest::new("Playlist.Add")
                    .with_params(Params {
                        playlistid: playlist_id,
                        item: ItemToAdd::File {
                            file: item.to_string(),
                        },
                    }),
            )
            .await?;
        Ok(())
    }

    pub async fn input(&self, action: InputAction) -> Result<bool, ClientError> {
        #[derive(Deserialize)]
        struct InputResult(bool);

        let result: InputResult = self.call(JsonRpcRequest::new(action.to_string())).await?;
        Ok(result.0)
    }

    pub async fn show_notification(&self, title: &str, message: &str) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            title: String,
            message: String,
            displaytime: i32,
        }

        #[derive(Deserialize)]
        struct NotificationResult;

        let _: NotificationResult = self
            .call(
                JsonRpcRequest::new("GUI.ShowNotification")
                    .with_params(Params {
                        title: title.to_string(),
                        message: message.to_string(),
                        displaytime: 5000,
                    }),
            )
            .await?;
        Ok(())
    }

    pub async fn get_files(
        &self,
        directory: Option<&str>,
        media: &str,
    ) -> Result<Vec<FileItem>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            directory: Option<String>,
            media: String,
            properties: Vec<String>,
        }

        #[derive(Deserialize)]
        struct FilesResult {
            files: Vec<FileItem>,
        }

        let result: FilesResult = self
            .call(
                JsonRpcRequest::new("Files.GetSources").with_params(Params {
                    directory: directory.map(|s| s.to_string()),
                    media: media.to_string(),
                    properties: vec![
                        "title".to_string(),
                        "thumbnail".to_string(),
                        "fanart".to_string(),
                    ],
                }),
            )
            .await?;
        Ok(result.files)
    }

    pub async fn get_movies(
        &self,
        properties: &[&str],
        limits: Option<(i32, i32)>,
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            limits: Option<Limits>,
        }

        #[derive(Serialize)]
        struct Limits {
            start: i32,
            end: i32,
        }

        #[derive(Deserialize)]
        struct MoviesResult {
            movies: Vec<serde_json::Value>,
        }

        let params = if let Some((start, end)) = limits {
            serde_json::to_value(Params {
                properties: properties.iter().map(|s| s.to_string()).collect(),
                limits: Some(Limits { start, end }),
            })
            .ok()
        } else {
            serde_json::to_value(Params {
                properties: properties.iter().map(|s| s.to_string()).collect(),
                limits: None,
            })
            .ok()
        };

        let result: MoviesResult = self
            .call(JsonRpcRequest::new("VideoLibrary.GetMovies").with_params(params))
            .await?;
        Ok(result.movies)
    }

    pub async fn get_tvshows(
        &self,
        properties: &[&str],
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<String>,
        }

        #[derive(Deserialize)]
        struct TvShowsResult {
            tvshows: Vec<serde_json::Value>,
        }

        let result: TvShowsResult = self
            .call(
                JsonRpcRequest::new("VideoLibrary.GetTVShows")
                    .with_params(Params { properties: properties.iter().map(|s| s.to_string()).collect() }),
            )
            .await?;
        Ok(result.tvshows)
    }

    pub async fn get_songs(
        &self,
        properties: &[&str],
        album_id: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            albumid: Option<i32>,
        }

        #[derive(Deserialize)]
        struct SongsResult {
            songs: Vec<serde_json::Value>,
        }

        let result: SongsResult = self
            .call(
                JsonRpcRequest::new("AudioLibrary.GetSongs")
                    .with_params(Params {
                        properties: properties.iter().map(|s| s.to_string()).collect(),
                        albumid: album_id,
                    }),
            )
            .await?;
        Ok(result.songs)
    }

    pub async fn get_albums(&self, properties: &[&str]) -> Result<Vec<serde_json::Value>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<String>,
        }

        #[derive(Deserialize)]
        struct AlbumsResult {
            albums: Vec<serde_json::Value>,
        }

        let result: AlbumsResult = self
            .call(
                JsonRpcRequest::new("AudioLibrary.GetAlbums")
                    .with_params(Params { properties: properties.iter().map(|s| s.to_string()).collect() }),
            )
            .await?;
        Ok(result.albums)
    }

    pub async fn get_favourites(&self) -> Result<Vec<Favourite>, ClientError> {
        #[derive(Deserialize)]
        struct FavouritesResult {
            favourites: Vec<Favourite>,
        }

        let result: FavouritesResult = self.call(JsonRpcRequest::new("Favourites.GetFavourites")).await?;
        Ok(result.favourites)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePlayer {
    pub playerid: i32,
    pub playertype: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favourite {
    pub title: String,
    pub r#type: String,
    pub path: Option<String>,
    pub thumbnail: Option<String>,
    pub window: Option<String>,
    pub windowparameter: Option<String>,
}

#[derive(Debug, Clone, Copy)]
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
            InputAction::Up => write!(f, "Input.Up"),
            InputAction::Down => write!(f, "Input.Down"),
            InputAction::Left => write!(f, "Input.Left"),
            InputAction::Right => write!(f, "Input.Right"),
            InputAction::Select => write!(f, "Input.Select"),
            InputAction::Back => write!(f, "Input.Back"),
            InputAction::Home => write!(f, "Input.Home"),
            InputAction::Info => write!(f, "Input.Info"),
            InputAction::ContextMenu => write!(f, "Input.ContextMenu"),
            InputAction::PreviousMenu => write!(f, "Input.PreviousMenu"),
        }
    }
}
