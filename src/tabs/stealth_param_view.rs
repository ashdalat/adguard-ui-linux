// Port of adguard_modern/tabs/stealth_param_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::toggle_switch::ToggleSwitch;

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
    lbl_hint: gtk4::Label,
    lbl_warning: gtk4::Label,
    btn_link: gtk4::Button,
    lbl_field_title: gtk4::Label,
    input_box: gtk4::Box,
    input_field: gtk4::Entry,
    btn_clear: gtk4::Button,
    current_key: RefCell<String>,
    link_url: RefCell<String>,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    value_saved_cb: RefCell<Option<Box<dyn Fn(&str, bool, String)>>>,
}

#[derive(Clone)]
pub struct StealthParamView {
    inner: Rc<Inner>,
}

impl StealthParamView {
    pub fn new() -> StealthParamView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
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
        let lbl_title = gtk4::Label::new(Some("Param title"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Description.
        let lbl_desc = gtk4::Label::new(Some("Description of this parameter"));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        root.append(&spacer(6));

        // Explanation / hint.
        let lbl_hint = gtk4::Label::new(Some("Explanation text"));
        lbl_hint.add_css_class("lbl-10");
        lbl_hint.set_wrap(true);
        lbl_hint.set_xalign(0.0);
        root.append(&lbl_hint);

        // Optional warning note.
        let lbl_warning = gtk4::Label::new(Some(""));
        lbl_warning.add_css_class("sp-warning");
        lbl_warning.set_wrap(true);
        lbl_warning.set_xalign(0.0);
        lbl_warning.set_visible(false);
        root.append(&lbl_warning);

        // Optional external link.
        let btn_link = gtk4::Button::new();
        btn_link.set_label("");
        btn_link.add_css_class("green-link");
        btn_link.set_halign(gtk4::Align::Start);
        btn_link.set_visible(false);
        root.append(&btn_link);

        root.append(&spacer(14));

        // Field label.
        let lbl_field_title = gtk4::Label::new(Some("Field name"));
        lbl_field_title.add_css_class("sp-field-title");
        lbl_field_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_field_title);

        // Input box with clear '✕' button.
        let input_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        input_box.add_css_class("sp-input-box");
        input_box.set_margin_top(0);
        input_box.set_margin_bottom(0);
        input_box.set_margin_start(12);
        input_box.set_margin_end(10);

        let input_field = gtk4::Entry::new();
        input_field.add_css_class("sp-entry");
        input_field.set_hexpand(true);
        input_box.append(&input_field);

        let btn_clear = gtk4::Button::new();
        btn_clear.set_label("✕");
        btn_clear.set_size_request(22, 22);
        btn_clear.add_css_class("sp-clear-btn");
        input_box.append(&btn_clear);
        root.append(&input_box);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            lbl_hint,
            lbl_warning,
            btn_link,
            lbl_field_title,
            input_box,
            input_field,
            btn_clear,
            current_key: RefCell::new(String::new()),
            link_url: RefCell::new(String::new()),
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            value_saved_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        // Python disconnects/reconnects the link click per set_config; a single
        // closure reading the stored URL is equivalent.
        let i2 = inner.clone();
        inner.btn_link.connect_clicked(move |_| {
            let url = i2.link_url.borrow().clone();
            if !url.is_empty() {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
        });

        let i3 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_switch_toggled(&i3, val)));

        let i4 = inner.clone();
        inner.input_field.connect_changed(move |e| on_text_changed(&i4, e.text().as_str()));

        let entry = inner.input_field.clone();
        inner.btn_clear.connect_clicked(move |_| entry.set_text(""));

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        StealthParamView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_value_saved(&self, cb: Box<dyn Fn(&str, bool, String)>) {
        *self.inner.value_saved_cb.borrow_mut() = Some(cb);
    }

