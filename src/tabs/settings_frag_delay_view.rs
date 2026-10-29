// Port of adguard_modern/tabs/settings_frag_delay_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::BackButton;
use regex::RegexBuilder;

fn proxy_yaml() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml")
        .to_string_lossy()
        .to_string()
}

const ROOT_PROXY_YAML: &str = "/root/.local/share/adguard-cli/proxy.yaml";

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_sub: gtk4::Label,
    lbl_input: gtk4::Label,
    input_delay: gtk4::Entry,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    delay_cb: RefCell<Option<Box<dyn Fn(i64)>>>,
}

pub struct SettingsFragDelayView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsFragDelayView {
    pub fn new() -> SettingsFragDelayView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(22));

        let lbl_title = gtk4::Label::new(Some("Fragmentation delay"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(6));

        let lbl_sub = gtk4::Label::new(Some(
            "Adds a delay before the second fragment of a ClientHello or HTTP request",
        ));
        lbl_sub.add_css_class("lbl-10");
        lbl_sub.set_wrap(true);
        lbl_sub.set_halign(gtk4::Align::Start);
        root.append(&lbl_sub);
        root.append(&spacer(26));

        let lbl_input = gtk4::Label::new(Some("Delay, in ms"));
        lbl_input.add_css_class("lbl-10-medium-white");
        lbl_input.set_halign(gtk4::Align::Start);
        root.append(&lbl_input);
        root.append(&spacer(8));

        let input_delay = gtk4::Entry::new();
        input_delay.set_size_request(-1, 44);
        input_delay.add_css_class("frag-input");
        root.append(&input_delay);

        let bb_root = btn_back.root.clone();
        let ip = input_delay.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_sub,
            lbl_input,
            input_delay,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            delay_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        ip.connect_changed(move |e| on_delay_changed(&i2, &e.text()));

        let i3 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i3)));

        load_delay(&inner);
        retranslate_ui(&inner);

        SettingsFragDelayView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_delay_changed(&self, cb: Box<dyn Fn(i64)>) {
        *self.inner.delay_cb.borrow_mut() = Some(cb);
    }

    pub fn load_delay(&self) {
        load_delay(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn load_delay(inner: &Rc<Inner>) {
    let mut val: i64 = 20;
    if std::path::Path::exists(&std::path::PathBuf::from(proxy_yaml())) {
        if let Ok(c) = std::fs::read_to_string(proxy_yaml()) {
            if let Some(m) = RegexBuilder::new(r"split_delay_ms:\s*([0-9]+)")
                .case_insensitive(false)
                .build()
                .unwrap()
                .captures(&c)
            {
                val = m[1].parse().unwrap_or(20);
            }
        }
    }
    *inner.syncing.borrow_mut() = true;
    inner.input_delay.set_text(&val.to_string());
    *inner.syncing.borrow_mut() = false;
}

fn on_delay_changed(inner: &Rc<Inner>, text: &str) {
    if *inner.syncing.borrow() {
        return;
    }
    let cleaned: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        return;
    }
    let val: i64 = match cleaned.parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    for path in [proxy_yaml(), ROOT_PROXY_YAML.to_string()] {
        if !std::path::Path::exists(&std::path::PathBuf::from(&path)) {
            continue;
        }
        if let Ok(mut c) = std::fs::read_to_string(&path) {
            if c.contains("split_delay_ms:") {
                let re = RegexBuilder::new(r"(split_delay_ms:\s*)([0-9]+)").build().unwrap();
                c = re.replace_all(&c, |caps: &regex::Captures| format!("{}{}", &caps[1], val)).to_string();
            } else {
                c.push_str(&format!("\nsplit_delay_ms: {}\n", val));
            }
            let _ = std::fs::write(&path, c);
        }
    }
    if let Some(cb) = inner.delay_cb.borrow().as_ref() {
        cb(val);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("frag_title"));
    inner.lbl_sub.set_text(&i18n::tr("frag_sub"));
    inner.lbl_input.set_text(&i18n::tr("frag_input_label"));
}
