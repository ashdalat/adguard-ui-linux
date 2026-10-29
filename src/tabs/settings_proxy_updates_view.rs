// Port of adguard_modern/tabs/settings_proxy_updates_view.py — the "Proxy for app
// and filter updates" subview.
use gtk4::prelude::*;
use regex::RegexBuilder;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use crate::widgets::common_controls::{no_wheel_combo, BackButton};

fn yaml_paths() -> Vec<PathBuf> {
    // Python order: root first, then the user directory.
    vec![
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
        c::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
    ]
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
    lbl_sub: gtk4::Label,
    lbl_sys_t: gtk4::Label,
    lbl_sys_d: gtk4::Label,
    lbl_none_t: gtk4::Label,
    lbl_none_d: gtk4::Label,
    lbl_custom_t: gtk4::Label,
    lbl_custom_d: gtk4::Label,
    lbl_pt: gtk4::Label,
    lbl_ps: gtk4::Label,
    lbl_pp: gtk4::Label,
    lbl_pu: gtk4::Label,
    lbl_pw: gtk4::Label,
    combo_type: gtk4::DropDown,
    input_server: gtk4::Entry,
    input_port: gtk4::Entry,
    input_user: gtk4::Entry,
    input_pass: gtk4::Entry,
    radio_sys: gtk4::CheckButton,
    radio_none: gtk4::CheckButton,
    radio_custom: gtk4::CheckButton,
    custom_box: gtk4::Box,
    /// Python blockSignals() guard: programmatic changes must not re-save.
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct SettingsProxyUpdatesView {
    inner: Rc<Inner>,
}

impl SettingsProxyUpdatesView {
    pub fn new() -> SettingsProxyUpdatesView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(20);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top_bar.append(&spacer);
        root.append(&top_bar);
        root.append(&spacer_px(22));

        let lbl_title = gtk4::Label::new(Some("Proxy for app and filter updates"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        let lbl_sub = gtk4::Label::new(Some("Allows you to select a proxy server for updating the app and filters"));
        lbl_sub.add_css_class("lbl-10");
        lbl_sub.set_halign(gtk4::Align::Start);
        root.append(&lbl_sub);
        root.append(&spacer_px(26));

        let c_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
        c_layout.set_margin_end(12);
        c_layout.set_margin_bottom(24);

        let (box_sys, radio_sys, lbl_sys_t, lbl_sys_d) = create_radio_item(
            "Use system proxy",
            "Detects if a proxy server is defined in the device settings",
        );
        c_layout.append(&box_sys);

        let (box_none, radio_none, lbl_none_t, lbl_none_d) = create_radio_item(
            "No proxy",
            "Receives AdGuard updates directly, bypassing any proxy servers",
        );
        radio_none.set_group(Some(&radio_sys));
        c_layout.append(&box_none);

        let (box_custom, radio_custom, lbl_custom_t, lbl_custom_d) = create_radio_item(
            "Use custom proxy",
            "Receives AdGuard updates using the proxy server specified below",
        );
        radio_custom.set_group(Some(&radio_sys));
        c_layout.append(&box_custom);
        c_layout.append(&spacer_px(14));

        // Custom proxy fields
        let custom_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        let lbl_pt = field_label(&custom_box, "Proxy type");
        let combo_type = no_wheel_combo();
        combo_type.set_model(Some(&string_list(&["HTTP", "HTTPS", "SOCKS4", "SOCKS5"])));
        combo_type.set_size_request(-1, 44);
        combo_type.add_css_class("proxy-input");
        custom_box.append(&combo_type);

        let lbl_ps = field_label(&custom_box, "Proxy server");
        let input_server = text_entry(&custom_box, "Domain or IP address", false);

        let lbl_pp = field_label(&custom_box, "Port");
        let input_port = text_entry(&custom_box, "", false);
        input_port.set_text("8080");

        let lbl_pu = field_label(&custom_box, "Username");
        let input_user = text_entry(&custom_box, "Username", false);

        let lbl_pw = field_label(&custom_box, "Password");
        let input_pass = text_entry(&custom_box, "Password", true);

        c_layout.append(&custom_box);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&c_layout));
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");
        root.append(&scroll);

        let bb_root = btn_back.root.clone();
        let rs = radio_sys.clone();
        let rn = radio_none.clone();
        let rc = radio_custom.clone();
        let ct = combo_type.clone();
        let is = input_server.clone();
        let ip = input_port.clone();
        let iu = input_user.clone();
        let ipw = input_pass.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_sub,
            lbl_sys_t,
            lbl_sys_d,
            lbl_none_t,
            lbl_none_d,
            lbl_custom_t,
            lbl_custom_d,
            lbl_pt,
            lbl_ps,
            lbl_pp,
            lbl_pu,
            lbl_pw,
            combo_type,
            input_server,
            input_port,
            input_user,
            input_pass,
            radio_sys,
            radio_none,
            radio_custom,
            custom_box,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });

        // Radio clicks: box click checks the radio then saves; radio toggled saves.
        for (box_, radio) in [(&box_sys, rs.clone()), (&box_none, rn.clone()), (&box_custom, rc.clone())] {
            let i3 = inner.clone();
            let r = radio.clone();
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_g, n, _x, _y| {
                if n != 1 {
                    return;
                }
                r.set_active(true);
                on_radio_toggled(&i3);
                save_config(&i3);
            });
            box_.add_controller(gesture);

            let i4 = inner.clone();
            radio.connect_toggled(move |r| {
                if *i4.syncing.borrow() {
                    return;
                }
                if r.is_active() {
                    on_radio_toggled(&i4);
                    save_config(&i4);
                }
            });
        }

        let i5 = inner.clone();
        ct.connect_selected_notify(move |d| {
            if *i5.syncing.borrow() {
                return;
            }
            let _ = d;
            save_config(&i5);
        });
        for entry in [is.clone(), ip.clone(), iu.clone(), ipw.clone()] {
            let i6 = inner.clone();
            entry.connect_activate(move |_| save_config(&i6));
        }

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        load_config(&inner);
        on_radio_toggled(&inner);
        retranslate_ui(&inner);

        SettingsProxyUpdatesView { inner }
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

    /// Python: `updates_proxy_view._load_config()` before showing the page.
    pub fn load_config(&self) {
        load_config(&self.inner);
    }
}

