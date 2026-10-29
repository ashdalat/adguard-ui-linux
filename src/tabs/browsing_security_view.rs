// Port of adguard_modern/tabs/browsing_security_view.py
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use regex::Regex;

use crate::backend;
use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::toggle_switch::ToggleSwitch;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn proxy_yaml() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/proxy.yaml")
}

fn root_proxy_yaml() -> PathBuf {
    PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml")
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

// ---------------------------------------------------------------------------
// BrowsingOptionRow
// ---------------------------------------------------------------------------
struct OptionRowInner {
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: Option<gtk4::Label>,
    btn_link: Option<gtk4::Button>,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct BrowsingOptionRow {
    inner: Rc<OptionRowInner>,
}

impl BrowsingOptionRow {
    pub fn new(
        title: &str,
        description: &str,
        link_text: &str,
        link_url: &str,
        initial_state: bool,
    ) -> BrowsingOptionRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("bs-option-row");
        root.set_margin_top(10);
        root.set_margin_bottom(10);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc: Option<gtk4::Label> = if !description.is_empty() {
            let d = gtk4::Label::new(Some(description));
            d.add_css_class("lbl-desc-9");
            d.set_wrap(true);
            d.set_xalign(0.0);
            text_box.append(&d);
            Some(d)
        } else {
            None
        };

        let btn_link: Option<gtk4::Button> = if !link_text.is_empty() && !link_url.is_empty() {
            let b = gtk4::Button::new();
            b.set_label(link_text);
            b.add_css_class("bs-link");
            b.set_halign(gtk4::Align::Start);
            let url = link_url.to_string();
            b.connect_clicked(move |_| {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            });
            text_box.append(&b);
            Some(b)
        } else {
            None
        };

        root.append(&text_box);

        let switch = ToggleSwitch::new(46, 26);
        switch.set_checked(initial_state, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(OptionRowInner {
            root,
            lbl_title,
            lbl_desc,
            btn_link,
            switch,
            toggled_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(val);
            }
        }));

        BrowsingOptionRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_text(&self, title: &str, desc: &str, link_text: &str) {
        self.inner.lbl_title.set_text(title);
        if let Some(d) = self.inner.lbl_desc.as_ref() {
            if !desc.is_empty() {
                d.set_text(desc);
            }
        }
        if let Some(b) = self.inner.btn_link.as_ref() {
            if !link_text.is_empty() {
                b.set_label(link_text);
            }
        }
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        self.inner.switch.set_checked(checked, animate);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.switch.is_checked()
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// BrowsingSecurityView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    row_notify: BrowsingOptionRow,
    row_sound: BrowsingOptionRow,
    row_stats: BrowsingOptionRow,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct BrowsingSecurityView {
    inner: Rc<Inner>,
}

impl BrowsingSecurityView {
    pub fn new() -> BrowsingSecurityView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        root.add_css_class("dns-protection-view");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // 1. Back button.
        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);

        // 2. Title row + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Browsing security"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // 3. Subtitle.
        let lbl_desc = gtk4::Label::new(Some("Blocks requests to malicious and phishing websites"));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        root.append(&spacer(12));

        // 4. Option rows container.
        let options = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        options.add_css_class("transparent-container");

        // Row 1: Get notified about blocked requests.
        let row_notify = BrowsingOptionRow::new(
            "Get notified about blocked requests",
            "Receive notifications about blocked requests to malicious and phishing websites",
            "",
            "",
            true,
        );
        options.append(row_notify.root());

        // Row 2: Enable sound notifications.
        let row_sound = BrowsingOptionRow::new("Enable sound notifications", "", "", "", true);
        options.append(row_sound.root());

        // Row 3: Help AdGuard improve Browsing security + Privacy policy.
        let row_stats = BrowsingOptionRow::new(
            "Help AdGuard improve Browsing security",
            "Send anonymous security-related information to AdGuard",
            "Privacy policy",
            "https://adguard.com/privacy.html",
            false,
        );
        options.append(row_stats.root());

        root.append(&options);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            row_notify,
            row_sound,
            row_stats,
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.root.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggled(&i2, val)));

        let i3 = inner.clone();
        inner.row_stats.connect_toggled(Box::new(move |val| on_stats_toggled(&i3, val)));

        let i4 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i4)));

        load_settings(&inner);
        retranslate_ui(&inner);

        BrowsingSecurityView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_master_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.master_cb.borrow_mut() = Some(cb);
    }

    pub fn set_active(&self, is_active: bool) {
        self.inner.master_switch.set_checked(is_active, false);
        update_options_enabled(&self.inner, is_active);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn load_settings(&self) {
        load_settings(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("bs_title"));
    inner.lbl_desc.set_text(&i18n::tr("bs_desc"));
    inner.row_notify.update_text(&i18n::tr("bs_opt_notify_t"), &i18n::tr("bs_opt_notify_d"), "");
    inner.row_sound.update_text(&i18n::tr("bs_opt_sound_t"), "", "");
    inner.row_stats.update_text(
        &i18n::tr("bs_opt_stats_t"),
        &i18n::tr("bs_opt_stats_d"),
        &i18n::tr("bs_link_privacy"),
    );
}

// Read live safebrowsing config from proxy.yaml.
fn load_settings(inner: &Rc<Inner>) {
    let path = proxy_yaml();
    let mut sb_enabled = true;
    let mut send_stats = false;

    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let re_en = Regex::new(r"(?i)^\s+enabled:\s*(true|false)").unwrap();
            let re_st = Regex::new(r"(?i)^\s+send_anonymous_statistics:\s*(true|false)").unwrap();
            let mut in_sb = false;
            for line in content.lines() {
                let clean = line.split('#').next().unwrap_or("").trim_end();
                if clean.starts_with("safebrowsing:") {
                    in_sb = true;
                    continue;
                }
                if in_sb {
                    if !clean.is_empty() && !clean.starts_with(' ') && !clean.starts_with('\t') {
                        in_sb = false;
                        continue;
                    }
                    if let Some(caps) = re_en.captures(clean) {
                        sb_enabled = caps[1].to_lowercase() == "true";
                    }
                    if let Some(caps) = re_st.captures(clean) {
                        send_stats = caps[1].to_lowercase() == "true";
                    }
                }
            }
        }
    }

    inner.master_switch.set_checked(sb_enabled, false);
    inner.row_stats.set_checked(send_stats, false);
    update_options_enabled(inner, sb_enabled);
}

