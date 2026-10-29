// Port of adguard_modern/tabs/custom_stealth_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::backend;
use crate::i18n;
use crate::widgets::toggle_switch::ToggleSwitch;

const STEALTH_PRESETS: [(&str, [(&str, bool); 16]); 3] = [
    (
        "standard",
        [
            ("block_trackers", true),
            ("remove_url_tracking", false),
            ("hide_search_queries", false),
            ("dnt", true),
            ("third_party_cookies", false),
            ("first_party_cookies", false),
            ("etag", false),
            ("auth_header", false),
            ("webrtc", false),
            ("push_api", false),
            ("location_api", false),
            ("referer", false),
            ("user_agent", false),
            ("mask_ip", false),
            ("x_client_data", false),
            ("anti_dpi", false),
        ],
    ),
    (
        "high",
        [
            ("block_trackers", true),
            ("remove_url_tracking", true),
            ("hide_search_queries", true),
            ("dnt", true),
            ("third_party_cookies", true),
            ("first_party_cookies", false),
            ("etag", true),
            ("auth_header", false),
            ("webrtc", false),
            ("push_api", false),
            ("location_api", false),
            ("referer", true),
            ("user_agent", false),
            ("mask_ip", false),
            ("x_client_data", true),
            ("anti_dpi", false),
        ],
    ),
    (
        "extreme",
        [
            ("block_trackers", true),
            ("remove_url_tracking", true),
            ("hide_search_queries", true),
            ("dnt", true),
            ("third_party_cookies", true),
            ("first_party_cookies", false),
            ("etag", true),
            ("auth_header", true),
            ("webrtc", true),
            ("push_api", true),
            ("location_api", true),
            ("referer", true),
            ("user_agent", true),
            ("mask_ip", false),
            ("x_client_data", true),
            ("anti_dpi", false),
        ],
    ),
];

// ---------------------------------------------------------------------------
// StealthSettingRow
// ---------------------------------------------------------------------------
struct StealthRowInner {
    key: String,
    has_subpage: bool,
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    lbl_param: Option<gtk4::Label>,
    lbl_warn: Option<gtk4::Label>,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct StealthSettingRow {
    inner: Rc<StealthRowInner>,
}

impl StealthSettingRow {
    pub fn new(
        key: &str,
        title: &str,
        desc: &str,
        checked: bool,
        param_text: &str,
        warning_text: &str,
        has_subpage: bool,
    ) -> StealthSettingRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("stealth-setting-row");
        if has_subpage {
            root.add_css_class("stealth-row-clickable");
        }
        root.set_margin_top(10);
        root.set_margin_bottom(10);
        root.set_margin_start(12);
        root.set_margin_end(16);

        // Text column.
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(desc));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        text_box.append(&lbl_desc);

        let lbl_param: Option<gtk4::Label> = if !param_text.is_empty() {
            let p = gtk4::Label::new(Some(param_text));
            p.add_css_class("lbl-9-bold-white");
            p.set_halign(gtk4::Align::Start);
            p.set_margin_top(2);
            text_box.append(&p);
            Some(p)
        } else {
            None
        };

        let lbl_warn: Option<gtk4::Label> = if !warning_text.is_empty() {
            let w = gtk4::Label::new(Some(warning_text));
            w.add_css_class("lbl-warning");
            w.set_wrap(true);
            w.set_xalign(0.0);
            w.set_margin_top(2);
            text_box.append(&w);
            Some(w)
        } else {
            None
        };

        root.append(&text_box);

