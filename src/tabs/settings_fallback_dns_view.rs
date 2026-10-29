// Port of adguard_modern/tabs/settings_fallback_dns_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::{no_wheel_combo, BackButton};
use crate::widgets::toggle_switch::ToggleSwitch;
use regex::Regex;

fn proxy_yaml() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml")
        .to_string_lossy()
        .to_string()
}

const ROOT_PROXY_YAML: &str = "/root/.local/share/adguard-cli/proxy.yaml";

const DEFAULT_EXCLUSIONS: &str = "*.local\n\
*.lan\n\
b._dns-sd._udp.*.in-addr.arpa\n\
lb._dns-sd._udp.*.in-addr.arpa\n\
*.service.arpa\n\
epdg.epc.aptg.com.tw\n\
epdg.epc.att.net";

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    sw_master: ToggleSwitch,
    lbl_sub: gtk4::Label,
    combo_type: gtk4::DropDown,
    box_custom: gtk4::Box,
    lbl_custom: gtk4::Label,
    edit_custom: gtk4::TextView,
    lbl_to_title: gtk4::Label,
    sw_timeout: ToggleSwitch,
    lbl_to_desc: gtk4::Label,
    lbl_to_ms: gtk4::Label,
    input_timeout: gtk4::Entry,
    lbl_ex_title: gtk4::Label,
    sw_ex: ToggleSwitch,
    lbl_ex_desc: gtk4::Label,
    lbl_ex_sub: gtk4::Label,
    edit_ex: gtk4::TextView,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    fallbacks_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

pub struct SettingsFallbackDnsView {
    inner: Rc<Inner>,
}

