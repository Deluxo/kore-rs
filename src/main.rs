use gtk::prelude::*;
use gtk::Application;

mod kodi;
mod host;

pub use host::Host;
pub use host::manager::HostManager;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Korers");

    let app = Application::builder()
        .application_id("org.korers.app")
        .build();
    
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Korers - Kodi Remote")
            .default_width(800)
            .default_height(600)
            .build();
    
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let header = gtk::HeaderBar::builder()
            .title_widget(&gtk::Label::new(Some("Korers")))
            .show_title_buttons(true)
            .build();

        let placeholder = gtk::Label::new(Some("Welcome to Korers\n\nA Rust desktop remote for Kodi"));

        vbox.append(&header);
        vbox.append(&placeholder);

        window.set_child(Some(&vbox));
        window.show();
    });
    
    app.run();
}
