use crate::host::Host;
use crate::kodi::KodiClient;
use super::BoxFuture;

pub trait KodiEffect: Send + Sync {
    fn connect(&self, host: &Host) -> BoxFuture<'static, Result<KodiClient, String>>;
    fn ping(&self, url: &str) -> BoxFuture<'static, Result<String, String>>;
}

pub struct KodiEffectImpl;

impl KodiEffectImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for KodiEffectImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl KodiEffect for KodiEffectImpl {
    fn connect(&self, host: &Host) -> BoxFuture<'static, Result<KodiClient, String>> {
        let url = host.url();
        Box::pin(async move {
            let client = KodiClient::from_url(&url);
            client.ping().await.map_err(|e| e.to_string())?;
            Ok(client)
        })
    }

    fn ping(&self, url: &str) -> BoxFuture<'static, Result<String, String>> {
        let url = url.to_string();
        Box::pin(async move {
            let client = KodiClient::from_url(&url);
            client.ping().await.map_err(|e| e.to_string())
        })
    }
}
