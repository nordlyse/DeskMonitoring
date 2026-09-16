use gtk::gdk;
use gtk::gio::{self, prelude::ActionMapExt};
use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, CssProvider};

use crate::config::{self, load_config};
use crate::setup;
use crate::theme;
use crate::window;

pub fn on_activate(app: &Application) {
    insert_stylesheet();
    install_app_actions(app);
    if !app.windows().is_empty() {
        return;
    }
    match load_config() {
        Some(config) => {
            config::store_live(&config);
            window::show_monitor(app, config);
        }
        None => setup::show_setup(app),
    }
}

fn install_app_actions(app: &Application) {
    if app.lookup_action("settings").is_some() {
        return;
    }

    let settings = gio::SimpleAction::new("settings", None);
    let app_weak = app.downgrade();
    settings.connect_activate(move |_, _| {
        let Some(app) = app_weak.upgrade() else {
            return;
        };
        setup::show_settings(&app, Some(config::live_slot()));
    });
    app.add_action(&settings);
    app.set_accels_for_action("app.settings", &["<Meta>comma", "<Control>comma"]);

    let palette = gio::SimpleAction::new("set-palette", Some(glib::VariantTy::STRING));
    let app_weak = app.downgrade();
    palette.connect_activate(move |_, value| {
        let Some(name) = value.and_then(|item| item.str().map(str::to_string)) else {
            return;
        };
        let kind = match name.as_str() {
            "turquoise" => config::PaletteKind::Turquoise,
            "blue" => config::PaletteKind::Blue,
            "pink" => config::PaletteKind::Pink,
            "yellow" => config::PaletteKind::Yellow,
            _ => config::PaletteKind::Matrix,
        };
        let mut cfg = config::current_config();
        cfg.palette = kind;
        let _ = config::write_config(&cfg);
        if let Some(app) = app_weak.upgrade() {
            window::refresh_overlay(&app);
        }
    });
    app.add_action(&palette);

    let menu = gio::Menu::new();
    menu.append(Some("Settings"), Some("app.settings"));
    let colors = gio::Menu::new();
    colors.append(Some("Matrix green"), Some("app.set-palette::matrix"));
    colors.append(Some("Turquoise"), Some("app.set-palette::turquoise"));
    colors.append(Some("Blue"), Some("app.set-palette::blue"));
    colors.append(Some("Pink"), Some("app.set-palette::pink"));
    colors.append(Some("Yellow"), Some("app.set-palette::yellow"));
    menu.append_submenu(Some("Color"), &colors);
    app.set_menubar(Some(&menu));
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
