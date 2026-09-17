mod app;
mod config;
mod hud;
mod metrics;
mod overlay;
#[cfg(target_os = "macos")]
mod macos_place;
mod setup;
mod snapshot;
mod theme;
mod weather_fx;
mod window;

use gtk::prelude::*;

fn main() {
    prepare_macos_bundle();
    if std::env::var_os("GSK_RENDERER").is_none() {
        std::env::set_var("GSK_RENDERER", "cairo");
    }
    let application = gtk::Application::builder()
        .application_id("org.nordlyse.DeskMonitoring")
        .build();
    application.connect_activate(app::on_activate);
    application.run();
}

fn prepare_macos_bundle() {
    #[cfg(target_os = "macos")]
    {
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let Some(macos_dir) = exe.parent() else {
            return;
        };
        let Some(contents) = macos_dir.parent() else {
            return;
        };
        if contents.file_name().and_then(|name| name.to_str()) != Some("Contents") {
            return;
        }
        let resources = contents.join("Resources");
        let schemas = resources.join("glib-2.0/schemas");
        if schemas.is_dir() {
            std::env::set_var("GSETTINGS_SCHEMA_DIR", &schemas);
        }
        let pixbuf_cache = resources.join("gdk-pixbuf-2.0/loaders.cache");
        if pixbuf_cache.is_file() {
            std::env::set_var("GDK_PIXBUF_MODULE_FILE", &pixbuf_cache);
        }
        if let Some(loaders) = first_dir(&resources.join("gdk-pixbuf-2.0"), "loaders") {
            std::env::set_var("GDK_PIXBUF_MODULEDIR", loaders);
        }
    }
}

#[cfg(target_os = "macos")]
fn first_dir(root: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = first_dir(&path, name) {
                return Some(found);
            }
        }
    }
    None
}
