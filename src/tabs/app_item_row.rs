// Port of adguard_modern/tabs/app_item_row.py
use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::constants;
use crate::widgets::svg;

// ---------------------------------------------------------------------------
// AppBadgeButton
// ---------------------------------------------------------------------------
struct BadgeInner {
    root: gtk4::Button,
    badge_type: String,
    is_enabled: Cell<bool>,
    toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
}

#[derive(Clone)]
pub struct AppBadgeButton {
    inner: Rc<BadgeInner>,
}

impl AppBadgeButton {
    pub fn new(badge_type: &str, enabled: bool) -> AppBadgeButton {
        let root = gtk4::Button::new();
        root.set_size_request(18, 18);
        root.add_css_class("app-badge-btn");

        let inner = Rc::new(BadgeInner {
            root,
            badge_type: badge_type.to_string(),
            is_enabled: Cell::new(enabled),
            toggled_cb: RefCell::new(None),
        });

        update_badge_ui(&inner);

        let i = inner.clone();
        inner.root.connect_clicked(move |_| {
            let enabled = !i.is_enabled.get();
            i.is_enabled.set(enabled);
            update_badge_ui(&i);
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(&i.badge_type, enabled);
            }
        });

        AppBadgeButton { inner }
    }

    pub fn root(&self) -> &gtk4::Button {
        &self.inner.root
    }

    pub fn set_enabled_state(&self, enabled: bool) {
        self.inner.is_enabled.set(enabled);
        update_badge_ui(&self.inner);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.is_enabled.get()
    }

    pub fn connect_toggled_state(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }
}

fn update_badge_ui(inner: &Rc<BadgeInner>) {
    let (fn_name, tip) = if inner.badge_type == "route" {
        (
            if inner.is_enabled.get() { "badge_shield.svg" } else { "badge_shield_off.svg" },
            if inner.is_enabled.get() {
                "Enabled: Route traffic through AdGuard"
            } else {
                "Disabled: Route traffic through AdGuard"
            },
        )
    } else if inner.badge_type == "filter" {
        (
            if inner.is_enabled.get() { "badge_slash.svg" } else { "badge_slash_off.svg" },
            if inner.is_enabled.get() {
                "Enabled: Filter traffic"
            } else {
                "Disabled: Filter traffic"
            },
        )
    } else {
        (
            if inner.is_enabled.get() { "badge_lock.svg" } else { "badge_lock_off.svg" },
            if inner.is_enabled.get() {
                "Enabled: Filter HTTPS traffic"
            } else {
                "Disabled: Filter HTTPS traffic"
            },
        )
    };

    let path = constants::icon_path(fn_name);
    if path.exists() {
        if let Some(tex) = svg::svg_texture(&path, 14) {
            let img = gtk4::Image::new();
            img.set_paintable(Some(&tex));
            img.set_size_request(14, 14);
            inner.root.set_child(Some(&img));
        }
    }
    inner.root.set_tooltip_text(Some(tip));
}

// ---------------------------------------------------------------------------
// AppItemRow
// ---------------------------------------------------------------------------
struct RowInner {
    root: gtk4::Box,
    app_name: String,
    exec_bin: String,
    btn_route: AppBadgeButton,
    btn_filter: AppBadgeButton,
    btn_https: AppBadgeButton,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    badge_cb: RefCell<Option<Box<dyn Fn(&str, &str, bool)>>>,
}

#[derive(Clone)]
pub struct AppItemRow {
    inner: Rc<RowInner>,
}

impl AppItemRow {
    pub fn new(
        icon_hint: &str,
        app_name: &str,
        exec_bin: &str,
        route_enabled: bool,
        filter_enabled: bool,
        https_enabled: bool,
        warning_note: &str,
    ) -> AppItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("app-item-row");
        root.set_margin_top(8);
        root.set_margin_bottom(8);
        root.set_margin_start(12);
        root.set_margin_end(14);

        // 1. Real App Icon (from system theme or SVGs).
        let icon_label = resolve_icon(icon_hint, app_name);
        icon_label.set_valign(gtk4::Align::Center);
        root.append(&icon_label);

        // 2. Text layout: Name + warning + 3 Badges.
        let text_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_layout.set_hexpand(true);

        let lbl_name = gtk4::Label::new(Some(app_name));
        lbl_name.add_css_class("lbl-10-bold-white");
        lbl_name.set_halign(gtk4::Align::Start);
        text_layout.append(&lbl_name);

        if !warning_note.is_empty() {
            let lbl_warn = gtk4::Label::new(Some(warning_note));
            lbl_warn.add_css_class("app-warn-8");
            lbl_warn.set_halign(gtk4::Align::Start);
            text_layout.append(&lbl_warn);
        }

