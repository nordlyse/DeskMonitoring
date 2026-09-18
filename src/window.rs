use std::sync::{Arc, Mutex};
use std::time::Duration;

use gtk::gdk;
use gtk::glib::{self, ControlFlow};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea, Orientation};

use crate::config::{self, Config};
use crate::hud;
use crate::metrics::{self, Collector};
use crate::overlay;
use crate::setup;
use crate::snapshot::Snapshot;

pub fn show_monitor(app: &Application, config: Config) {
    for window in app.windows() {
        if window.has_css_class("desk-monitor") {
            window.present();
            return;
        }
    }

    config::store_live(&config);
    let live = config::live_slot();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desk Monitor")
        .build();
    overlay::apply_window_chrome(&window);
    overlay::place_overlay(&window, &config);

    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    let mut collector = Collector::add();
    collector.refresh_local(&snapshot);
    metrics::spawn_remote_loop(live.clone(), snapshot.clone());

    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.add_css_class("desk-monitor-canvas");

    let draw_live = live.clone();
    let draw_snapshot = snapshot.clone();
    area.set_draw_func(move |_, cr, width, height| {
        let snap = metrics::copy_snapshot(&draw_snapshot);
        let config = draw_live.lock().map(|guard| guard.clone()).unwrap_or_default();
        hud::paint(cr, width, height, &snap, &config);
    });

    let host = gtk::Box::new(Orientation::Vertical, 0);
    host.add_css_class("desk-monitor-host");
    host.append(&area);
    window.set_child(Some(&host));
    attach_pointer(&window, &area, live.clone());

    let metric_area = area.clone();
    let metric_snapshot = snapshot.clone();
    let mut collector = collector;
    let mut ticks = 0u32;
    glib::timeout_add_local(Duration::from_millis(33), move || {
        ticks = ticks.wrapping_add(1);
        if ticks % 30 == 0 {
            collector.refresh_local(&metric_snapshot);
        }
        metric_area.queue_draw();
        ControlFlow::Continue
    });

    window.present();
    #[cfg(target_os = "macos")]
    crate::macos_dock::install_dock_menu();
}

pub fn refresh_overlay(app: &Application) {
    let config = config::current_config();
    for window in app.windows() {
        if window.has_css_class("desk-monitor") {
            if let Ok(monitor) = window.downcast::<ApplicationWindow>() {
                overlay::place_overlay(&monitor, &config);
                monitor.queue_draw();
            }
        }
    }
}

fn attach_pointer(window: &ApplicationWindow, area: &DrawingArea, live: Arc<Mutex<Config>>) {
    let menu_model = crate::options::options_menu();
    let popover = gtk::PopoverMenu::from_model(Some(&menu_model));
    popover.set_parent(area);
    popover.set_has_arrow(false);

    let drag = gtk::GestureClick::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    let window_weak = window.downgrade();
    let area_click = area.clone();
    let live_click = live.clone();
    let popover_ctrl = popover.clone();
    drag.connect_pressed(move |gesture, n_press, x, y| {
        if n_press != 1 {
            return;
        }
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if gesture
            .current_event_state()
            .contains(gdk::ModifierType::CONTROL_MASK)
        {
            popover_ctrl.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover_ctrl.popup();
            return;
        }
        let config = live_click.lock().map(|guard| guard.clone()).unwrap_or_default();
        if hud::hit_settings(x, y, area_click.width(), config.position) {
            if let Some(app) = window.application() {
                setup::show_settings(&app, Some(live_click.clone()));
            }
            return;
        }
        let Some(native) = window.native() else {
            return;
        };
        let Some(surface) = native.surface() else {
            return;
        };
        let Ok(toplevel) = surface.downcast::<gdk::Toplevel>() else {
            return;
        };
        let Some(device) = gesture.device() else {
            return;
        };
        toplevel.begin_move(&device, gdk::BUTTON_PRIMARY as i32, x, y, gdk::CURRENT_TIME);
    });
    area.add_controller(drag);

    let menu = gtk::GestureClick::new();
    menu.set_button(gdk::BUTTON_SECONDARY);
    let popover_click = popover.clone();
    menu.connect_pressed(move |_, n_press, x, y| {
        if n_press != 1 {
            return;
        }
        popover_click.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover_click.popup();
    });
    area.add_controller(menu);
}
