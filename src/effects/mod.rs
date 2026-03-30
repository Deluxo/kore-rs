mod discovery;
mod kodi;

pub use discovery::DiscoveryEffect;
pub use kodi::KodiEffect;

use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct Effects {
    pub discovery: Box<dyn DiscoveryEffect>,
    pub kodi: Box<dyn KodiEffect>,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            discovery: Box::new(discovery::DiscoveryEffectImpl::new()),
            kodi: Box::new(kodi::KodiEffectImpl),
        }
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}
