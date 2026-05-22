use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create tokio runtime"))
}

/// Execute a future synchronously using the shared tokio runtime.
/// This is the only place we should call block_on in the entire app.
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    get().block_on(future)
}
