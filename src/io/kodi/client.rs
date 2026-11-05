use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::types::*;
use crate::io::host::Host;

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
    #[error("Request failed: {0}")]
    Request(String),
}

pub struct KodiClient {
    client: Client,
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl KodiClient {
    pub fn new(host: &HostInfo) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: host.url(),
            username: None,
            password: None,
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
            username: None,
            password: None,
        }
    }

    pub fn from_host(host: &Host) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(5))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: format!("http://{}:{}", host.address, host.port),
            username: host.username.clone(),
            password: host.password.clone(),
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn call<R>(&self, request: JsonRpcRequest) -> Result<R, ClientError>
    where
        R: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/jsonrpc", self.base_url);
        tracing::debug!("Kodi request: {} {}", request.method, url);

        let body = serde_json::to_value(&request)?;
        tracing::debug!("Request body: {}", body);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request);

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            let credentials = format!("{}:{}", user, pass);
            let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
            req = req.header("Authorization", format!("Basic {}", encoded));
        }

        let resp = req.send().await?;

        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            tracing::warn!("Kodi HTTP error: {} - {}", status, body_text);
            return Err(ClientError::Request(format!(
                "HTTP {}: {}",
                status, body_text
            )));
        }

        tracing::debug!("Response body: {}", body_text);

        let response: JsonRpcResponse<R> = serde_json::from_str(&body_text)?;

        if let Some(error) = response.error {
            return Err(ClientError::Kodi {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| ClientError::Request("No result in response".to_string()))
    }

    pub async fn ping(&self) -> Result<String, ClientError> {
        let resp: String = self
            .call(JsonRpcRequest::new("JSONRPC.Ping"))
            .await?;
        Ok(resp)
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<&'static str>,
        }

        self.call(
            JsonRpcRequest::new("System.GetInfo")
                .with_params(Params {
                    properties: vec!["name", "version", "build_date", "hostname", "os"],
                }),
        )
        .await
    }

    pub async fn get_application_properties(&self) -> Result<ApplicationProperty, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<&'static str>,
        }

        self.call(
            JsonRpcRequest::new("Application.GetProperties")
                .with_params(Params {
                    properties: vec!["volume", "muted", "name", "version"],
                }),
        )
        .await
    }

    pub async fn set_volume(&self, volume: i32) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            volume: i32,
        }
        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Application.SetVolume").with_params(Params { volume }))
            .await?;
        Ok(())
    }

    pub async fn set_mute(&self, mute: bool) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            mute: bool,
        }
        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Application.SetMute").with_params(Params { mute }))
            .await?;
        Ok(())
    }

    pub async fn get_active_players(&self) -> Result<Vec<ActivePlayer>, ClientError> {
        self.call(JsonRpcRequest::new("Player.GetActivePlayers"))
            .await
    }

    pub async fn get_player_properties(
        &self,
        player_id: i32,
    ) -> Result<PlayerProperty, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            properties: Vec<&'static str>,
        }

        self.call(
            JsonRpcRequest::new("Player.GetProperties").with_params(Params {
                playerid: player_id,
                properties: vec![
                    "speed",
                    "time",
                    "totaltime",
                    "playlistid",
                    "currentaudiostream",
                    "subtitleenabled",
                ],
            }),
        )
        .await
    }

    pub async fn get_current_item(&self, player_id: i32) -> Result<PlayerItem, ClientError> {
        let params = serde_json::json!({
            "playerid": player_id,
            "properties": ["title", "file", "thumbnail", "art", "plot", "tagline", "artist", "album", "showtitle", "season", "episode"]
        });

        let resp: GetItemResponse = self
            .call(JsonRpcRequest::new("Player.GetItem").with_params(params))
            .await?;
        Ok(resp.item)
    }

    pub async fn play_pause(&self, player_id: i32) -> Result<i32, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            play: &'static str,
        }

        #[derive(Deserialize)]
        struct Response {
            speed: i32,
        }

        let resp: Response = self
            .call(
                JsonRpcRequest::new("Player.PlayPause").with_params(Params {
                    playerid: player_id,
                    play: "toggle",
                }),
            )
            .await?;
        Ok(resp.speed)
    }

    pub async fn play_pause_with(&self, player_id: i32, play: bool) -> Result<i32, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            play: bool,
        }

        #[derive(Deserialize)]
        struct Response {
            speed: i32,
        }

        let resp: Response = self
            .call(
                JsonRpcRequest::new("Player.PlayPause").with_params(Params {
                    playerid: player_id,
                    play,
                }),
            )
            .await?;
        Ok(resp.speed)
    }

    pub async fn stop(&self, player_id: i32) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
        }

        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Player.Stop").with_params(Params {
                playerid: player_id,
            }))
            .await?;
        Ok(())
    }

    pub async fn seek(&self, player_id: i32, time_secs: i64) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Player.Seek").with_params(
                serde_json::json!({
                    "playerid": player_id,
                    "value": {
                        "time": {
                            "hours": time_secs / 3600,
                            "minutes": (time_secs % 3600) / 60,
                            "seconds": time_secs % 60,
                            "milliseconds": 0,
                        }
                    },
                }),
            ))
            .await?;
        Ok(())
    }

    pub async fn seek_percentage(
        &self,
        player_id: i32,
        percentage: f64,
    ) -> Result<(), ClientError> {
        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Player.Seek").with_params(
                serde_json::json!({
                    "playerid": player_id,
                    "value": {"percentage": percentage},
                }),
            ))
            .await?;
        Ok(())
    }

    pub async fn open(&self, file: Option<&str>) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct File {
            file: String,
        }

        #[derive(Serialize)]
        struct Params {
            item: File,
            options: Option<serde_json::Value>,
        }

        let _: serde_json::Value = self
            .call(JsonRpcRequest::new("Player.Open").with_params(Params {
                item: File {
                    file: file.unwrap_or("").to_string(),
                },
                options: None,
            }))
            .await?;
        Ok(())
    }

    pub async fn go_to(&self, player_id: i32, to: &str) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            playerid: i32,
            to: String,
        }

        let _: serde_json::Value = self
            .call(
                JsonRpcRequest::new("Player.GoTo").with_params(Params {
                    playerid: player_id,
                    to: to.to_string(),
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn get_playlist(&self, playlist_id: i32) -> Result<Vec<PlaylistItem>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            playlistid: i32,
            properties: Vec<&'static str>,
        }

        self.call(
            JsonRpcRequest::new("Playlist.GetItems").with_params(Params {
                playlistid: playlist_id,
                properties: vec!["id", "type", "title", "file"],
            }),
        )
        .await
    }

    pub async fn add_to_playlist(
        &self,
        playlist_id: i32,
        item: &str,
    ) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Item {
            file: String,
        }

        #[derive(Serialize)]
        struct Params {
            playlistid: i32,
            item: Item,
        }

        let _: serde_json::Value = self
            .call(
                JsonRpcRequest::new("Playlist.Add").with_params(Params {
                    playlistid: playlist_id,
                    item: Item {
                        file: item.to_string(),
                    },
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn show_notification(
        &self,
        title: &str,
        message: &str,
    ) -> Result<(), ClientError> {
        #[derive(Serialize)]
        struct Params {
            title: String,
            message: String,
        }

        let _: serde_json::Value = self
            .call(
                JsonRpcRequest::new("GUI.ShowNotification").with_params(Params {
                    title: title.to_string(),
                    message: message.to_string(),
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn input(&self, action: InputAction) -> Result<(), ClientError> {
        let method = format!("Input.{}", action);
        let _: serde_json::Value = self
            .call(JsonRpcRequest::new(method))
            .await?;
        Ok(())
    }

    pub async fn get_files(
        &self,
        media: &str,
    ) -> Result<Vec<FileItem>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            media: String,
        }

        self.call(
            JsonRpcRequest::new("Files.GetSources").with_params(Params {
                media: media.to_string(),
            }),
        )
        .await
    }

    pub async fn get_movies(
        &self,
        properties: &[&str],
        limits: Option<(i32, i32)>,
        sort: Option<(&str, &str)>,
    ) -> Result<serde_json::Value, ClientError> {
        #[derive(Serialize)]
        struct Params<'a> {
            properties: &'a [&'a str],
            #[serde(skip_serializing_if = "Option::is_none")]
            limits: Option<serde_json::Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            sort: Option<serde_json::Value>,
        }

        let limits_val = limits.map(|(start, end)| {
            serde_json::json!({ "start": start, "end": end })
        });

        let sort_val = sort.map(|(method, order)| {
            serde_json::json!({ "method": method, "order": order })
        });

        self.call(
            JsonRpcRequest::new("VideoLibrary.GetMovies").with_params(Params {
                properties,
                limits: limits_val,
                sort: sort_val,
            }),
        )
        .await
    }

    pub async fn get_tvshows(
        &self,
        properties: &[&str],
        limits: Option<(i32, i32)>,
    ) -> Result<serde_json::Value, ClientError> {
        #[derive(Serialize)]
        struct Params<'a> {
            properties: &'a [&'a str],
            #[serde(skip_serializing_if = "Option::is_none")]
            limits: Option<serde_json::Value>,
        }

        let limits_val = limits.map(|(start, end)| {
            serde_json::json!({ "start": start, "end": end })
        });

        self.call(
            JsonRpcRequest::new("VideoLibrary.GetTVShows").with_params(Params {
                properties,
                limits: limits_val,
            }),
        )
        .await
    }

    pub async fn get_songs(
        &self,
        properties: &[&str],
        limits: Option<(i32, i32)>,
    ) -> Result<serde_json::Value, ClientError> {
        #[derive(Serialize)]
        struct Params<'a> {
            properties: &'a [&'a str],
            #[serde(skip_serializing_if = "Option::is_none")]
            limits: Option<serde_json::Value>,
        }

        let limits_val = limits.map(|(start, end)| {
            serde_json::json!({ "start": start, "end": end })
        });

        self.call(
            JsonRpcRequest::new("AudioLibrary.GetSongs").with_params(Params {
                properties,
                limits: limits_val,
            }),
        )
        .await
    }

    pub async fn get_albums(
        &self,
        properties: &[&str],
    ) -> Result<Vec<serde_json::Value>, ClientError> {
        #[derive(Serialize)]
        struct Params<'a> {
            properties: &'a [&'a str],
        }

        self.call(
            JsonRpcRequest::new("AudioLibrary.GetAlbums").with_params(Params {
                properties,
            }),
        )
        .await
    }

    pub async fn get_favourites(&self) -> Result<Vec<Favourite>, ClientError> {
        #[derive(Serialize)]
        struct Params {
            properties: Vec<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            limits: Option<serde_json::Value>,
        }

        self.call(
            JsonRpcRequest::new("Favourites.GetFavourites").with_params(Params {
                properties: vec!["id", "type", "title", "path", "thumbnail", "window", "windowparameter"],
                limits: None,
            }),
        )
        .await
    }

    pub async fn prepare_download(&self, path: &str) -> Result<String, ClientError> {
        #[derive(Serialize)]
        struct Params {
            path: String,
        }

        #[derive(Deserialize)]
        struct Response {
            path: Option<String>,
            mode: Option<String>,
            protocol: Option<String>,
            details: Option<serde_json::Value>,
        }

        let resp: Response = self
            .call(
                JsonRpcRequest::new("Files.PrepareDownload").with_params(Params {
                    path: path.to_string(),
                }),
            )
            .await?;

        Ok(resp.path.unwrap_or_default())
    }

    pub async fn get_thumbnail(&self, url: &str) -> Result<Vec<u8>, ClientError> {
        let is_remote = url.contains("fanart.tv") || url.contains("assets.fanart.tv") || url.contains("https://");

        if is_remote {
            let resp = self.client.get(url).send().await?;
            let bytes = resp.bytes().await?;
            Ok(bytes.to_vec())
        } else if url.starts_with("image://") {
            let download_path = self.prepare_download(url).await?;
            let thumb_url = format!("{}/{}", self.base_url, download_path);
            let mut req = self.client.get(&thumb_url);
            if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                let credentials = format!("{}:{}", user, pass);
                let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
                req = req.header("Authorization", format!("Basic {}", encoded));
            }
            let resp = req.send().await?;
            let bytes = resp.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let resp = self.client.get(url).send().await?;
            let bytes = resp.bytes().await?;
            Ok(bytes.to_vec())
        }
    }
}