        // Optional vertical line + chevron (drilldown rows).
        if has_subpage {
            let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            divider.add_css_class("divider-v");
            divider.set_size_request(1, 26);
            divider.set_valign(gtk4::Align::Center);
            root.append(&divider);

            let btn_arrow = gtk4::Button::new();
            btn_arrow.set_label("›");
            btn_arrow.set_size_request(24, 24);
            btn_arrow.add_css_class("chevron-label");
            btn_arrow.set_valign(gtk4::Align::Center);
            root.append(&btn_arrow);

            let i = Rc::new(StealthRowInner {
                key: key.to_string(),
                has_subpage,
                root: root.clone(),
                lbl_title: lbl_title.clone(),
                lbl_desc: lbl_desc.clone(),
                lbl_param: lbl_param.clone(),
                lbl_warn: lbl_warn.clone(),
                switch: ToggleSwitch::new(46, 26),
                toggled_cb: RefCell::new(None),
                drilldown_cb: RefCell::new(None),
            });

            // Python: arrow button click and left-click anywhere on the row
            // both emit drilldown_clicked.
            let i1 = i.clone();
            btn_arrow.connect_clicked(move |_| {
                if let Some(cb) = i1.drilldown_cb.borrow().as_ref() {
                    cb(&i1.key);
                }
            });

            let i2 = i.clone();
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_g, n, _x, _y| {
                if n == 1 && i2.has_subpage {
                    if let Some(cb) = i2.drilldown_cb.borrow().as_ref() {
                        cb(&i2.key);
                    }
                }
            });
            i.root.add_controller(gesture);

            let sw = i.switch.clone();
            let i3 = i.clone();
            sw.connect_toggled(Box::new(move |val| {
                if let Some(cb) = i3.toggled_cb.borrow().as_ref() {
                    cb(&i3.key, val);
                }
            }));

            i.switch.set_checked(checked, false);
            i.switch.root().set_valign(gtk4::Align::Center);
            i.root.append(i.switch.root());

            return StealthSettingRow { inner: i };
        }

        // Non-subpage rows: spacing then switch.
        let sp = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        sp.set_size_request(6, -1);
        root.append(&sp);

        let switch = ToggleSwitch::new(46, 26);
        switch.set_checked(checked, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(StealthRowInner {
            key: key.to_string(),
            has_subpage,
            root,
            lbl_title,
            lbl_desc,
            lbl_param,
            lbl_warn,
            switch,
            toggled_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(&i.key, val);
            }
        }));

        StealthSettingRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_text(&self, title: &str, desc: &str, warning_text: Option<&str>) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
        if let Some(w) = self.inner.lbl_warn.as_ref() {
            if let Some(t) = warning_text {
                w.set_text(t);
            }
        }
    }

    pub fn update_param_text(&self, text: &str) {
        if let Some(p) = self.inner.lbl_param.as_ref() {
            p.set_text(text);
        }
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        self.inner.switch.set_checked(checked, animate);
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_drilldown_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }


    pub fn is_checked(&self) -> bool {
        self.inner.switch.is_checked()
    }
}

// ---------------------------------------------------------------------------
// CustomStealthView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    sec_headers: RefCell<HashMap<String, gtk4::Label>>,
    rows: RefCell<HashMap<String, StealthSettingRow>>,
    badge_values: RefCell<HashMap<String, String>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    param_drilldown_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct CustomStealthView {
    inner: Rc<Inner>,
}

impl CustomStealthView {
    pub fn new() -> CustomStealthView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        root.add_css_class("transparent-container");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: '‹ Back' + '⋮' menu.
        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = gtk4::Button::new();
        btn_back.set_label("‹ Back");
        btn_back.add_css_class("tp-back-btn");
        top_bar.append(&btn_back);

        // Python: top_bar.addStretch() — keeps ⋮ pinned to the right edge
        let top_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_spacer.set_hexpand(true);
        top_bar.append(&top_spacer);

        let btn_more = gtk4::Button::new();
        btn_more.set_label("⋮");
        btn_more.set_size_request(28, 28);
        btn_more.add_css_class("tp-back-btn");
        top_bar.append(&btn_more);
        root.append(&top_bar);

