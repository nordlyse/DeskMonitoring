use gtk::gdk;
use gtk::prelude::*;
use gtk::{Application, CssProvider};

use crate::setup;
use crate::theme;

pub fn on_activate(app: &Application) {
    insert_stylesheet();
    if app.windows().is_empty() {
        setup::show_setup(app);
    }
}

fn insert_stylesheet() {
    let provider = CssProvider::new();
    provider.load_from_data(theme::window_css());
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}
