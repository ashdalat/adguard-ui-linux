// Port of adguard_modern/tabs/settings_filtering_mode_view.py — Filtering mode
// view with the Port configuration and the Proxy-for-updates card.
use gtk4::prelude::*;
use regex::RegexBuilder;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::toggle_switch::ToggleSwitch;

fn yaml_paths() -> Vec<PathBuf> {
    vec![
        c::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
    ]
}

fn get_yaml_val(pattern: &str, default: &str) -> String {
    let re = match RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(r) => r,
        Err(_) => return default.to_string(),
    };
    for p in yaml_paths() {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Some(m) = re.captures(&content) {
                    return m.get(1).map(|s| s.as_str().to_string()).unwrap_or_else(|| default.to_string());
                }
            }
        }
    }
    default.to_string()
}

fn set_yaml_key(pattern: &str, replacement: &str) {
    let re = match RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(r) => r,
        Err(_) => return,
    };
    for p in yaml_paths() {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                let updated = re.replace_all(&content, |caps: &regex::Captures| format!("{}{}", &caps[1], replacement));
                let _ = std::fs::write(&p, updated.as_ref());
            }
        }
    }
}

fn cli_bin() -> String {
    let home_bin = c::home_dir().join(".local/bin/adguard-cli");
    if home_bin.exists() {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    }
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_manual_t: gtk4::Label,
    lbl_manual_d: gtk4::Label,
    lbl_port: gtk4::Label,
    input_port: gtk4::Entry,
    lbl_auto_t: gtk4::Label,
    lbl_auto_d: gtk4::Label,
    lbl_proxy_t: gtk4::Label,
    lbl_proxy_d: gtk4::Label,
    radio_manual: gtk4::CheckButton,
    radio_auto: gtk4::CheckButton,
    switch_updates_proxy: ToggleSwitch,
    /// Python blockSignals() guards: programmatic setChecked must not re-save.
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    mode_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    open_updates_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct SettingsFilteringModeView {
    inner: Rc<Inner>,
}

impl SettingsFilteringModeView {
    pub fn new() -> SettingsFilteringModeView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(24);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer_px(18));

        let lbl_title = gtk4::Label::new(Some("Filtering mode"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer_px(24));

        let c_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        c_layout.set_margin_end(12);
        c_layout.set_margin_bottom(20);

        // Option 1: System proxy (Manual)
        let (box_manual, radio_manual, lbl_manual_t, lbl_manual_d) =
            create_mode_option("System proxy (Manual)", "Filters web traffic directed through local proxy port 127.0.0.1:3129. Recommended for Linux desktop stability.");
        c_layout.append(&box_manual);

        // Port configuration under Manual
        let port_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        port_container.set_margin_start(42);
        port_container.set_margin_bottom(8);
        let lbl_port = gtk4::Label::new(Some("Port"));
        lbl_port.add_css_class("lbl-white-10");
        lbl_port.set_halign(gtk4::Align::Start);
        port_container.append(&lbl_port);

        let http_val = get_yaml_val(r"http_proxy:\s*(-?[0-9]+)", "3129");
        let port_display = if http_val != "-1" { http_val.clone() } else { "3129".to_string() };
        let input_port = gtk4::Entry::new();
        input_port.set_text(&port_display);
        input_port.set_size_request(-1, 44);
        input_port.add_css_class("port-input");
        port_container.append(&input_port);
        c_layout.append(&port_container);

        // Option 2: Automatic redirect (iptables)
        let (box_auto, radio_auto, lbl_auto_t, lbl_auto_d) = create_mode_option(
            "Automatic redirect (iptables)",
            "Uses Linux kernel iptables to intercept and redirect network traffic directly into AdGuard without manual proxy configuration.");
        radio_auto.set_group(Some(&radio_manual));
        c_layout.append(&box_auto);
        c_layout.append(&spacer_px(14));

        // Proxy for app and filter updates
        let proxy_val = get_yaml_val(r"outbound_proxy:\s*\n\s*enabled:\s*(true|false)", "false");
        let switch_updates_proxy = ToggleSwitch::new(46, 26);
        switch_updates_proxy.set_checked(proxy_val.to_lowercase() == "true", false);

        let (card_updates_proxy, lbl_proxy_t, lbl_proxy_d) = create_card(
            "Proxy for app and filter updates",
            "Allows you to select a proxy server for updating the app and filters",
            &switch_updates_proxy,
            true,
        );
        c_layout.append(&card_updates_proxy);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&c_layout));
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");
        root.append(&scroll);

        let bb_root = btn_back.root.clone();
        let rm = radio_manual.clone();
        let ra = radio_auto.clone();
        let ip = input_port.clone();
        let sup = switch_updates_proxy.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_manual_t,
            lbl_manual_d,
            lbl_port,
            input_port,
            lbl_auto_t,
            lbl_auto_d,
            lbl_proxy_t,
            lbl_proxy_d,
            radio_manual,
            radio_auto,
            switch_updates_proxy,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            mode_cb: RefCell::new(None),
            open_updates_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });

        // Radio / box clicks save the mode (Python: box click checks the radio,
        // radio toggled saves when checked).
        let i3 = inner.clone();
        rm.connect_toggled(move |r| {
            if *i3.syncing.borrow() {
                return;
            }
            if r.is_active() {
                save_mode(&i3, "manual");
            }
        });
        let i4 = inner.clone();
        ra.connect_toggled(move |r| {
            if *i4.syncing.borrow() {
                return;
            }
            if r.is_active() {
                save_mode(&i4, "auto");
            }
        });
        let i5 = inner.clone();
        box_manual.add_controller(click_gesture(&i5, "manual", &rm));
        let i6 = inner.clone();
        box_auto.add_controller(click_gesture(&i6, "auto", &ra));

        ip.connect_activate(move |e| {
            let port_val = e.text().trim().to_string();
            let port_val = if port_val.is_empty() { "3129".to_string() } else { port_val };
            set_yaml_key(r"(http_proxy:\s*)-?[0-9]+", &port_val);
            let bin = cli_bin();
            let _ = std::process::Command::new(&bin)
                .args(["config", "set", "http_proxy", &port_val])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        });

        let i8 = inner.clone();
        sup.connect_toggled(Box::new(move |checked| on_updates_proxy_toggled(&i8, checked)));

        let i9 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i9)));

        load_current_mode(&inner);
        retranslate_ui(&inner);

        SettingsFilteringModeView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_mode_changed(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.mode_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_updates_proxy_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_updates_cb.borrow_mut() = Some(cb);
    }

    /// Python: `mode_view._load_current_mode()` before showing the page.
    pub fn load_current_mode(&self) {
        load_current_mode(&self.inner);
    }
}