        // Title.
        let lbl_title = gtk4::Label::new(Some("Custom"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some("Allows you to set your own tracking protection parameters"));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        // Scroll area for sections.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        container.add_css_class("transparent-container");
        container.set_margin_top(4);
        container.set_margin_end(12);
        container.set_margin_bottom(20);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            lbl_desc,
            sec_headers: RefCell::new(HashMap::new()),
            rows: RefCell::new(HashMap::new()),
            badge_values: RefCell::new(HashMap::new()),
            back_cb: RefCell::new(None),
            param_drilldown_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let bm = inner.btn_more.clone();
        let i2 = inner.clone();
        bm.connect_clicked(move |_| show_reset_menu(&i2));

        // Section 1: General.
        add_section_header(&inner, &container, "sec_general", "General");
        add_row(&inner, &container, "block_trackers", "Block trackers", "Blocks trackers and web analytics using AdGuard Tracking Protection filter", true, "", "", false);
        add_row(&inner, &container, "remove_url_tracking", "Remove tracking parameters from URLs", "Removes tracking parameters from page URLs using AdGuard URL Tracking filter", true, "", "", false);
        add_row(&inner, &container, "hide_search_queries", "Hide search queries", "Hides your searches from websites visited through search engine results", true, "", "", false);
        add_row(&inner, &container, "dnt", "Ask websites not to track me", "Sends the Global Privacy Control and Do Not Track signals to the websites you visit", true, "", "", false);

        // Section 2: Tracking methods.
        add_section_header(&inner, &container, "sec_methods", "Tracking methods");
        add_row(&inner, &container, "third_party_cookies", "Delete third-party cookies", "Prevents advertisers from tracking your behavior across multiple pages by limiting the lifetime of third-party cookies", true, "Current lifetime: 180 min", "", true);
        add_row(&inner, &container, "first_party_cookies", "Delete first-party cookies (not recommended)", "Prevents websites from remembering your details, such as your login credentials and language preferences, by limiting the lifetime of first-party cookies", false, "Current lifetime: 4320 min", "", true);
        add_row(&inner, &container, "etag", "Block ETag and If-None-Match headers", "Removes the ETag and If-None-Match headers from requests and responses to prevent tracking by them", true, "", "", false);
        add_row(&inner, &container, "auth_header", "Block third-party Authorization header", "Disables cached authorization data that can be used for tracking", true, "", "May interfere with the functionality of some browser extensions and websites", false);

        // Section 3: Browser API.
        add_section_header(&inner, &container, "sec_browser_api", "Browser API");
        add_row(&inner, &container, "webrtc", "Prevent IP address leaks through WebRTC", "Makes WebRTC use a stricter IP address policy to help keep your real IP address hidden when you use a proxy or VPN", true, "", "", false);
        add_row(&inner, &container, "push_api", "Block Push API", "Blocks push notifications from websites", true, "", "", false);
        add_row(&inner, &container, "location_api", "Block Location API", "Prevents websites from detecting your location", true, "", "", false);

        // Note: "Windows tracking" section is removed completely for Linux.

        // Section 4: Miscellaneous.
        add_section_header(&inner, &container, "sec_misc", "Miscellaneous");
        add_row(&inner, &container, "referer", "Hide Referer from third parties", "Prevents third parties from knowing what websites you visit", true, "Custom Referer: Default", "", true);
        add_row(&inner, &container, "user_agent", "Hide User-Agent", "Removes identifying information from the User-Agent header", true, "Custom User-Agent: Default", "", true);
        add_row(&inner, &container, "mask_ip", "Mask IP address", "Tricks websites into believing you are a proxy with the IP address below", true, "Custom IP address: 208.54.104.184", "", true);
        add_row(&inner, &container, "x_client_data", "Remove X-Client-Data header", "Blocks Google Chrome from sending its version and modifications information to Google domains", true, "", "", false);
        add_row(&inner, &container, "anti_dpi", "Protect against DPI", "Modifies your outgoing traffic so that your ISP's Deep Packet Inspection systems can't detect the websites you visit. Can't guarantee complete protection against all DPI systems", false, "", "", true);

        let i3 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i3)));

        retranslate_ui(&inner);

        CustomStealthView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_param_drilldown_requested(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.param_drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn update_row_badge(&self, key: &str, is_active: bool, val: &str) {
        self.inner.badge_values.borrow_mut().insert(key.to_string(), val.to_string());
        if let Some(row) = self.inner.rows.borrow().get(key) {
            row.set_checked(is_active, false);
            apply_row_badge_text(&self.inner, key, val);
        }
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn row(&self, key: &str) -> Option<StealthSettingRow> {
        self.inner.rows.borrow().get(key).cloned()
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("cs_title"));
    inner.lbl_desc.set_text(&i18n::tr("cs_desc"));

    let sec_keys = [
        ("sec_general", "cs_sec_general"),
        ("sec_methods", "cs_sec_methods"),
        ("sec_browser_api", "cs_sec_browser_api"),
        ("sec_misc", "cs_sec_misc"),
    ];
    for (s_id, s_key) in sec_keys {
        if let Some(lbl) = inner.sec_headers.borrow().get(s_id) {
            lbl.set_text(&i18n::tr(s_key));
        }
    }

    let row_defs: [(&str, &str, &str, Option<&str>); 16] = [
        ("block_trackers", "cs_row_block_trackers_t", "cs_row_block_trackers_d", None),
        ("remove_url_tracking", "cs_row_remove_url_tracking_t", "cs_row_remove_url_tracking_d", None),
        ("hide_search_queries", "cs_row_hide_search_queries_t", "cs_row_hide_search_queries_d", None),
        ("dnt", "cs_row_dnt_t", "cs_row_dnt_d", None),
        ("third_party_cookies", "cs_row_third_party_cookies_t", "cs_row_third_party_cookies_d", None),
        ("first_party_cookies", "cs_row_first_party_cookies_t", "cs_row_first_party_cookies_d", None),
        ("etag", "cs_row_etag_t", "cs_row_etag_d", None),
        ("auth_header", "cs_row_auth_header_t", "cs_row_auth_header_d", Some("cs_row_auth_header_w")),
        ("webrtc", "cs_row_webrtc_t", "cs_row_webrtc_d", None),
        ("push_api", "cs_row_push_api_t", "cs_row_push_api_d", None),
        ("location_api", "cs_row_location_api_t", "cs_row_location_api_d", None),
        ("referer", "cs_row_referer_t", "cs_row_referer_d", None),
        ("user_agent", "cs_row_user_agent_t", "cs_row_user_agent_d", None),
        ("mask_ip", "cs_row_mask_ip_t", "cs_row_mask_ip_d", None),
        ("x_client_data", "cs_row_x_client_data_t", "cs_row_x_client_data_d", None),
        ("anti_dpi", "cs_row_anti_dpi_t", "cs_row_anti_dpi_d", None),
    ];
    for (k, t_k, d_k, w_k) in row_defs {
        if let Some(row) = inner.rows.borrow().get(k) {
            let w = w_k.map(|w| i18n::tr(w));
            row.update_text(&i18n::tr(t_k), &i18n::tr(d_k), w.as_deref());
        }
    }

    // Refresh badges in active language.
    let badges: Vec<(String, String)> = inner
        .badge_values
        .borrow()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    for (k, val) in badges {
        apply_row_badge_text(inner, &k, &val);
    }
}

fn apply_row_badge_text(inner: &Rc<Inner>, key: &str, val: &str) {
    let rows = inner.rows.borrow();
    let row = match rows.get(key as &str) {
        Some(r) => r,
        None => return,
    };
    match key {
        "third_party_cookies" => {
            let v = if val.is_empty() { "180" } else { val };
            row.update_param_text(&i18n::tr("cs_badge_lifetime").replace("{val}", v));
        }
        "first_party_cookies" => {
            let v = if val.is_empty() { "4320" } else { val };
            row.update_param_text(&i18n::tr("cs_badge_lifetime").replace("{val}", v));
        }
        "referer" => {
            let v = if val.is_empty() { "Default" } else { val };
            row.update_param_text(&i18n::tr("cs_badge_referer").replace("{val}", v));
        }
        "user_agent" => {
            let v = if val.is_empty() { "Default" } else { val };
            row.update_param_text(&i18n::tr("cs_badge_ua").replace("{val}", v));
        }
        "mask_ip" => {
            let v = if val.is_empty() { "208.54.104.184" } else { val };
            row.update_param_text(&i18n::tr("cs_badge_ip").replace("{val}", v));
        }
        _ => {}
    }
}

fn add_section_header(inner: &Rc<Inner>, container: &gtk4::Box, sec_id: &str, text: &str) {
    let lbl = gtk4::Label::new(Some(text));
    lbl.add_css_class("lbl-13-bold-white");
    lbl.set_halign(gtk4::Align::Start);
    lbl.set_margin_top(10);
    container.append(&lbl);
    inner.sec_headers.borrow_mut().insert(sec_id.to_string(), lbl);
}

fn add_row(
    inner: &Rc<Inner>,
    container: &gtk4::Box,
    key: &str,
    title: &str,
    desc: &str,
    checked: bool,
    param_text: &str,
    warning_text: &str,
    has_subpage: bool,
) {
    let row = StealthSettingRow::new(key, title, desc, checked, param_text, warning_text, has_subpage);

    let i = inner.clone();
    row.connect_toggled(Box::new(move |k, is_checked| on_setting_toggled(&i, k, is_checked)));

    if has_subpage {
        let i = inner.clone();
        row.connect_drilldown_clicked(Box::new(move |k| {
            if let Some(cb) = i.param_drilldown_cb.borrow().as_ref() {
                cb(k);
            }
        }));
    }

    container.append(row.root());
    inner.rows.borrow_mut().insert(key.to_string(), row);
}

fn cli_map_entry(key: &str, is_checked: bool) -> Option<String> {
    let b = if is_checked { "true" } else { "false" };
    let cmd = match key {
        "block_trackers" => format!("filters {} 3", if is_checked { "enable" } else { "disable" }),
        "remove_url_tracking" => format!("filters {} 17", if is_checked { "enable" } else { "disable" }),
        "hide_search_queries" => format!("config set stealthmode.hide_search_queries {b}"),
        "dnt" => format!("config set stealthmode.send_do_not_track_signals {b}"),
        "third_party_cookies" => format!("config set stealthmode.block_third_party_cookies {b}"),
        "first_party_cookies" => format!("config set stealthmode.block_first_party_cookies {b}"),
        "etag" => format!("config set stealthmode.disable_third_party_cache {b}"),
        "auth_header" => format!("config set stealthmode.block_third_party_authorization {b}"),
        "webrtc" => format!("config set stealthmode.block_web_rtc {b}"),
        "push_api" => format!("config set stealthmode.block_browser_push_api {b}"),
        "location_api" => format!("config set stealthmode.block_browser_location_api {b}"),
        "referer" => format!("config set stealthmode.remove_referrer_from_third_party_requests {b}"),
        "user_agent" => format!("config set stealthmode.hide_user_agent {b}"),
        "mask_ip" => format!("config set stealthmode.hide_ip {b}"),
        "x_client_data" => format!("config set stealthmode.remove_x_client_data_header {b}"),
        "anti_dpi" => format!("config set stealthmode.anti_dpi.enabled {b}"),
        _ => return None,
    };
    Some(cmd)
}

fn on_setting_toggled(inner: &Rc<Inner>, key: &str, is_checked: bool) {
    // Sync with adguard-cli configuration.
    if let Some(cmd) = cli_map_entry(key, is_checked) {
        let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
        backend::run_cmd_async(&format!("adguard-cli {cmd}"), noop);
    }
    let _ = inner;
}

fn show_reset_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("menu-popover");
    popover.set_parent(&inner.btn_more);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    for key in ["menu_reset_standard", "menu_reset_high", "menu_reset_extreme"] {
        let preset = key.replace("menu_reset_", "");
        let item = gtk4::Button::new();
        item.set_label(&i18n::tr(key));
        item.add_css_class("menu-item");
        let p = popover.clone();
        let i = inner.clone();
        let pr = preset.clone();
        item.connect_clicked(move |_| {
            p.hide();
            reset_to_preset(&i, &pr);
        });
        menu_box.append(&item);
    }

    popover.set_child(Some(&menu_box));
    popover.show();
}

fn reset_to_preset(inner: &Rc<Inner>, preset_name: &str) {
    let preset = match STEALTH_PRESETS.iter().find(|(n, _)| *n == preset_name) {
        Some((_, p)) => p,
        None => return,
    };

    let mut cmds: Vec<String> = Vec::new();
    for (key, state) in preset {
        if let Some(row) = inner.rows.borrow().get(key as &str) {
            row.set_checked(*state, true);
        }
        if let Some(cmd) = cli_map_entry(key, *state) {
            cmds.push(format!("adguard-cli {cmd}"));
            if *key == "third_party_cookies" && *state {
                let min_val = if preset_name == "extreme" { 0 } else { 180 };
                cmds.push(format!(
                    "adguard-cli config set stealthmode.block_third_party_cookies_min {min_val}"
                ));
                inner
                    .badge_values
                    .borrow_mut()
                    .insert(key.to_string(), min_val.to_string());
                apply_row_badge_text(inner, key, &min_val.to_string());
            }
        }
    }

    if !cmds.is_empty() {
        let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
        backend::run_cmd_async(&cmds.join(" && "), noop);
    }
}
