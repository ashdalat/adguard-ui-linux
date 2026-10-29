// Port of adguard_modern/widgets/common_controls.py
use gtk4::prelude::*;

use super::svg;

/// Port of AdGuardBackButton: chevron + text, text never cut off.
pub struct BackButton {
    pub root: gtk4::Button,
    lbl: gtk4::Label,
}

impl BackButton {
    pub fn new(text: &str) -> BackButton {
        let root = gtk4::Button::new();
        root.add_css_class("back-btn");
        root.set_size_request(110, 28);

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let chev = svg::svg_image("chevron_left.svg", 14);
        row.append(&chev);

        let lbl = gtk4::Label::new(Some(&clean_initial(text)));
        lbl.add_css_class("lbl");
        row.append(&lbl);
        root.set_child(Some(&row));

        BackButton { root, lbl }
    }

    /// Port of setText(): the button itself never shows text, only the label.
    pub fn set_text(&self, text: &str) {
        self.lbl.set_text(&clean_initial(text));
    }

    pub fn text(&self) -> String {
        self.lbl.text().to_string()
    }
}

fn clean_initial(text: &str) -> String {
    text.trim_start_matches("< ").trim().to_string()
}

/// Port of NoWheelComboBox: a dropdown that ignores mouse-wheel scrolling so the
/// page scrolls instead. GTK4's Dropdown is click-only (it has no wheel handler),
/// so the behaviour is already equivalent; this helper keeps the intent explicit.
pub fn no_wheel_combo() -> gtk4::DropDown {
    let combo = gtk4::DropDown::new(None::<gtk4::StringList>, None::<gtk4::Expression>);
    combo.add_css_class("combo-no-wheel");
    combo
}
