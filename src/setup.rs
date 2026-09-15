use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CheckButton, Entry, Label, Orientation};

use crate::config::{write_config, Config, PaletteKind, Position};
use crate::window::show_monitor;

pub fn show_setup(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desk Monitor setup")
        .default_width(520)
        .default_height(720)
        .resizable(true)
        .build();
    window.add_css_class("desk-setup");

    let root = gtk::Box::new(Orientation::Vertical, 12);
    root.set_margin_top(20);
    root.set_margin_bottom(20);
    root.set_margin_start(24);
    root.set_margin_end(24);

    let title = Label::new(Some("Desk Monitor"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    root.append(&title);

    let intro = Label::new(Some(
        "Choose panel position and color palette. Mail and calendar are optional.",
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
    pos_right.set_active(true);
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
    pal_matrix.set_active(true);
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

    root.append(&section_label("Mail IMAP (optional)"));
    let imap_host = add_field(&root, "Host", "imap.example.com", "");
    let imap_port = add_field(&root, "Port", "993", "993");
    let imap_user = add_field(&root, "Username", "you@example.com", "");
    let imap_pass = add_field(&root, "Password", "", "");
    imap_pass.set_visibility(false);
    let imap_inbox = add_field(&root, "Inbox mailbox", "INBOX", "INBOX");
    let imap_sent = add_field(&root, "Sent mailbox", "Sent", "Sent");

    root.append(&section_label("Calendar .ics path (optional)"));
    let calendar = add_field(&root, "File or folder", "/path/to/calendar.ics", "");

    let error = Label::new(None);
    error.add_css_class("error");
    error.set_halign(gtk::Align::Start);
    root.append(&error);

    let start = gtk::Button::with_label("Start monitoring");
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
            if let Some(app) = window.application() {
                window.close();
                show_monitor(&app, config);
            }
        }
    ));

    window.present();
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