fn create_radio_item(title: &str, description: &str) -> (gtk4::Box, gtk4::CheckButton, gtk4::Label, gtk4::Label) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    box_.add_css_class("proxy-radio-box");
    box_.set_margin_start(8);
    box_.set_margin_end(12);
    box_.set_margin_top(4);
    box_.set_margin_bottom(4);

    let radio = gtk4::CheckButton::new();
    radio.add_css_class("radio-indicator");
    box_.append(&radio);

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    col.set_hexpand(true);
    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("lbl-title");
    lbl_t.set_halign(gtk4::Align::Start);
    col.append(&lbl_t);
    let lbl_d = gtk4::Label::new(Some(description));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_halign(gtk4::Align::Start);
    col.append(&lbl_d);
    box_.append(&col);
    (box_, radio, lbl_t, lbl_d)
}

fn field_label(parent: &gtk4::Box, text: &str) -> gtk4::Label {
    let lbl = gtk4::Label::new(Some(text));
    lbl.add_css_class("lbl-white-10");
    lbl.set_halign(gtk4::Align::Start);
    parent.append(&lbl);
    lbl
}

fn text_entry(parent: &gtk4::Box, placeholder: &str, secret: bool) -> gtk4::Entry {
    let entry = gtk4::Entry::new();
    if !placeholder.is_empty() {
        entry.set_placeholder_text(Some(placeholder));
    }
    if secret {
        entry.set_visibility(false);
    }
    entry.set_size_request(-1, 44);
    entry.add_css_class("proxy-input");
    parent.append(&entry);
    entry
}

fn string_list(items: &[&str]) -> gtk4::StringList {
    gtk4::StringList::new(items)
}

