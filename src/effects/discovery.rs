use crate::kodi::discovery::DiscoveryService;
use crate::kodi::types::HostInfo;
use super::BoxFuture;

pub trait DiscoveryEffect: Send + Sync {
    fn discover(&self, timeout_secs: u64) -> BoxFuture<'static, Result<Vec<HostInfo>, String>>;
}

pub struct DiscoveryEffectImpl {
    service: DiscoveryService,
}

impl DiscoveryEffectImpl {
    pub fn new() -> Self {
        Self {
            service: DiscoveryService::new(),
        }
    }
}

impl Default for DiscoveryEffectImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryEffect for DiscoveryEffectImpl {
    fn discover(&self, timeout_secs: u64) -> BoxFuture<'static, Result<Vec<HostInfo>, String>> {
        let service = DiscoveryService::new();
        Box::pin(async move {
            service.discover_all(timeout_secs).map_err(|e| e.to_string())
        })
    }
}