impl SettingsFallbackDnsView {
    pub fn new() -> SettingsFallbackDnsView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(22));

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 20);
        container.add_css_class("transparent-container");
        container.set_margin_end(12);
        container.set_margin_bottom(24);

        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Use fallback DNS servers"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_hexpand(true);
        lbl_title.set_valign(gtk4::Align::Center);
        title_row.append(&lbl_title);

        let sw_master = ToggleSwitch::new(46, 26);
        sw_master.set_checked(true, false);
        sw_master.root().set_valign(gtk4::Align::Center);
        title_row.append(sw_master.root());
        container.append(&title_row);

        let lbl_sub = gtk4::Label::new(Some(
            "DNS servers used for timed-out requests or DNS exclusions",
        ));
        lbl_sub.add_css_class("lbl-10");
        lbl_sub.set_wrap(true);
        lbl_sub.set_halign(gtk4::Align::Start);
        container.append(&lbl_sub);
        container.append(&spacer(6));

        let combo_type = no_wheel_combo();
        combo_type.set_size_request(-1, 44);
        combo_type.add_css_class("fb-combo");
        combo_type.set_model(Some(&string_list(&["System DNS servers", "Custom DNS servers"])));
        container.append(&combo_type);

        let box_custom = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        box_custom.set_margin_top(6);
        let lbl_custom = gtk4::Label::new(Some("Custom DNS server addresses, one per line"));
        lbl_custom.add_css_class("lbl-10-medium-white");
        lbl_custom.set_halign(gtk4::Align::Start);
        box_custom.append(&lbl_custom);

        let edit_custom = gtk4::TextView::new();
        edit_custom.set_size_request(-1, 100);
        edit_custom.add_css_class("fb-edit");
        box_custom.append(&edit_custom);
        container.append(&box_custom);
        box_custom.set_visible(false);
        container.append(&spacer(10));

        // Section 2: timed-out requests.
        let to_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_to_title = gtk4::Label::new(Some("Use fallback for timed-out requests"));
        lbl_to_title.add_css_class("lbl-11-bold-white");
        lbl_to_title.set_hexpand(true);
        lbl_to_title.set_valign(gtk4::Align::Center);
        to_row.append(&lbl_to_title);

        let sw_timeout = ToggleSwitch::new(46, 26);
        sw_timeout.set_checked(false, false);
        sw_timeout.root().set_valign(gtk4::Align::Center);
        to_row.append(sw_timeout.root());
        container.append(&to_row);

        let lbl_to_desc = gtk4::Label::new(Some(
            "Redirects requests to the fallback DNS servers if the DNS server selected in the app doesn’t reply within the specified timeout period",
        ));
        lbl_to_desc.add_css_class("lbl-10");
        lbl_to_desc.set_wrap(true);
        lbl_to_desc.set_halign(gtk4::Align::Start);
        container.append(&lbl_to_desc);
        container.append(&spacer(4));

        let lbl_to_ms = gtk4::Label::new(Some("DNS server timeout, ms"));
        lbl_to_ms.add_css_class("lbl-10-medium-white");
        lbl_to_ms.set_halign(gtk4::Align::Start);
        container.append(&lbl_to_ms);

        let input_timeout = gtk4::Entry::new();
        input_timeout.set_text("0");
        input_timeout.set_size_request(-1, 44);
        input_timeout.add_css_class("frag-input");
        container.append(&input_timeout);
        container.append(&spacer(10));

        // Section 3: DNS exclusions.
        let ex_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_ex_title = gtk4::Label::new(Some("Use fallback for DNS exclusions"));
        lbl_ex_title.add_css_class("lbl-11-bold-white");
        lbl_ex_title.set_hexpand(true);
        lbl_ex_title.set_valign(gtk4::Align::Center);
        ex_row.append(&lbl_ex_title);

        let sw_ex = ToggleSwitch::new(46, 26);
        sw_ex.set_checked(false, false);
        sw_ex.root().set_valign(gtk4::Align::Center);
        ex_row.append(sw_ex.root());
        container.append(&ex_row);

        let lbl_ex_desc = gtk4::Label::new(Some(
            "Redirects requests for the domains below to the fallback DNS servers instead of the DNS server selected in the app. DNS user rules are not applied to these requests",
        ));
        lbl_ex_desc.add_css_class("lbl-10");
        lbl_ex_desc.set_wrap(true);
        lbl_ex_desc.set_halign(gtk4::Align::Start);
        container.append(&lbl_ex_desc);
        container.append(&spacer(4));

        let lbl_ex_sub = gtk4::Label::new(Some("DNS exclusions, one per line"));
        lbl_ex_sub.add_css_class("lbl-10-medium-white");
        lbl_ex_sub.set_halign(gtk4::Align::Start);
        container.append(&lbl_ex_sub);

        let edit_ex = gtk4::TextView::new();
        edit_ex.buffer().set_text(DEFAULT_EXCLUSIONS);
        edit_ex.set_size_request(-1, 140);
        edit_ex.add_css_class("fb-edit-10");
        container.append(&edit_ex);

        scroll.set_child(Some(&container));
        root.append(&scroll);

        let bb_root = btn_back.root.clone();
        let ct = combo_type.clone();
        let ec = edit_custom.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            sw_master,
            lbl_sub,
            combo_type,
            box_custom,
            lbl_custom,
            edit_custom,
            lbl_to_title,
            sw_timeout,
            lbl_to_desc,
            lbl_to_ms,
            input_timeout,
            lbl_ex_title,
            sw_ex,
            lbl_ex_desc,
            lbl_ex_sub,
            edit_ex,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            fallbacks_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        ct.connect_selected_notify(move |c| {
            if *i2.syncing.borrow() {
                return;
            }
            on_combo_changed(&i2, c.selected() as usize);
        });

        let i3 = inner.clone();
        let buf = ec.buffer();
        buf.connect_changed(move |_b| save_fallbacks(&i3));

        let i4 = inner.clone();
        inner.sw_master.connect_toggled(Box::new(move |checked| on_master_toggled(&i4, checked)));

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        load_config(&inner);
        retranslate_ui(&inner);

        SettingsFallbackDnsView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_fallbacks_changed(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.fallbacks_cb.borrow_mut() = Some(cb);
    }

    pub fn load_config(&self) {
        load_config(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn string_list(items: &[&str]) -> gtk4::StringList {
    let sl = gtk4::StringList::new(&[]);
    for s in items {
        sl.append(s);
    }
    sl
}

fn text_of(buffer: &gtk4::TextBuffer) -> String {
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    buffer.text(&start, &end, false).to_string()
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("fb_title"));
    inner.lbl_sub.set_text(&i18n::tr("fb_sub"));

    // Python rebuilds the combo items with translations while keeping the index,
    // with signals blocked.
    let curr = inner.combo_type.selected() as i64;
    *inner.syncing.borrow_mut() = true;
    let sl = string_list(&[&i18n::tr("fb_combo_system"), &i18n::tr("fb_combo_custom")]);
    inner.combo_type.set_model(Some(&sl));
    inner.combo_type.set_selected(if curr >= 0 { curr as u32 } else { 0 });
    *inner.syncing.borrow_mut() = false;

    inner.lbl_custom.set_text(&i18n::tr("fb_custom_label"));
    // Deviation: GTK4 TextView has no placeholder text ("Server addresses").
    inner.lbl_to_title.set_text(&i18n::tr("fb_timeout_title"));
    inner.lbl_to_desc.set_text(&i18n::tr("fb_timeout_desc"));
    inner.lbl_to_ms.set_text(&i18n::tr("fb_timeout_ms"));
    inner.lbl_ex_title.set_text(&i18n::tr("fb_ex_title"));
    inner.lbl_ex_desc.set_text(&i18n::tr("fb_ex_desc"));
    inner.lbl_ex_sub.set_text(&i18n::tr("fb_ex_sub"));
}

fn load_config(inner: &Rc<Inner>) {
    let mut val = "default".to_string();
    let path = proxy_yaml();
    if std::path::Path::exists(&std::path::PathBuf::from(&path)) {
        if let Ok(c) = std::fs::read_to_string(&path) {
            if let Some(m) = Regex::new(r"fallbacks:\s*([^\n]+)").unwrap().captures(&c) {
                val = m[1].trim().to_string();
            }
        }
    }

    *inner.syncing.borrow_mut() = true;
    if val == "default" || val.is_empty() {
        inner.combo_type.set_selected(0);
        inner.box_custom.set_visible(false);
        inner.edit_custom.buffer().set_text("");
    } else {
        inner.combo_type.set_selected(1);
        inner.box_custom.set_visible(true);
        inner
            .edit_custom
            .buffer()
            .set_text(&val.split_whitespace().collect::<Vec<_>>().join("\n"));
    }
    *inner.syncing.borrow_mut() = false;
}

fn on_master_toggled(inner: &Rc<Inner>, checked: bool) {
    inner.combo_type.set_sensitive(checked);
    inner.box_custom.set_sensitive(checked);
    inner.sw_timeout.root().set_sensitive(checked);
    inner.sw_ex.root().set_sensitive(checked);
    inner.input_timeout.set_sensitive(checked);
    inner.edit_ex.set_sensitive(checked);
    if !checked {
        write_fallback_to_yaml("");
        if let Some(cb) = inner.fallbacks_cb.borrow().as_ref() {
            cb("disabled");
        }
    } else {
        save_fallbacks(inner);
    }
}

fn on_combo_changed(inner: &Rc<Inner>, index: usize) {
    inner.box_custom.set_visible(index == 1);
    save_fallbacks(inner);
}

fn save_fallbacks(inner: &Rc<Inner>) {
    if *inner.syncing.borrow() {
        return;
    }
    if !inner.sw_master.is_checked() {
        return;
    }
    let fb_val = if inner.combo_type.selected() == 0 {
        "default".to_string()
    } else {
        let text = text_of(&inner.edit_custom.buffer()).trim().to_string();
        let servers: Vec<String> = text
            .split('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if servers.is_empty() {
            "default".to_string()
        } else {
            servers.join(" ")
        }
    };

    write_fallback_to_yaml(&fb_val);
    if let Some(cb) = inner.fallbacks_cb.borrow().as_ref() {
        cb(&fb_val);
    }
}

fn write_fallback_to_yaml(val: &str) {
    let effective = if val.is_empty() { "default" } else { val };
    for path in [proxy_yaml(), ROOT_PROXY_YAML.to_string()] {
        if !std::path::Path::exists(&std::path::PathBuf::from(&path)) {
            continue;
        }
        if let Ok(mut c) = std::fs::read_to_string(&path) {
            if c.contains("fallbacks:") {
                let re = Regex::new(r"(fallbacks:\s*)([^\n]+)").unwrap();
                c = re
                    .replace_all(&c, |caps: &regex::Captures| format!("{}{}", &caps[1], effective))
                    .to_string();
            } else {
                c.push_str(&format!("\nfallbacks: {}\n", effective));
            }
            let _ = std::fs::write(&path, c);
        }
    }
}
