// Port of adguard_modern/tabs/anti_dpi_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::toggle_switch::ToggleSwitch;

#[derive(Clone)]
pub struct AntiDpiConfig {
    pub enabled: bool,
    pub client_hello: i64,
    pub http_split: i64,
    pub space_juggling: bool,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    lbl_ch: gtk4::Label,
    box_ch: gtk4::Box,
    input_ch: gtk4::Entry,
    lbl_http: gtk4::Label,
    box_http: gtk4::Box,
    input_http: gtk4::Entry,
    lbl_jug_title: gtk4::Label,
    lbl_jug_desc: gtk4::Label,
    lbl_jug_warn: gtk4::Label,
    switch_juggling: ToggleSwitch,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    config_changed_cb: RefCell<Option<Box<dyn Fn(&AntiDpiConfig)>>>,
}

#[derive(Clone)]
pub struct AntiDpiView {
    inner: Rc<Inner>,
}

impl AntiDpiView {
    pub fn new() -> AntiDpiView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("transparent-container");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: '‹ Back'.
        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = gtk4::Button::new();
        btn_back.set_label("‹ Back");
        btn_back.add_css_class("tp-back-btn");
        top_bar.append(&btn_back);
        root.append(&top_bar);

        // Title row: title + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Protect against DPI"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Description.
        let lbl_desc = gtk4::Label::new(Some(
            "Modifies your outgoing traffic so that your ISP's Deep Packet Inspection systems can't detect the websites you visit. Can't guarantee complete protection against all DPI systems",
        ));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        root.append(&spacer(10));

        // Input 1: ClientHello fragment size.
        let lbl_ch = gtk4::Label::new(Some("Size of the first ClientHello fragment in bytes (from 1 to 1500)"));
        lbl_ch.add_css_class("sp-field-title");
        lbl_ch.set_halign(gtk4::Align::Start);
        root.append(&lbl_ch);

        let (box_ch, input_ch) = create_input_box(&root, "1");

        root.append(&spacer(8));

        // Input 2: HTTP request fragment size.
        let lbl_http = gtk4::Label::new(Some("Size of the first HTTP request fragment in bytes (from 1 to 1500)"));
        lbl_http.add_css_class("sp-field-title");
        lbl_http.set_halign(gtk4::Align::Start);
        root.append(&lbl_http);

        let (box_http, input_http) = create_input_box(&root, "1");

        root.append(&spacer(14));

        // Sub-option: HTTP space juggling.
        let juggling_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        let jug_text = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        jug_text.set_hexpand(true);

        let lbl_jug_title = gtk4::Label::new(Some("HTTP space juggling"));
        lbl_jug_title.add_css_class("lbl-title");
        lbl_jug_title.set_halign(gtk4::Align::Start);
        jug_text.append(&lbl_jug_title);

        let lbl_jug_desc = gtk4::Label::new(Some(
            "Modifies spacing in HTTP requests to prevent DPI systems from recognizing them",
        ));
        lbl_jug_desc.add_css_class("lbl-desc-9");
        lbl_jug_desc.set_xalign(0.0);
        jug_text.append(&lbl_jug_desc);

        let lbl_jug_warn = gtk4::Label::new(Some("Can break access to some unencrypted websites"));
        lbl_jug_warn.add_css_class("adp-warn");
        lbl_jug_warn.set_xalign(0.0);
        jug_text.append(&lbl_jug_warn);

        juggling_row.append(&jug_text);

