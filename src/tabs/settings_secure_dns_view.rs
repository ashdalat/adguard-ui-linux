// Port of adguard_modern/tabs/settings_secure_dns_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;
use regex::RegexBuilder;

fn proxy_yaml() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml")
        .to_string_lossy()
        .to_string()
}

const ROOT_PROXY_YAML: &str = "/root/.local/share/adguard-cli/proxy.yaml";

struct RowInner {
    root: gtk4::Box,
    key: String,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    check: gtk4::Image,
    click_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct SecureDnsOptionRow {
    inner: Rc<RowInner>,
}

impl SecureDnsOptionRow {
    pub fn new(key: &str, title: &str, description: &str) -> SecureDnsOptionRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("dns-option-row");
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(16);
        root.set_margin_end(16);

        let t_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        t_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title-10");
        lbl_title.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_desc);

        root.append(&t_box);

        let check = svg::svg_image("green_check.svg", 16);
        check.set_visible(false);
        check.set_valign(gtk4::Align::Center);
        root.append(&check);

        let inner = Rc::new(RowInner {
            root,
            key: key.to_string(),
            lbl_title,
            lbl_desc,
            check,
            click_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            if n != 1 {
                return;
            }
            if let Some(cb) = i1.click_cb.borrow().as_ref() {
                cb(&i1.key);
            }
        });
        inner.root.add_controller(gesture);

        SecureDnsOptionRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_selected(&self, selected: bool) {
        self.inner.check.set_visible(selected);
    }

    pub fn update_text(&self, title: &str, description: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(description);
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.click_cb.borrow_mut() = Some(cb);
    }
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    sw_master: ToggleSwitch,
    lbl_sub: gtk4::Label,
    lbl_mode_title: gtk4::Label,
    dropdown_frame: gtk4::Box,
    header: gtk4::Box,
    lbl_selected_title: gtk4::Label,
    lbl_arrow: gtk4::Label,
    panel: gtk4::Box,
    opt_fly: SecureDnsOptionRow,
    opt_redirect: SecureDnsOptionRow,
    current_mode: RefCell<String>,
    is_enabled: RefCell<bool>,
    is_expanded: RefCell<bool>,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    mode_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

pub struct SettingsSecureDnsView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsSecureDnsView {
    pub fn new() -> SettingsSecureDnsView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(22));

        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Filter secure DNS"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_hexpand(true);
        lbl_title.set_valign(gtk4::Align::Center);
        title_row.append(&lbl_title);

        let sw_master = ToggleSwitch::new(46, 26);
        sw_master.set_checked(true, false);
        sw_master.root().set_valign(gtk4::Align::Center);
        title_row.append(sw_master.root());
        root.append(&title_row);
        root.append(&spacer(6));

        let lbl_sub = gtk4::Label::new(Some(
            "Filters encrypted DNS requests in addition to plain DNS requests",
        ));
        lbl_sub.add_css_class("lbl-10");
        lbl_sub.set_wrap(true);
        lbl_sub.set_halign(gtk4::Align::Start);
        root.append(&lbl_sub);
        root.append(&spacer(26));

