mod app;
mod config;
mod hud;
mod metrics;
mod overlay;
mod setup;
mod snapshot;
mod theme;
mod window;

use gtk::prelude::*;

fn main() {
    let application = gtk::Application::builder()
        .application_id("org.nordlyse.DeskMonitoring")
        .build();
    application.connect_activate(app::on_activate);
    application.run();
}
