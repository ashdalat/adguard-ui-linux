// Port of adguard_modern/tabs/settings_bootstrap_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::BackButton;
use regex::Regex;

fn proxy_yaml() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml")
        .to_string_lossy()
        .to_string()
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_sub: gtk4::Label,
    lbl_note: gtk4::Label,
    lbl_input: gtk4::Label,
    edit_bs: gtk4::TextView,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    bootstraps_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

pub struct SettingsBootstrapView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsBootstrapView {
    pub fn new() -> SettingsBootstrapView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(22));

        let lbl_title = gtk4::Label::new(Some("Custom bootstrap addresses"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(6));

        let lbl_sub = gtk4::Label::new(Some(
            "IP addresses of intermediate DNS servers used to resolve the domain name of the secure DNS server selected in the app",
        ));
        lbl_sub.add_css_class("lbl-10");
        lbl_sub.set_wrap(true);
        lbl_sub.set_halign(gtk4::Align::Start);
        root.append(&lbl_sub);
        root.append(&spacer(14));

        let lbl_note = gtk4::Label::new(Some(
            "The addresses are applied in the order you list them. If all addresses are invalid or no address is specified, your ISP’s DNS addresses are used",
        ));
        lbl_note.add_css_class("lbl-10");
        lbl_note.set_wrap(true);
        lbl_note.set_halign(gtk4::Align::Start);
        root.append(&lbl_note);
        root.append(&spacer(24));

        let lbl_input = gtk4::Label::new(Some("Bootstrap addresses, one per line"));
        lbl_input.add_css_class("lbl-10-medium-white");
        lbl_input.set_halign(gtk4::Align::Start);
        root.append(&lbl_input);
        root.append(&spacer(8));

        let edit_bs = gtk4::TextView::new();
        edit_bs.set_size_request(-1, 160);
        edit_bs.add_css_class("bs-edit");
        root.append(&edit_bs);

        let bb_root = btn_back.root.clone();
        let ev = edit_bs.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_sub,
            lbl_note,
            lbl_input,
            edit_bs,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            bootstraps_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        let buf = ev.buffer();
        buf.connect_changed(move |_b| on_text_changed(&i2));

        let i3 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i3)));

        load_bootstraps(&inner);
        retranslate_ui(&inner);

        SettingsBootstrapView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_bootstraps_changed(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.bootstraps_cb.borrow_mut() = Some(cb);
    }

    pub fn load_bootstraps(&self) {
        load_bootstraps(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn load_bootstraps(inner: &Rc<Inner>) {
    let mut val = "default".to_string();
    let path = proxy_yaml();
    if std::path::Path::exists(&std::path::PathBuf::from(&path)) {
        if let Ok(c) = std::fs::read_to_string(&path) {
            if let Some(m) = Regex::new(r"bootstraps:\s*([^\n]+)").unwrap().captures(&c) {
                val = m[1].trim().to_string();
            }
        }
    }
    *inner.syncing.borrow_mut() = true;
    let buf = inner.edit_bs.buffer();
    if val == "default" || val.is_empty() {
        buf.set_text("");
    } else {
        buf.set_text(&val.split_whitespace().collect::<Vec<_>>().join("\n"));
    }
    *inner.syncing.borrow_mut() = false;
}

fn on_text_changed(inner: &Rc<Inner>) {
    if *inner.syncing.borrow() {
        return;
    }
    let buf = inner.edit_bs.buffer();
    let start = buf.start_iter();
    let end = buf.end_iter();
    let text = buf.text(&start, &end, false).trim().to_string();
    let addrs: Vec<String> = text
        .split('\n')
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();
    let val = if addrs.is_empty() {
        "default".to_string()
    } else {
        addrs.join(" ")
    };
    let path = proxy_yaml();
    if std::path::Path::exists(&std::path::PathBuf::from(&path)) {
        if let Ok(mut c) = std::fs::read_to_string(&path) {
            let re = Regex::new(r"(bootstraps:\s*)([^\n]+)").unwrap();
            c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}", &caps[1], val)).to_string();
            let _ = std::fs::write(&path, c);
        }
    }
    if let Some(cb) = inner.bootstraps_cb.borrow().as_ref() {
        cb(&val);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("bs_title"));
    inner.lbl_sub.set_text(&i18n::tr("bs_sub"));
    inner.lbl_note.set_text(&i18n::tr("bs_note"));
    inner.lbl_input.set_text(&i18n::tr("bs_input_label"));
    // Deviation: GTK4 TextView has no placeholder text, so the Python
    // placeholder "IPv4 and IPv6" cannot be rendered here.
}