        let lbl_mode_title = gtk4::Label::new(Some("Filtering mode"));
        lbl_mode_title.add_css_class("lbl-10-medium-white");
        lbl_mode_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_mode_title);
        root.append(&spacer(8));

        let dropdown_frame = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        dropdown_frame.add_css_class("dns-dropdown-frame");

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header.add_css_class("dns-header");
        header.set_size_request(-1, 46);
        header.set_margin_start(16);
        header.set_margin_end(16);

        let lbl_selected_title = gtk4::Label::new(Some("Redirect to a custom DNS server"));
        lbl_selected_title.add_css_class("lbl-10-medium-white");
        lbl_selected_title.set_hexpand(true);
        lbl_selected_title.set_valign(gtk4::Align::Center);
        header.append(&lbl_selected_title);

        let lbl_arrow = gtk4::Label::new(Some("^"));
        lbl_arrow.add_css_class("dns-arrow");
        lbl_arrow.set_valign(gtk4::Align::Center);
        header.append(&lbl_arrow);

        dropdown_frame.append(&header);

        let panel = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        panel.add_css_class("dns-panel");
        panel.set_margin_top(6);
        panel.set_margin_bottom(6);
        panel.set_margin_start(6);
        panel.set_margin_end(6);

        let opt_fly = SecureDnsOptionRow::new(
            "transparent",
            "Filter on the fly",
            "Filters DNS-over-HTTPS traffic without redirecting it to the local DNS proxy. Uses the DNS server specified in the browser settings",
        );
        panel.append(opt_fly.root());

        let opt_redirect = SecureDnsOptionRow::new(
            "redirect",
            "Redirect to a custom DNS server",
            "Redirects DNS-over-HTTPS requests to the local DNS proxy for filtering. Uses the DNS server selected in the app",
        );
        panel.append(opt_redirect.root());

        dropdown_frame.append(&panel);
        root.append(&dropdown_frame);

        let bb_root = btn_back.root.clone();
        let hd = header.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            sw_master,
            lbl_sub,
            lbl_mode_title,
            dropdown_frame,
            header,
            lbl_selected_title,
            lbl_arrow,
            panel,
            opt_fly,
            opt_redirect,
            current_mode: RefCell::new("transparent".to_string()),
            is_enabled: RefCell::new(true),
            is_expanded: RefCell::new(true),
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            mode_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        hd.add_controller(click_gesture(&i2));

        let i3 = inner.clone();
        inner.opt_fly.connect_clicked(Box::new(move |key| select_mode(&i3, key)));
        let i4 = inner.clone();
        inner.opt_redirect.connect_clicked(Box::new(move |key| select_mode(&i4, key)));

        let i5 = inner.clone();
        inner.sw_master.connect_toggled(Box::new(move |checked| on_master_toggled(&i5, checked)));

        let i6 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i6)));

        load_config(&inner);
        retranslate_ui(&inner);

        SettingsSecureDnsView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_mode_changed(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.mode_cb.borrow_mut() = Some(cb);
    }

    pub fn load_config(&self) {
        load_config(&self.inner);
    }

    pub fn set_mode(&self, mode: &str) {
        if mode == "off" {
            *self.inner.syncing.borrow_mut() = true;
            self.inner.sw_master.set_checked(false, true);
            *self.inner.syncing.borrow_mut() = false;
        } else {
            *self.inner.syncing.borrow_mut() = true;
            self.inner.sw_master.set_checked(true, true);
            *self.inner.syncing.borrow_mut() = false;
            select_mode(&self.inner, mode);
        }
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn click_gesture(inner: &Rc<Inner>) -> gtk4::GestureClick {
    let i = inner.clone();
    let gesture = gtk4::GestureClick::new();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        if n != 1 {
            return;
        }
        toggle_dropdown(&i);
    });
    gesture
}

fn toggle_dropdown(inner: &Rc<Inner>) {
    if !*inner.is_enabled.borrow() {
        return;
    }
    // Read into a local before writing: `borrow_mut() = !borrow()` holds both
    // borrows at once and panics with "RefCell already borrowed".
    let expanded = !*inner.is_expanded.borrow();
    *inner.is_expanded.borrow_mut() = expanded;
    inner.panel.set_visible(expanded);
    inner.lbl_arrow.set_text(if expanded { "^" } else { "v" });
}