        let switch_juggling = ToggleSwitch::new(46, 26);
        switch_juggling.set_checked(false, false);
        switch_juggling.root().set_valign(gtk4::Align::Center);
        juggling_row.append(switch_juggling.root());
        root.append(&juggling_row);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            lbl_ch,
            box_ch,
            input_ch,
            lbl_http,
            box_http,
            input_http,
            lbl_jug_title,
            lbl_jug_desc,
            lbl_jug_warn,
            switch_juggling,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            config_changed_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |_val| on_values_changed(&i2)));

        let i3 = inner.clone();
        inner.switch_juggling.connect_toggled(Box::new(move |_val| on_values_changed(&i3)));

        // QIntValidator(1, 1500) approximation: strip non-digits on change.
        for entry in [&inner.input_ch, &inner.input_http] {
            let i = inner.clone();
            let e = entry.clone();
            entry.connect_changed(move |e2| {
                if *i.syncing.borrow() {
                    return;
                }
                let raw = e2.text().to_string();
                let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
                if digits != raw {
                    *i.syncing.borrow_mut() = true;
                    e.set_text(&digits);
                    *i.syncing.borrow_mut() = false;
                }
                on_values_changed(&i);
            });
        }

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        retranslate_ui(&inner);

        AntiDpiView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_config_changed(&self, cb: Box<dyn Fn(&AntiDpiConfig)>) {
        *self.inner.config_changed_cb.borrow_mut() = Some(cb);
    }

    pub fn set_config(&self, enabled: bool, client_hello: i64, http_split: i64, space_juggling: bool) {
        *self.inner.syncing.borrow_mut() = true;
        self.inner.master_switch.set_checked(enabled, false);
        self.inner.input_ch.set_text(&client_hello.to_string());
        self.inner.input_http.set_text(&http_split.to_string());
        self.inner.switch_juggling.set_checked(space_juggling, false);

        set_inputs_enabled(&self.inner, enabled);
        *self.inner.syncing.borrow_mut() = false;
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn create_input_box(root: &gtk4::Box, default_val: &str) -> (gtk4::Box, gtk4::Entry) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    box_.add_css_class("sp-input-box");
    box_.set_margin_start(12);
    box_.set_margin_end(10);

    let entry = gtk4::Entry::new();
    entry.set_text(default_val);
    entry.add_css_class("sp-entry");
    entry.set_hexpand(true);
    box_.append(&entry);

    let btn_clear = gtk4::Button::new();
    btn_clear.set_label("✕");
    btn_clear.set_size_request(22, 22);
    btn_clear.add_css_class("sp-clear-btn");
    box_.append(&btn_clear);

    let e = entry.clone();
    btn_clear.connect_clicked(move |_| e.set_text(""));

    root.append(&box_);
    (box_, entry)
}

fn set_inputs_enabled(inner: &Rc<Inner>, enabled: bool) {
    for box_ in [&inner.box_ch, &inner.box_http] {
        box_.set_sensitive(enabled);
        box_.remove_css_class("sp-input-box");
        box_.remove_css_class("sp-input-box-dim");
        if enabled {
            box_.add_css_class("sp-input-box");
        } else {
            box_.add_css_class("sp-input-box-dim");
        }
    }
    inner.switch_juggling.root().set_sensitive(enabled);
}

fn on_values_changed(inner: &Rc<Inner>) {
    if *inner.syncing.borrow() {
        return;
    }
    let enabled = inner.master_switch.is_checked();
    set_inputs_enabled(inner, enabled);

    let ch_val = parse_digit(inner.input_ch.text().as_str());
    let http_val = parse_digit(inner.input_http.text().as_str());
    let jug_val = inner.switch_juggling.is_checked();

    let cfg = AntiDpiConfig {
        enabled,
        client_hello: ch_val,
        http_split: http_val,
        space_juggling: jug_val,
    };
    if let Some(cb) = inner.config_changed_cb.borrow().as_ref() {
        cb(&cfg);
    }
}

fn parse_digit(text: &str) -> i64 {
    if !text.is_empty() && text.chars().all(|c| c.is_ascii_digit()) {
        text.parse::<i64>().unwrap_or(1)
    } else {
        1
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("anti_dpi_title"));
    inner.lbl_desc.set_text(&i18n::tr("anti_dpi_desc"));
    inner.lbl_ch.set_text(&i18n::tr("anti_dpi_ch_label"));
    inner.lbl_http.set_text(&i18n::tr("anti_dpi_http_label"));
    inner.lbl_jug_title.set_text(&i18n::tr("anti_dpi_jug_t"));
    inner.lbl_jug_desc.set_text(&i18n::tr("anti_dpi_jug_d"));
    inner.lbl_jug_warn.set_text(&i18n::tr("anti_dpi_jug_w"));
}
