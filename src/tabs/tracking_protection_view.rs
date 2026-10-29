// Port of adguard_modern/tabs/tracking_protection_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::backend;
use crate::i18n;
use crate::widgets::privacy_level_row::PrivacyLevelRow;
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
    btn_how_it_works: gtk4::Button,
    lbl_section: gtk4::Label,
    rows: HashMap<String, PrivacyLevelRow>,
    current_level: RefCell<String>,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct TrackingProtectionView {
    inner: Rc<Inner>,
}

impl TrackingProtectionView {
    pub fn new() -> TrackingProtectionView {
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

        // Title row: Tracking protection + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Tracking protection"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some(
            "Protects your online activity and personal data from web trackers",
        ));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        // Link: How it works.
        let btn_how_it_works = gtk4::Button::new();
        btn_how_it_works.set_label("How it works  ↗");
        btn_how_it_works.add_css_class("green-link");
        btn_how_it_works.set_halign(gtk4::Align::Start);
        root.append(&btn_how_it_works);

        root.append(&spacer(10));

        // Section header: Privacy protection level.
        let lbl_section = gtk4::Label::new(Some("Privacy protection level"));
        lbl_section.add_css_class("lbl-12-bold-white");
        lbl_section.set_halign(gtk4::Align::Start);
        root.append(&lbl_section);

        // Scrollable cards container.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        container.add_css_class("transparent-container");
        container.set_margin_top(4);
        container.set_margin_end(8);
        container.set_margin_bottom(10);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        let mut rows: HashMap<String, PrivacyLevelRow> = HashMap::new();
        for (id, title, desc, warning, drilldown) in [
            ("standard", "Standard", "Blocks trackers and analytics systems", "", false),
            (
                "high",
                "High",
                "Removes tracking parameters from page URLs. Limits the lifetime of third-party cookies",
                "You may have to re-log in to some websites or face other cookie-related issues",
                false,
            ),
            (
                "extreme",
                "Extreme",
                "Limits the lifetime of third-party cookies. Blocks browser APIs",
                "Some websites may get broken or work incorrectly",
                false,
            ),
            (
                "custom",
                "Custom",
                "Allows to set your own tracking protection parameters",
                "",
                true,
            ),
        ] {
            let row = PrivacyLevelRow::new(id, title, desc, warning, drilldown);
            container.append(row.root());
            rows.insert(id.to_string(), row);
        }

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            btn_how_it_works,
            lbl_section,
            rows,
            current_level: RefCell::new("custom".to_string()),
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let hw = inner.btn_how_it_works.clone();
        hw.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/stealth-mode/")
                .spawn();
        });

        let i2 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggled(&i2, val)));

        for row in inner.rows.values() {
            let i = inner.clone();
            row.connect_clicked(Box::new(move |level_id| select_level(&i, level_id)));
        }

        let i3 = inner.clone();
        if let Some(row) = inner.rows.get("custom") {
            row.connect_drilldown_requested(Box::new(move || {
                if let Some(cb) = i3.drilldown_cb.borrow().as_ref() {
                    cb();
                }
            }));
        }

        let i4 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i4)));

        // Default active level: Custom.
        select_level(&inner, "custom");
        retranslate_ui(&inner);

        TrackingProtectionView { inner }
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

    pub fn connect_custom_drilldown_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn set_active(&self, is_active: bool) {
        *self.inner.syncing.borrow_mut() = true;
        self.inner.master_switch.set_checked(is_active, false);
        for row in self.inner.rows.values() {
            // Python's set_active() calls row.setEnabled(is_active) for every row;
            // the level rows also need the dimming so an off master switch is
            // visibly reflected in the four options below it.
            row.set_active_state(is_active);
        }
        *self.inner.syncing.borrow_mut() = false;
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_arrow_back"));
    inner.lbl_title.set_text(&i18n::tr("tp_title"));
    inner.lbl_desc.set_text(&i18n::tr("tp_desc"));
    inner.btn_how_it_works.set_label(&i18n::tr("tp_how_it_works"));
    inner.lbl_section.set_text(&i18n::tr("tp_level_section"));
    if let Some(row) = inner.rows.get("standard") {
        row.update_text(&i18n::tr("tp_std_t"), &i18n::tr("tp_std_d"), "");
    }
    if let Some(row) = inner.rows.get("high") {
        row.update_text(&i18n::tr("tp_high_t"), &i18n::tr("tp_high_d"), &i18n::tr("tp_high_w"));
    }
    if let Some(row) = inner.rows.get("extreme") {
        row.update_text(
            &i18n::tr("tp_extreme_t"),
            &i18n::tr("tp_extreme_d"),
            &i18n::tr("tp_extreme_w"),
        );
    }
    if let Some(row) = inner.rows.get("custom") {
        row.update_text(&i18n::tr("tp_custom_t"), &i18n::tr("tp_custom_d"), "");
    }
}

fn select_level(inner: &Rc<Inner>, level_id: &str) {
    *inner.current_level.borrow_mut() = level_id.to_string();
    for (k, row) in inner.rows.iter() {
        row.set_checked(k == level_id);
    }

    // Apply settings via backend.
    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    if level_id == "standard" {
        backend::run_cmd_async(
            "adguard-cli config set stealthmode.enabled true && adguard-cli config set stealthmode.block_third_party_cookies false && adguard-cli config set stealthmode.block_web_rtc false && adguard-cli config set stealthmode.hide_user_agent false && adguard-cli config set stealthmode.remove_referrer_from_third_party_requests false && adguard-cli config set stealthmode.hide_search_queries false && adguard-cli config set stealthmode.block_browser_push_api false && adguard-cli config set stealthmode.block_browser_location_api false && adguard-cli filters enable 3",
            noop,
        );
    } else if level_id == "high" {
        backend::run_cmd_async(
            "adguard-cli config set stealthmode.enabled true && adguard-cli config set stealthmode.block_third_party_cookies true && adguard-cli config set stealthmode.block_third_party_cookies_min 180 && adguard-cli config set stealthmode.remove_referrer_from_third_party_requests true && adguard-cli config set stealthmode.hide_search_queries true && adguard-cli config set stealthmode.disable_third_party_cache true && adguard-cli config set stealthmode.remove_x_client_data_header true && adguard-cli filters enable 3 && adguard-cli filters enable 17",
            noop,
        );
    } else if level_id == "extreme" {
        backend::run_cmd_async(
            "adguard-cli config set stealthmode.enabled true && adguard-cli config set stealthmode.block_third_party_cookies true && adguard-cli config set stealthmode.block_third_party_cookies_min 0 && adguard-cli config set stealthmode.block_web_rtc true && adguard-cli config set stealthmode.block_browser_push_api true && adguard-cli config set stealthmode.block_browser_location_api true && adguard-cli config set stealthmode.hide_user_agent true && adguard-cli config set stealthmode.remove_referrer_from_third_party_requests true && adguard-cli config set stealthmode.block_third_party_authorization true && adguard-cli filters enable 3 && adguard-cli filters enable 17",
            noop,
        );
    } else if level_id == "custom" {
        backend::run_cmd_async("adguard-cli config set stealthmode.enabled true", noop);
    }
}

fn on_master_toggled(inner: &Rc<Inner>, val: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(val);
    }

    // Dim or enable level rows.
    for row in inner.rows.values() {
        row.root().set_sensitive(val);
        row.root().set_opacity(if val { 1.0 } else { 0.5 });
    }

    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    let cmd = if val {
        "adguard-cli config set stealthmode.enabled true"
    } else {
        "adguard-cli config set stealthmode.enabled false"
    };
    backend::run_cmd_async(cmd, noop);
}
