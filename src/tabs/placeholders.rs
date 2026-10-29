// Port of adguard_modern/tabs/placeholders.py
use gtk4::prelude::*;


pub const DEFAULT_SUBTITLE: &str = "Chụp ảnh tab này bên Windows để tao clone tiếp nhé!";

pub fn create_placeholder_tab(tab_name: &str, subtitle: &str) -> gtk4::Box {
    let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    widget.add_css_class("placeholder-tab");

    widget.append(&spacer(20, 40));

    let lbl_title = gtk4::Label::new(Some(tab_name));
    lbl_title.add_css_class("placeholder-title");
    lbl_title.set_halign(gtk4::Align::Center);
    widget.append(&lbl_title);

    let gap = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    gap.set_size_request(-1, 10);
    widget.append(&gap);

    let lbl_sub = gtk4::Label::new(Some(subtitle));
    lbl_sub.add_css_class("placeholder-sub");
    lbl_sub.set_halign(gtk4::Align::Center);
    widget.append(&lbl_sub);

    widget.append(&spacer(20, 60));
    widget
}

fn spacer(_w: i32, h: i32) -> gtk4::Box {
    let s = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    s.set_size_request(-1, h);
    s.set_vexpand(true);
    s
}

pub struct SettingsTab {
    root: gtk4::Box,
}

impl SettingsTab {
    pub fn new() -> SettingsTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.append(&create_placeholder_tab(
            "Settings",
            "Chụp ảnh tab Settings bên Windows dán vào để tao hoàn thiện!",
        ));
        SettingsTab { root }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.root
    }
}