fn load_config(inner: &Rc<Inner>) {
    let mut mode = "transparent".to_string();
    let path = proxy_yaml();
    if std::path::Path::exists(&std::path::PathBuf::from(&path)) {
        if let Ok(c) = std::fs::read_to_string(&path) {
            let re = RegexBuilder::new(r#"filter_secure_dns_mode:\s*['"]?(transparent|redirect|off)['"]?"#)
                .case_insensitive(true)
                .build()
                .unwrap();
            if let Some(m) = re.captures(&c) {
                mode = m[1].to_lowercase();
            }
        }
    }

    *inner.syncing.borrow_mut() = true;
    if mode == "off" {
        *inner.is_enabled.borrow_mut() = false;
        inner.sw_master.set_checked(false, false);
        update_enabled_state(inner, false);
    } else {
        *inner.is_enabled.borrow_mut() = true;
        inner.sw_master.set_checked(true, false);
        *inner.current_mode.borrow_mut() = mode.clone();
        update_selected_ui(inner, &mode);
        update_enabled_state(inner, true);
    }
    *inner.syncing.borrow_mut() = false;
}

fn on_master_toggled(inner: &Rc<Inner>, checked: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    *inner.is_enabled.borrow_mut() = checked;
    update_enabled_state(inner, checked);
    let val = if checked {
        inner.current_mode.borrow().clone()
    } else {
        "off".to_string()
    };
    write_yaml(&val);
    if let Some(cb) = inner.mode_cb.borrow().as_ref() {
        cb(&val);
    }
}

fn select_mode(inner: &Rc<Inner>, mode: &str) {
    *inner.current_mode.borrow_mut() = mode.to_string();
    update_selected_ui(inner, mode);
    if *inner.is_enabled.borrow() {
        write_yaml(mode);
        if let Some(cb) = inner.mode_cb.borrow().as_ref() {
            cb(mode);
        }
    }
}

fn update_selected_ui(inner: &Rc<Inner>, mode: &str) {
    if mode == "transparent" {
        inner.lbl_selected_title.set_text(&i18n::tr("sec_dns_opt_fly_t"));
        inner.opt_fly.set_selected(true);
        inner.opt_redirect.set_selected(false);
    } else {
        inner.lbl_selected_title.set_text(&i18n::tr("sec_dns_opt_redirect_t"));
        inner.opt_fly.set_selected(false);
        inner.opt_redirect.set_selected(true);
    }
}

fn update_enabled_state(inner: &Rc<Inner>, enabled: bool) {
    inner.header.set_sensitive(enabled);
    inner.panel.set_sensitive(enabled);
    if enabled {
        inner.dropdown_frame.remove_css_class("dns-dimmed");
    } else {
        inner.dropdown_frame.add_css_class("dns-dimmed");
    }
}

fn write_yaml(mode: &str) {
    for path in [proxy_yaml(), ROOT_PROXY_YAML.to_string()] {
        if !std::path::Path::exists(&std::path::PathBuf::from(&path)) {
            continue;
        }
        if let Ok(mut c) = std::fs::read_to_string(&path) {
            if c.contains("filter_secure_dns_mode:") {
                let re = RegexBuilder::new(r#"(filter_secure_dns_mode:\s*['"]?)(transparent|redirect|off)(['"]?)"#)
                    .case_insensitive(true)
                    .build()
                    .unwrap();
                let m = mode.to_string();
                c = re
                    .replace_all(&c, |caps: &regex::Captures| {
                        format!("{}{}{}", &caps[1], m, &caps[3])
                    })
                    .to_string();
            } else {
                c.push_str(&format!("\nfilter_secure_dns_mode: {}\n", mode));
            }
            let _ = std::fs::write(&path, c);
        }
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("sec_dns_title"));
    inner.lbl_sub.set_text(&i18n::tr("sec_dns_sub"));
    inner.lbl_mode_title.set_text(&i18n::tr("sec_dns_mode_label"));
    inner.opt_fly.update_text(&i18n::tr("sec_dns_opt_fly_t"), &i18n::tr("sec_dns_opt_fly_d"));
    inner.opt_redirect.update_text(
        &i18n::tr("sec_dns_opt_redirect_t"),
        &i18n::tr("sec_dns_opt_redirect_d"),
    );
    let mode = inner.current_mode.borrow().clone();
    update_selected_ui(inner, &mode);
}
