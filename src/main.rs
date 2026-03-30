mod app;
mod effects;
mod host;
mod kodi;
mod ui;

use app::App;

fn main() {
    App::new()
        .init_logging()
        .load_hosts()
        .show_window()
        .show_host_selection()
        .run();
}