fn click_gesture(inner: &Rc<Inner>, mode: &str, radio: &gtk4::CheckButton) -> gtk4::GestureClick {
    let gesture = gtk4::GestureClick::new();
    let m = mode.to_string();
    let r = radio.clone();
    let i = inner.clone();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        if n != 1 {
            return;
        }
        r.set_active(true);
        save_mode(&i, &m);
    });
    gesture
}

fn create_mode_option(
    title: &str,
    description: &str,
) -> (gtk4::Box, gtk4::CheckButton, gtk4::Label, gtk4::Label) {    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    box_.add_css_class("mode-option");
    box_.set_margin_start(8);
    box_.set_margin_end(12);
    box_.set_margin_top(6);
    box_.set_margin_bottom(6);

    let radio = gtk4::CheckButton::new();
    radio.add_css_class("radio-indicator");
    box_.append(&radio);

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    col.set_hexpand(true);
    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("nav-row-title");
    lbl_t.set_halign(gtk4::Align::Start);
    col.append(&lbl_t);
    let lbl_d = gtk4::Label::new(Some(description));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_wrap(true);
    lbl_d.set_halign(gtk4::Align::Start);
    col.append(&lbl_d);
    box_.append(&col);
    (box_, radio, lbl_t, lbl_d)
}

fn create_card(
    title: &str,
    desc: &str,
    switch: &ToggleSwitch,
    clickable: bool,
) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let card = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    card.add_css_class("settings-card");
    if clickable {
        card.add_css_class("settings-card-clickable");
    }
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    col.set_hexpand(true);
    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("lbl-title");
    lbl_t.set_halign(gtk4::Align::Start);
    col.append(&lbl_t);
    let lbl_d = gtk4::Label::new(Some(desc));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_halign(gtk4::Align::Start);
    col.append(&lbl_d);
    card.append(&col);
    card.append(switch.root());
    (card, lbl_t, lbl_d)
}

