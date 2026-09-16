use std::sync::{Arc, Mutex};

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CheckButton, Entry, Label, Orientation};

use crate::config::{load_config, write_config, Config, PaletteKind, Panels, Position};
use crate::overlay;
use crate::window::show_monitor;

pub fn show_setup(app: &Application) {
    show_settings(app, None);
}

pub fn show_settings(app: &Application, live: Option<Arc<Mutex<Config>>>) {
    for existing in app.windows() {
        if existing.has_css_class("desk-setup") {
            existing.present();
            return;
        }
    }

    let config = live
        .as_ref()
        .and_then(|slot| slot.lock().ok().map(|guard| guard.clone()))
        .or_else(load_config)
        .unwrap_or_default();
    let monitor_open = find_monitor(app).is_some();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desk Monitor settings")
        .default_width(520)
        .default_height(780)
        .resizable(true)
        .build();
    window.add_css_class("desk-setup");

    let root = gtk::Box::new(Orientation::Vertical, 12);
    root.set_margin_top(20);
    root.set_margin_bottom(20);
    root.set_margin_start(24);
    root.set_margin_end(24);

    let title = Label::new(Some("Settings"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    root.append(&title);

    let intro = Label::new(Some(
        "Change color and position any time. Uncheck a panel to hide it. Fill mail or calendar later, or clear those fields to drop them.",
    ));
    intro.set_wrap(true);
    intro.set_halign(gtk::Align::Start);
    root.append(&intro);

    root.append(&section_label("Panel position"));
    let pos_box = gtk::Box::new(Orientation::Vertical, 4);
    let pos_left = CheckButton::with_label("Left");
    let pos_right = CheckButton::with_label("Right");
    let pos_top = CheckButton::with_label("Top");
    let pos_bottom = CheckButton::with_label("Bottom");
    let pos_center = CheckButton::with_label("Center");
    pos_right.set_group(Some(&pos_left));
    pos_top.set_group(Some(&pos_left));
    pos_bottom.set_group(Some(&pos_left));
    pos_center.set_group(Some(&pos_left));
    match config.position {
        Position::Left => pos_left.set_active(true),
        Position::Right => pos_right.set_active(true),
        Position::Top => pos_top.set_active(true),
        Position::Bottom => pos_bottom.set_active(true),
        Position::Center => pos_center.set_active(true),
    }
    for btn in [&pos_left, &pos_right, &pos_top, &pos_bottom, &pos_center] {
        pos_box.append(btn);
    }
    root.append(&pos_box);

    root.append(&section_label("Color palette"));
    let pal_box = gtk::Box::new(Orientation::Vertical, 4);
    let pal_matrix = CheckButton::with_label("Matrix green");
    let pal_turquoise = CheckButton::with_label("Turquoise");
    let pal_blue = CheckButton::with_label("Blue");
    let pal_pink = CheckButton::with_label("Pink");
    let pal_yellow = CheckButton::with_label("Yellow");
    pal_turquoise.set_group(Some(&pal_matrix));
    pal_blue.set_group(Some(&pal_matrix));
    pal_pink.set_group(Some(&pal_matrix));
    pal_yellow.set_group(Some(&pal_matrix));
    match config.palette {
        PaletteKind::Matrix => pal_matrix.set_active(true),
        PaletteKind::Turquoise => pal_turquoise.set_active(true),
        PaletteKind::Blue => pal_blue.set_active(true),
        PaletteKind::Pink => pal_pink.set_active(true),
        PaletteKind::Yellow => pal_yellow.set_active(true),
    }
    for btn in [
        &pal_matrix,
        &pal_turquoise,
        &pal_blue,
        &pal_pink,
        &pal_yellow,
    ] {
        pal_box.append(btn);
    }
    root.append(&pal_box);

    root.append(&section_label("Visible panels"));
    let panel_cpu = CheckButton::with_label("CPU");
    let panel_ram = CheckButton::with_label("Memory");
    let panel_disk = CheckButton::with_label("Disk");
    let panel_net = CheckButton::with_label("Network");
    let panel_mail = CheckButton::with_label("Mail");
    let panel_cal = CheckButton::with_label("Calendar");
    let panel_wx = CheckButton::with_label("Weather");
    panel_cpu.set_active(config.panels.cpu);
    panel_ram.set_active(config.panels.ram);
    panel_disk.set_active(config.panels.disk);
    panel_net.set_active(config.panels.network);
    panel_mail.set_active(config.panels.mail);
    panel_cal.set_active(config.panels.calendar);
    panel_wx.set_active(config.panels.weather);
    for btn in [
        &panel_cpu,
        &panel_ram,
        &panel_disk,
        &panel_net,
        &panel_mail,
        &panel_cal,
        &panel_wx,
    ] {
        root.append(btn);
    }

    root.append(&section_label("Mail IMAP (optional, leave host empty to drop)"));
    let imap_host = add_field(&root, "Host", "imap.example.com", &config.imap_host);
    let imap_port = add_field(&root, "Port", "993", &config.imap_port.to_string());
    let imap_user = add_field(&root, "Username", "you@example.com", &config.imap_user);
    let imap_pass = add_field(&root, "Password", "", &config.imap_password);
    imap_pass.set_visibility(false);
    let imap_inbox = add_field(&root, "Inbox mailbox", "INBOX", &config.imap_inbox);
    let imap_sent = add_field(&root, "Sent mailbox", "Sent", &config.imap_sent);

    root.append(&section_label("Calendar .ics path (optional, leave empty to drop)"));
    let calendar = add_field(
        &root,
        "File or folder",
        "/path/to/calendar.ics",
        &config.calendar_ics,
    );

    let error = Label::new(None);
    error.add_css_class("error");
    error.set_halign(gtk::Align::Start);
    root.append(&error);

    let start = gtk::Button::with_label(if monitor_open {
        "Apply settings"
    } else {
        "Start monitoring"
    });
    start.add_css_class("suggested-action");
    root.append(&start);

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&root));
    window.set_child(Some(&scroll));

    start.connect_clicked(clone!(
        #[weak] window,
        #[weak] pos_left,
        #[weak] pos_right,
        #[weak] pos_top,
        #[weak] pos_bottom,
        #[weak] pos_center,
        #[weak] pal_matrix,
        #[weak] pal_turquoise,
        #[weak] pal_blue,
        #[weak] pal_pink,
        #[weak] pal_yellow,
        #[weak] panel_cpu,
        #[weak] panel_ram,
        #[weak] panel_disk,
        #[weak] panel_net,
        #[weak] panel_mail,
        #[weak] panel_cal,
        #[weak] panel_wx,
        #[weak] imap_host,
        #[weak] imap_port,
        #[weak] imap_user,
        #[weak] imap_pass,
        #[weak] imap_inbox,
        #[weak] imap_sent,
        #[weak] calendar,
        #[weak] error,
        move |_| {
            let position = if pos_left.is_active() {
                Position::Left
            } else if pos_right.is_active() {
                Position::Right
            } else if pos_top.is_active() {
                Position::Top
            } else if pos_bottom.is_active() {
                Position::Bottom
            } else if pos_center.is_active() {
                Position::Center
            } else {
                Position::Right
            };
            let palette = if pal_matrix.is_active() {
                PaletteKind::Matrix
            } else if pal_turquoise.is_active() {
                PaletteKind::Turquoise
            } else if pal_blue.is_active() {
                PaletteKind::Blue
            } else if pal_pink.is_active() {
                PaletteKind::Pink
            } else if pal_yellow.is_active() {
                PaletteKind::Yellow
            } else {
                PaletteKind::Matrix
            };
            let port = imap_port.text().parse::<u16>().unwrap_or(993);
            let config = Config {
                position,
                palette,
                panels: Panels {
                    cpu: panel_cpu.is_active(),
                    ram: panel_ram.is_active(),
                    disk: panel_disk.is_active(),
                    network: panel_net.is_active(),
                    mail: panel_mail.is_active(),
                    calendar: panel_cal.is_active(),
                    weather: panel_wx.is_active(),
                },
                imap_host: imap_host.text().to_string(),
                imap_port: port,
                imap_user: imap_user.text().to_string(),
                imap_password: imap_pass.text().to_string(),
                imap_inbox: imap_inbox.text().to_string(),
                imap_sent: imap_sent.text().to_string(),
                calendar_ics: calendar.text().to_string(),
            };
            if let Err(err) = write_config(&config) {
                error.set_text(&format!("Could not write settings: {err}"));
                return;
            }
            let Some(app) = window.application() else {
                return;
            };
            if let Some(slot) = &live {
                if let Ok(mut guard) = slot.lock() {
                    *guard = config.clone();
                }
            }
            if let Some(monitor) = find_monitor(&app) {
                overlay::place_overlay(&monitor, &config);
                monitor.queue_draw();
                window.close();
                return;
            }
            window.close();
            show_monitor(&app, config);
        }
    ));

    window.present();
}

fn find_monitor(app: &Application) -> Option<ApplicationWindow> {
    for window in app.windows() {
        if window.has_css_class("desk-monitor") {
            return window.downcast::<ApplicationWindow>().ok();
        }
    }
    None
}

fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_halign(gtk::Align::Start);
    label.add_css_class("heading");
    label.set_margin_top(8);
    label
}

fn add_field(root: &gtk::Box, label: &str, hint: &str, initial: &str) -> Entry {
    let caption = Label::new(Some(label));
    caption.set_halign(gtk::Align::Start);
    root.append(&caption);
    let entry = Entry::new();
    entry.set_placeholder_text(Some(hint));
    if !initial.is_empty() {
        entry.set_text(initial);
    }
    root.append(&entry);
    entry
}