fn update_options_enabled(inner: &Rc<Inner>, enabled: bool) {
    for row in [&inner.row_notify, &inner.row_sound, &inner.row_stats] {
        row.root().set_sensitive(enabled);
        row.root().set_opacity(if enabled { 1.0 } else { 0.4 });
    }
}

fn on_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    update_options_enabled(inner, is_active);
    save_yaml_setting(inner, "enabled", is_active);

    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(
        &format!("adguard-cli config set safebrowsing.enabled {}", if is_active { "true" } else { "false" }),
        noop,
    );

    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(is_active);
    }
}

fn on_stats_toggled(inner: &Rc<Inner>, is_active: bool) {
    save_yaml_setting(inner, "send_anonymous_statistics", is_active);

    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(
        &format!("adguard-cli config set safebrowsing.send_anonymous_statistics {}", if is_active { "true" } else { "false" }),
        noop,
    );
}

fn save_yaml_setting(inner: &Rc<Inner>, key: &str, value: bool) {
    let val_str = if value { "true" } else { "false" };
    for p in [proxy_yaml(), root_proxy_yaml()] {
        if !p.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&p) {
            let re_key = Regex::new(&format!(r"^\s*{key}:\s*")).unwrap();
            let mut out: Vec<String> = Vec::new();
            let mut in_sb = false;
            let mut key_found = false;
            for line in content.lines() {
                if line.starts_with("safebrowsing:") {
                    in_sb = true;
                    out.push(line.to_string());
                    continue;
                }
                if in_sb {
                    if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                        if !key_found {
                            out.push(format!("  {key}: {val_str}"));
                        }
                        in_sb = false;
                    } else if re_key.is_match(line) {
                        out.push(format!("  {key}: {val_str}"));
                        key_found = true;
                        continue;
                    }
                }
                out.push(line.to_string());
            }
            let _ = std::fs::write(&p, out.join("\n") + "\n");
        }
    }
    let _ = inner;
}