fn spacer_px(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn on_radio_toggled(inner: &Rc<Inner>) {
    let is_custom = inner.radio_custom.is_active();
    inner.custom_box.set_sensitive(is_custom);
    if is_custom {
        inner.custom_box.remove_css_class("dimmed");
    } else {
        inner.custom_box.add_css_class("dimmed");
    }
}

fn build_re(pattern: &str, dotall: bool) -> Option<regex::Regex> {
    let mut builder = RegexBuilder::new(pattern);
    if dotall {
        builder.dot_matches_new_line(true);
    }
    builder.case_insensitive(true).build().ok()
}

fn load_config(inner: &Rc<Inner>) {
    for p in yaml_paths() {
        if !p.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let is_en = build_re(r"outbound_proxy:\s*\n\s*enabled:\s*(true|false)", false)
            .and_then(|re| re.captures(&content))
            .map(|m| m.get(1).unwrap().as_str().to_lowercase() == "true")
            .unwrap_or(false);

        *inner.syncing.borrow_mut() = true;
        if is_en {
            inner.radio_custom.set_active(true);
        } else {
            inner.radio_sys.set_active(true);
        }

        if let Some(m) = build_re(r#"outbound_proxy:.*?\n\s*mode:\s*['"]?(\w+)['"]?"#, true).and_then(|re| re.captures(&content)) {
            let want = m.get(1).unwrap().as_str().to_uppercase();
            let items: Vec<String> = (0..4)
                .map(|i| {
                    inner
                        .combo_type
                        .model()
                        .and_then(|mo| mo.downcast_ref::<gtk4::StringList>().and_then(|sl| sl.string(i)))
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                })
                .collect();
            if let Some(idx) = items.iter().position(|s| *s == want) {
                inner.combo_type.set_selected(idx as u32);
            }
        }
        if let Some(m) = build_re(r#"outbound_proxy:.*?\n\s*host:\s*['"]?([^'"]*)['"]?"#, true).and_then(|re| re.captures(&content)) {
            inner.input_server.set_text(m.get(1).unwrap().as_str());
        }
        if let Some(m) = build_re(r"outbound_proxy:.*?\n\s*port:\s*([0-9]+)", true).and_then(|re| re.captures(&content)) {
            inner.input_port.set_text(m.get(1).unwrap().as_str());
        }
        if let Some(m) = build_re(r#"outbound_proxy:.*?\n\s*username:\s*['"]?([^'"]*)['"]?"#, true).and_then(|re| re.captures(&content)) {
            inner.input_user.set_text(m.get(1).unwrap().as_str());
        }
        if let Some(m) = build_re(r#"outbound_proxy:.*?\n\s*password:\s*['"]?([^'"]*)['"]?"#, true).and_then(|re| re.captures(&content)) {
            inner.input_pass.set_text(m.get(1).unwrap().as_str());
        }
        *inner.syncing.borrow_mut() = false;
        break;
    }
}

fn save_config(inner: &Rc<Inner>) {
    let is_custom = inner.radio_custom.is_active();
    let mode_val = inner
        .combo_type
        .model()
        .and_then(|mo| mo.downcast_ref::<gtk4::StringList>().and_then(|sl| sl.string(inner.combo_type.selected())))
        .unwrap_or_default()
        .to_string();
    let host_val = {
        let t = inner.input_server.text().trim().to_string();
        if t.is_empty() { "localhost".to_string() } else { t }
    };
    let port_val = {
        let t = inner.input_port.text().trim().to_string();
        if t.is_empty() { "3128".to_string() } else { t }
    };
    let user_val = inner.input_user.text().trim().to_string();
    let pass_val = inner.input_pass.text().trim().to_string();
    let en_str = if is_custom { "true" } else { "false" };

    let re_en = match build_re(r"(outbound_proxy:\s*\n\s*enabled:\s*)(true|false)", false) {
        Some(r) => r,
        None => return,
    };
    let re_mode = build_re(r#"(outbound_proxy:.*?\n\s*mode:\s*['"]?)[^'"]*(['"]?)"#, true);
    let re_host = build_re(r#"(outbound_proxy:.*?\n\s*host:\s*['"]?)[^'"]*(['"]?)"#, true);
    let re_port = build_re(r"(outbound_proxy:.*?\n\s*port:\s*)[0-9]+", true);
    let re_user = build_re(r#"(outbound_proxy:.*?\n\s*username:\s*['"]?)[^'"]*(['"]?)"#, true);
    let re_pass = build_re(r#"(outbound_proxy:.*?\n\s*password:\s*['"]?)[^'"]*(['"]?)"#, true);

    for p in yaml_paths() {
        if !p.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&p) {
            let mut c = re_en.replace_all(&content, |caps: &regex::Captures| format!("{}{}", &caps[1], en_str)).as_ref().to_string();
            if let Some(re) = &re_mode {
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}{}", &caps[1], mode_val, &caps[2])).as_ref().to_string();
            }
            if let Some(re) = &re_host {
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}{}", &caps[1], host_val, &caps[2])).as_ref().to_string();
            }
            if let Some(re) = &re_port {
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}", &caps[1], port_val)).as_ref().to_string();
            }
            if let Some(re) = &re_user {
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}{}", &caps[1], user_val, &caps[2])).as_ref().to_string();
            }
            if let Some(re) = &re_pass {
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}{}", &caps[1], pass_val, &caps[2])).as_ref().to_string();
            }
            let _ = std::fs::write(&p, c);
        }
    }

    let bin = cli_bin();
    let _ = std::process::Command::new("sudo")
        .arg("-n")
        .arg(&bin)
        .args(["config", "set", "outbound_proxy.enabled", &en_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::process::Command::new(&bin)
        .args(["config", "set", "outbound_proxy.enabled", &en_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("proxy_updates_card_t"));
    inner.lbl_sub.set_text(&i18n::tr("proxy_updates_card_d"));
    inner.lbl_sys_t.set_text(&i18n::tr("proxy_opt_sys_t"));
    inner.lbl_sys_d.set_text(&i18n::tr("proxy_opt_sys_d"));
    inner.lbl_none_t.set_text(&i18n::tr("proxy_opt_none_t"));
    inner.lbl_none_d.set_text(&i18n::tr("proxy_opt_none_d"));
    inner.lbl_custom_t.set_text(&i18n::tr("proxy_opt_custom_t"));
    inner.lbl_custom_d.set_text(&i18n::tr("proxy_opt_custom_d"));
    inner.lbl_pt.set_text(&i18n::tr("proxy_field_type"));
    inner.lbl_ps.set_text(&i18n::tr("proxy_field_server"));
    inner.input_server.set_placeholder_text(Some(&i18n::tr("proxy_field_server_ph")));
    inner.lbl_pp.set_text(&i18n::tr("mode_port_label"));
    inner.lbl_pu.set_text(&i18n::tr("proxy_field_user"));
    inner.input_user.set_placeholder_text(Some(&i18n::tr("proxy_field_user")));
    inner.lbl_pw.set_text(&i18n::tr("proxy_field_pass"));
    inner.input_pass.set_placeholder_text(Some(&i18n::tr("proxy_field_pass")));
}
