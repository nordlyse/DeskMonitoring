use gtk::gio;

pub fn options_menu() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(Some("Settings"), Some("app.settings"));
    menu.append_section(Some("Color"), &color_section());
    menu
}

pub fn menubar() -> gio::Menu {
    let bar = gio::Menu::new();
    bar.append_submenu(Some("Options"), &options_menu());
    bar
}

fn color_section() -> gio::Menu {
    let colors = gio::Menu::new();
    colors.append(Some("Matrix green"), Some("app.set-palette::matrix"));
    colors.append(Some("Turquoise"), Some("app.set-palette::turquoise"));
    colors.append(Some("Blue"), Some("app.set-palette::blue"));
    colors.append(Some("Pink"), Some("app.set-palette::pink"));
    colors.append(Some("Yellow"), Some("app.set-palette::yellow"));
    colors
}