        // 3. Interactive badges with exact tooltips.
        let badges_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        badges_layout.set_margin_top(2);

        let btn_route = AppBadgeButton::new("route", route_enabled);
        let btn_filter = AppBadgeButton::new("filter", filter_enabled);
        let btn_https = AppBadgeButton::new("https", https_enabled);

        badges_layout.append(btn_route.root());
        badges_layout.append(btn_filter.root());
        badges_layout.append(btn_https.root());
        text_layout.append(&badges_layout);

        root.append(&text_layout);

        // Chevron >
        let chev_path = constants::icon_path("chevron_right.svg");
        if chev_path.exists() {
            let chevron = svg::svg_image("chevron_right.svg", 14);
            chevron.set_valign(gtk4::Align::Center);
            root.append(&chevron);
        } else {
            let lbl_chev = gtk4::Label::new(Some("›"));
            lbl_chev.add_css_class("chevron-label");
            lbl_chev.set_valign(gtk4::Align::Center);
            root.append(&lbl_chev);
        }

        let inner = Rc::new(RowInner {
            root,
            app_name: app_name.to_string(),
            exec_bin: exec_bin.to_string(),
            btn_route,
            btn_filter,
            btn_https,
            clicked_cb: RefCell::new(None),
            badge_cb: RefCell::new(None),
        });

        for btn in [&inner.btn_route, &inner.btn_filter, &inner.btn_https] {
            let i = inner.clone();
            btn.connect_toggled_state(Box::new(move |badge_type, enabled| {
                if let Some(cb) = i.badge_cb.borrow().as_ref() {
                    cb(&i.exec_bin, badge_type, enabled);
                }
            }));
        }

        // Left-click on the row emits clicked. Deviation: Python explicitly
        // excludes presses that land on a QPushButton child; in GTK4 child
        // buttons consume the press themselves, so the row gesture does not
        // fire for badge clicks — the observable behaviour matches.
        let i = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            if n != 1 {
                return;
            }
            if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                cb(&i.app_name);
            }
        });
        inner.root.add_controller(gesture);

        AppItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn get_states(&self) -> (bool, bool, bool) {
        (
            self.inner.btn_route.is_checked(),
            self.inner.btn_filter.is_checked(),
            self.inner.btn_https.is_checked(),
        )
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_badge_changed(&self, cb: Box<dyn Fn(&str, &str, bool)>) {
        *self.inner.badge_cb.borrow_mut() = Some(cb);
    }

    pub fn app_name(&self) -> &str {
        &self.inner.app_name
    }

    pub fn exec_bin(&self) -> &str {
        &self.inner.exec_bin
    }

    pub fn set_route(&self, enabled: bool) {
        self.inner.btn_route.set_enabled_state(enabled);
    }

    pub fn set_filter(&self, enabled: bool) {
        self.inner.btn_filter.set_enabled_state(enabled);
    }

    pub fn set_https(&self, enabled: bool) {
        self.inner.btn_https.set_enabled_state(enabled);
    }
}

fn resolve_icon(icon_hint: &str, app_name: &str) -> gtk4::Image {
    let img = gtk4::Image::new();
    img.set_size_request(26, 26);

    // 1. Check system icon theme.
    if !icon_hint.is_empty() {
        if let Some(display) = gtk4::gdk::Display::default() {
            let theme = gtk4::IconTheme::for_display(&display);
            if theme.has_icon(icon_hint) {
                img.set_icon_name(Some(icon_hint));
                return img;
            }
        }
    }

    // 2. Check company SVG icons.
    let cand_keys = [icon_hint, &app_name.to_lowercase().replace(' ', "")];
    for k in cand_keys {
        if k.is_empty() {
            continue;
        }
        for fname in [format!("company_{}.svg", k), format!("{}.svg", k), format!("{}.png", k)] {
            let p = constants::icons_dir().join("companies").join(&fname);
            if p.exists() {
                if let Some(tex) = svg::svg_texture(&p, 26) {
                    img.set_paintable(Some(&tex));
                    return img;
                }
            }
        }
        for fname in [format!("{}.svg", k), format!("{}.png", k)] {
            let p = constants::icon_path(&fname);
            if p.exists() {
                if let Some(tex) = svg::svg_texture(&p, 26) {
                    img.set_paintable(Some(&tex));
                    return img;
                }
            }
        }
    }

    // 3. Fallback generic app icon.
    let p_def = constants::icon_path("apps.svg");
    if p_def.exists() {
        if let Some(tex) = svg::svg_texture(&p_def, 26) {
            img.set_paintable(Some(&tex));
            return img;
        }
    }

    img.add_css_class("svg-icon");
    img
}
