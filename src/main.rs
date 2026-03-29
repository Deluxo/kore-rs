use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};

mod kodi;
mod host;
mod ui;

pub use kodi::client::KodiClient;
pub use kodi::types::*;

pub use host::Host;
pub use host::manager::HostManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMsg {
    Quit,
}

struct AppModel;

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("Korers - Kodi Remote")
            .default_width(800)
            .default_height(600)
            .build()
    }

    fn init(
        _init: Self::Init,
        window: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        let header = gtk::HeaderBar::builder()
            .title_widget(&gtk::Label::new(Some("Korers")))
            .show_title_buttons(true)
            .build();

        let placeholder = gtk::Label::new(Some("Welcome to Korers\n\nA Rust desktop remote for Kodi"));
        placeholder.set_margin_all(20);

        vbox.append(&header);
        vbox.append(&placeholder);

        window.set_child(Some(&vbox));

        ComponentParts { model: AppModel, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Quit => {
                tracing::info!("Quit requested");
            }
        }
    }

    fn update_view(&self, _widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {}
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Korers");

    let app = RelmApp::new("org.korers.app");
    app.run::<AppModel>(());
}