fn spacer_px(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn load_current_mode(inner: &Rc<Inner>) {
    *inner.syncing.borrow_mut() = true;
    let user_yaml = c::home_dir().join(".local/share/adguard-cli/proxy.yaml");
    let mut val = String::new();
    if user_yaml.exists() {
        if let Ok(content) = std::fs::read_to_string(&user_yaml) {
            if let Some(m) = RegexBuilder::new(r#"proxy_mode:\s*['"]?([a-zA-Z0-9_-]+)['"]?"#)
                .case_insensitive(false)
                .build()
                .ok()
                .and_then(|re| re.captures(&content))
            {
                val = m.get(1).unwrap().as_str().to_lowercase();
            }
        }
    }
    if val.is_empty() {
        let bin = cli_bin();
        if let Ok(out) = std::process::Command::new(&bin).args(["config", "get", "proxy_mode"]).output() {
            val = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
        }
    }
    if val.contains("auto") {
        inner.radio_auto.set_active(true);
        inner.input_port.set_sensitive(false);
    } else {
        inner.radio_manual.set_active(true);
        inner.input_port.set_sensitive(true);
    }
    *inner.syncing.borrow_mut() = false;

    let http_val = get_yaml_val(r"http_proxy:\s*(-?[0-9]+)", "3129");
    if http_val != "-1" {
        inner.input_port.set_text(&http_val);
    }

    let proxy_val = get_yaml_val(r"outbound_proxy:\s*\n\s*enabled:\s*(true|false)", "false");
    *inner.syncing.borrow_mut() = true;
    inner.switch_updates_proxy.set_checked(proxy_val.to_lowercase() == "true", false);
    *inner.syncing.borrow_mut() = false;
}

fn save_mode(inner: &Rc<Inner>, mode_val: &str) {
    inner.input_port.set_sensitive(mode_val == "manual");

    let bin = cli_bin();
    let _ = std::process::Command::new(&bin).args(["config", "set", "proxy_mode", mode_val]).status();
    let _ = std::process::Command::new("sudo").arg("-n").arg(&bin).args(["config", "set", "proxy_mode", mode_val]).status();

    if let Ok(re) = RegexBuilder::new(r"^[ \t]*proxy_mode:[ \t]*.*$").multi_line(true).build() {
        for p in yaml_paths() {
            if p.exists() {
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let updated = re.replace_all(&content, format!("proxy_mode: '{}'", mode_val));
                    let _ = std::fs::write(&p, updated.as_ref());
                }
            }
        }
    }

    let bin2 = bin.clone();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("sudo").arg("-n").arg(&bin2).arg("restart").status();
    });

    if let Some(cb) = inner.mode_cb.borrow().as_ref() {
        cb(mode_val);
    }
}

fn on_updates_proxy_toggled(inner: &Rc<Inner>, checked: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    if checked {
        let host = get_yaml_val(r#"host:\s*['"]?([^'"\n]+)['"]?"#, "");
        if host.is_empty() || host == "localhost" || host == "127.0.0.1" {
            *inner.syncing.borrow_mut() = true;
            inner.switch_updates_proxy.set_checked(false, true);
            *inner.syncing.borrow_mut() = false;
            if let Some(cb) = inner.open_updates_cb.borrow().as_ref() {
                cb();
            }
            return;
        }
    }
    let val_str = if checked { "true" } else { "false" };
    set_yaml_key(r"(outbound_proxy:\s*\n\s*enabled:\s*)(true|false)", val_str);
    let bin = cli_bin();
    let _ = std::process::Command::new(&bin)
        .args(["config", "set", "outbound_proxy.enabled", val_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("filtering_mode_title"));
    inner.lbl_manual_t.set_text(&i18n::tr("mode_manual_t"));
    inner.lbl_manual_d.set_text(&i18n::tr("mode_manual_d"));
    inner.lbl_port.set_text(&i18n::tr("mode_port_label"));
    inner.lbl_auto_t.set_text(&i18n::tr("mode_auto_t"));
    inner.lbl_auto_d.set_text(&i18n::tr("mode_auto_d"));
    inner.lbl_proxy_t.set_text(&i18n::tr("proxy_updates_card_t"));
    inner.lbl_proxy_d.set_text(&i18n::tr("proxy_updates_card_d"));
}