    pub fn set_config(
        &self,
        key: &str,
        title: &str,
        desc: &str,
        hint: &str,
        field_label: &str,
        initial_val: &str,
        is_active: bool,
        warning: &str,
        link_text: &str,
        link_url: &str,
        placeholder: &str,
    ) {
        *self.inner.syncing.borrow_mut() = true;
        *self.inner.current_key.borrow_mut() = key.to_string();
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
        self.inner.lbl_hint.set_text(hint);
        self.inner.lbl_field_title.set_text(field_label);
        self.inner.input_field.set_placeholder_text(Some(placeholder));
        self.inner.input_field.set_text(initial_val);
        self.inner.master_switch.set_checked(is_active, false);

        if !warning.is_empty() {
            self.inner.lbl_warning.set_text(warning);
            self.inner.lbl_warning.set_visible(true);
        } else {
            self.inner.lbl_warning.set_visible(false);
        }

        if !link_text.is_empty() && !link_url.is_empty() {
            self.inner.btn_link.set_label(link_text);
            *self.inner.link_url.borrow_mut() = link_url.to_string();
            self.inner.btn_link.set_visible(true);
        } else {
            self.inner.btn_link.set_visible(false);
        }

        self.inner.input_box.set_sensitive(is_active);
        self.inner.input_box.remove_css_class("sp-input-box");
        if is_active {
            self.inner.input_box.add_css_class("sp-input-box");
        } else {
            self.inner.input_box.add_css_class("sp-input-box-dim");
        }

        retranslate_ui(&self.inner);
        *self.inner.syncing.borrow_mut() = false;
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn on_switch_toggled(inner: &Rc<Inner>, val: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    inner.input_box.set_sensitive(val);
    let key = inner.current_key.borrow().clone();
    let text = inner.input_field.text().trim().to_string();
    if let Some(cb) = inner.value_saved_cb.borrow().as_ref() {
        cb(&key, val, text);
    }
}

fn on_text_changed(inner: &Rc<Inner>, text: &str) {
    if *inner.syncing.borrow() {
        return;
    }
    let key = inner.current_key.borrow().clone();
    let is_active = inner.master_switch.is_checked();
    if let Some(cb) = inner.value_saved_cb.borrow().as_ref() {
        cb(&key, is_active, text.trim().to_string());
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    let key = inner.current_key.borrow().clone();
    if key.is_empty() {
        return;
    }
    match key.as_str() {
        "third_party_cookies" => {
            inner.lbl_title.set_text(&i18n::tr("cs_row_third_party_cookies_t"));
            inner.lbl_desc.set_text(&i18n::tr("cs_row_third_party_cookies_d"));
            inner.lbl_hint.set_text(&i18n::tr("sp_hint_cookies"));
            inner.lbl_field_title.set_text(&i18n::tr("sp_field_lifetime"));
            inner.lbl_warning.set_text(&i18n::tr("sp_warn_third_cookies"));
        }
        "first_party_cookies" => {
            inner.lbl_title.set_text(&i18n::tr("cs_row_first_party_cookies_t"));
            inner.lbl_desc.set_text(&i18n::tr("cs_row_first_party_cookies_d"));
            inner.lbl_hint.set_text(&i18n::tr("sp_hint_cookies"));
            inner.lbl_field_title.set_text(&i18n::tr("sp_field_lifetime"));
            inner.lbl_warning.set_text(&i18n::tr("sp_warn_first_cookies"));
        }
        "referer" => {
            inner.lbl_title.set_text(&i18n::tr("cs_row_referer_t"));
            inner.lbl_desc.set_text(&i18n::tr("cs_row_referer_d"));
            inner.lbl_hint.set_text(&i18n::tr("sp_hint_referer"));
            inner.lbl_field_title.set_text(&i18n::tr("sp_field_referer"));
            inner.btn_link.set_label(&i18n::tr("sp_link_referer"));
        }
        "user_agent" => {
            inner.lbl_title.set_text(&i18n::tr("cs_row_user_agent_t"));
            inner.lbl_desc.set_text(&i18n::tr("cs_row_user_agent_d"));
            inner.lbl_hint.set_text(&i18n::tr("sp_hint_ua"));
            inner.lbl_field_title.set_text(&i18n::tr("sp_field_ua"));
            inner.btn_link.set_label(&i18n::tr("sp_link_ua"));
        }
        "mask_ip" => {
            inner.lbl_title.set_text(&i18n::tr("cs_row_mask_ip_t"));
            inner.lbl_desc.set_text(&i18n::tr("cs_row_mask_ip_d"));
            inner.lbl_field_title.set_text(&i18n::tr("sp_field_ip"));
            inner.btn_link.set_label(&i18n::tr("sp_link_ip"));
        }
        _ => {}
    }
}
