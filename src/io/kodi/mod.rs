pub mod actions;
pub mod client;
pub mod discovery;
pub mod types;

use std::cell::RefCell;
use std::rc::Rc;

pub use client::KodiClient;
pub use discovery::DiscoveryService;

/// Kodi client owned by the GTK thread.
///
/// MPRIS never touches this directly: it queues commands that the local ticker
/// executes, so all Kodi I/O stays on the existing on-demand path.
pub type ClientCell = Rc<RefCell<Option<KodiClient>>>;

/// Run `f` with the current client, if one is connected.
pub fn with_client<R>(cell: &ClientCell, f: impl FnOnce(&KodiClient) -> R) -> Option<R> {
    let cell = cell.try_borrow().ok()?;
    cell.as_ref().map(f)
}
