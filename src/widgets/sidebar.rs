// Port of adguard_modern/widgets/sidebar.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use super::svg;

const MENU_KEYS: [(&str, &str); 6] = [
    ("home", "nav_home"),
    ("protection", "nav_protection"),
    ("statistics", "nav_statistics"),
    ("apps", "nav_apps"),
    ("settings", "nav_settings"),
    ("updates", "nav_updates"),
];

const MENU_ITEMS: [(&str, &str, &str); 6] = [
    ("Home", "home.svg", "home"),
    ("Protection", "protection.svg", "protection"),
    ("Statistics", "statistics.svg", "statistics"),
    ("App management", "apps.svg", "apps"),
    ("Settings", "settings.svg", "settings"),
    ("Check for updates", "updates.svg", "updates"),
];

struct Inner {
    root: gtk4::Box,
    buttons: HashMap<String, gtk4::ToggleButton>,
    labels: HashMap<String, gtk4::Label>,
    title_label: gtk4::Label,
    lbl_hide: gtk4::Label,
    chevron: gtk4::Image,
    is_collapsed: RefCell<bool>,
    tab_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    hide_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct Sidebar {
    inner: Rc<Inner>,
}

impl Sidebar {
    pub fn new() -> Sidebar {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        root.set_size_request(236, -1);
        root.add_css_class("sidebar");
        // Python: self.main_layout.setContentsMargins(10, 18, 10, 18) — the margins
        // live on the INNER layout, so the sidebar background reaches the window
        // edges. Putting them on the sidebar itself left a strip of window
        // background visible along the left/top/bottom, which read as a different
        // colour next to the content area.
        root.set_margin_start(0);
        root.set_margin_end(0);
        root.set_margin_top(0);
        root.set_margin_bottom(0);

        // Top header: vector logo + "adguard"
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        header.set_margin_start(8 + 10);
        header.set_margin_end(8 + 10);
        header.set_margin_top(4 + 18);
        header.set_margin_bottom(20);
        let logo = gtk4::Image::new();
        logo.set_size_request(26, 26);
        let logo_path = c::logo_path();
        if logo_path.exists() {
            if let Ok(tex) = gtk4::gdk::Texture::from_filename(&logo_path) {
                logo.set_paintable(Some(&tex));
            }
        }
        header.append(&logo);
        let title_label = gtk4::Label::new(Some("adguard"));
        title_label.add_css_class("title");
        header.append(&title_label);
        root.append(&header);

        let mut buttons = HashMap::new();
        let mut labels = HashMap::new();

        for (text, icon_fn, tab_id) in MENU_ITEMS {
            let btn = gtk4::ToggleButton::new();
            btn.add_css_class("sidebar-btn");
            btn.set_tooltip_text(Some(text));
            let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
            row.set_margin_start(14 + 10);
            row.set_margin_end(14 + 10);
            row.append(&svg::svg_image(icon_fn, 22));
            let lbl = gtk4::Label::new(Some(text));
            lbl.add_css_class("lbl");
            row.append(&lbl);
            btn.set_child(Some(&row));
            root.append(&btn);
            buttons.insert(tab_id.to_string(), btn);
            labels.insert(tab_id.to_string(), lbl);
        }

        let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        root.append(&spacer);

        // Bottom Hide / Expand button
        let collapse_btn = gtk4::Button::new();
        collapse_btn.add_css_class("sidebar-btn");
        let toggle_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        toggle_row.set_margin_start(14 + 10);
        toggle_row.set_margin_end(14 + 10);
        toggle_row.set_margin_bottom(18);
        let chevron = svg::svg_image("chevron_left.svg", 14);
        toggle_row.append(&chevron);
        let lbl_hide = gtk4::Label::new(Some("Hide"));
        lbl_hide.add_css_class("lbl");
        toggle_row.append(&lbl_hide);
        collapse_btn.set_child(Some(&toggle_row));
        root.append(&collapse_btn);

        let inner = Rc::new(Inner {
            root,
            buttons,
            labels,
            title_label,
            lbl_hide,
            chevron,
            is_collapsed: RefCell::new(false),
            tab_cb: RefCell::new(None),
            hide_cb: RefCell::new(None),
        });
        let s = Sidebar { inner: inner.clone() };

        // Exclusive button group (QButtonGroup setExclusive(true))
        for (tab_id, btn) in inner.buttons.iter() {
            let s2 = s.clone();
            let tid = tab_id.clone();
            btn.connect_clicked(move |_| {
                for (other_id, other) in s2.inner.buttons.iter() {
                    if other_id != &tid {
                        other.set_active(false);
                    }
                }
                s2.refresh_styles();
                if let Some(cb) = s2.inner.tab_cb.borrow().as_ref() {
                    cb(&tid);
                }
            });
        }

        collapse_btn.connect_clicked({
            let s2 = s.clone();
            move |_| s2.toggle_collapse()
        });

        // Connect to i18n, then retranslate and set the default active tab
        i18n::connect_language_changed(Box::new({
            let s2 = s.clone();
            move |_| s2.retranslate_ui()
        }));
        s.retranslate_ui();
        s.set_active_tab("home");

        s
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_tab_selected(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.tab_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_hide_requested(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.hide_cb.borrow_mut() = Some(cb);
    }

    pub fn retranslate_ui(&self) {
        for (tab_id, key) in MENU_KEYS {
            if self.inner.labels.contains_key(tab_id) {
                let text = i18n::tr(key);
                if let Some(lbl) = self.inner.labels.get(tab_id) {
                    lbl.set_text(&text);
                }
                if let Some(btn) = self.inner.buttons.get(tab_id) {
                    btn.set_tooltip_text(Some(&text));
                }
            }
        }
        self.inner.lbl_hide.set_text(&i18n::tr("nav_hide"));
    }

    pub fn toggle_collapse(&self) {
        let collapsed = !*self.inner.is_collapsed.borrow();
        *self.inner.is_collapsed.borrow_mut() = collapsed;
        if collapsed {
            self.inner.root.set_size_request(56, -1);
            self.inner.title_label.set_visible(false);
            self.inner.lbl_hide.set_visible(false);
            if let Some(tex) = svg::svg_texture(&c::icons_dir().join("chevron_right.svg"), 14) {
                self.inner.chevron.set_paintable(Some(&tex));
            }
            for lbl in self.inner.labels.values() {
                lbl.set_visible(false);
            }
        } else {
            self.inner.root.set_size_request(236, -1);
            self.inner.title_label.set_visible(true);
            self.inner.lbl_hide.set_visible(true);
            if let Some(tex) = svg::svg_texture(&c::icons_dir().join("chevron_left.svg"), 14) {
                self.inner.chevron.set_paintable(Some(&tex));
            }
            for lbl in self.inner.labels.values() {
                lbl.set_visible(true);
            }
        }
        if let Some(cb) = self.inner.hide_cb.borrow().as_ref() {
            cb(collapsed);
        }
    }

    pub fn set_active_tab(&self, tab_id: &str) {
        if let Some(btn) = self.inner.buttons.get(tab_id) {
            btn.set_active(true);
            self.refresh_styles();
        }
    }

    fn refresh_styles(&self) {
        for (tab_id, btn) in self.inner.buttons.iter() {
            if btn.is_active() {
                btn.add_css_class("active");
                if let Some(lbl) = self.inner.labels.get(tab_id) {
                    lbl.add_css_class("bold");
                }
            } else {
                btn.remove_css_class("active");
                if let Some(lbl) = self.inner.labels.get(tab_id) {
                    lbl.remove_css_class("bold");
                }
            }
        }
    }

    /// Port of set_protection_state: "Keep original logo intact." — no-op.
    pub fn set_protection_state(&self, _is_active: bool) {}
}
