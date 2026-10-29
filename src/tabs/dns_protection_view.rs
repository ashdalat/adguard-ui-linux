// Port of adguard_modern/tabs/dns_protection_view.py
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use regex::Regex;

use crate::backend;
use crate::i18n;
use crate::widgets::assistant_modal::AdGuardAssistantModal;
use crate::widgets::filter_editor_window::FilterEditorWindow;
use crate::widgets::filtering_log_window::FilteringLogDialog;
use crate::widgets::svg;
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
// DnsServerCard
// ---------------------------------------------------------------------------
struct ServerCardInner {
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    lbl_current: gtk4::Label,
    server_name: RefCell<String>,
    clicked_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct DnsServerCard {
    inner: Rc<ServerCardInner>,
}

impl DnsServerCard {
    pub fn new() -> DnsServerCard {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("dns-server-card");
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(16);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some("DNS server"));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(
            "Allows to select a DNS server to resolve DNS requests, block ads and trackers, and encrypt DNS traffic",
        ));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        text_box.append(&lbl_desc);

        let lbl_current = gtk4::Label::new(Some("Current: System DNS"));
        lbl_current.add_css_class("dns-current");
        lbl_current.set_halign(gtk4::Align::Start);
        lbl_current.set_margin_top(4);
        text_box.append(&lbl_current);

        root.append(&text_box);

        let chevron = gtk4::Label::new(Some("›"));
        chevron.add_css_class("dns-chevron");
        chevron.set_valign(gtk4::Align::Center);
        root.append(&chevron);

        let inner = Rc::new(ServerCardInner {
            root,
            lbl_title,
            lbl_desc,
            lbl_current,
            server_name: RefCell::new("System DNS".to_string()),
            clicked_cb: RefCell::new(None),
        });

        let i = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                    cb();
                }
            }
        });
        inner.root.add_controller(gesture);

        DnsServerCard { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_current_server(&self, name: &str) {
        *self.inner.server_name.borrow_mut() = name.to_string();
        self.update_display();
    }

    fn update_display(&self) {
        let name = self.inner.server_name.borrow().clone();
        let t = i18n::tr("dns_card_server_cur").replace("{name}", &name);
        self.inner.lbl_current.set_text(&t);
    }

    pub fn update_text(&self, title: &str, desc: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
        self.update_display();
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// DnsToggleRow
// ---------------------------------------------------------------------------
struct ToggleRowInner {
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
    clicked_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct DnsToggleRow {
    inner: Rc<ToggleRowInner>,
}

impl DnsToggleRow {
    pub fn new(title: &str, description: &str, initial_state: bool) -> DnsToggleRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("dns-toggle-row");
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(16);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        text_box.append(&lbl_desc);

        root.append(&text_box);

        let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        divider.add_css_class("dns-divider");
        divider.set_size_request(1, 30);
        divider.set_valign(gtk4::Align::Center);
        root.append(&divider);

        let sp = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        sp.set_size_request(2, -1);
        root.append(&sp);

        let switch = ToggleSwitch::new(46, 26);
        switch.set_checked(initial_state, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(ToggleRowInner {
            root,
            lbl_title,
            lbl_desc,
            switch,
            toggled_cb: RefCell::new(None),
            clicked_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(val);
            }
        }));

        let i = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                    cb();
                }
            }
        });
        inner.root.add_controller(gesture);

        DnsToggleRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_text(&self, title: &str, desc: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
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

    pub fn connect_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// DnsProtectionView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    btn_what_is_dns: gtk4::Button,
    card_dns_server: DnsServerCard,
    row_dns_filters: DnsToggleRow,
    row_dns_user_rules: DnsToggleRow,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
    server_drilldown_cb: RefCell<Option<Box<dyn Fn()>>>,
    filters_drilldown_cb: RefCell<Option<Box<dyn Fn()>>>,
    user_rules_drilldown_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct DnsProtectionView {
    inner: Rc<Inner>,
}

impl DnsProtectionView {
    pub fn new() -> DnsProtectionView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("dns-protection-view");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: '‹ Back' left, '⋮' right.
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
        btn_more.set_size_request(28, 28);
        btn_more.add_css_class("detail-more-btn");
        btn_more.set_child(Some(&svg::svg_image("stat_more.svg", 18)));
        top_bar.append(&btn_more);
        root.append(&top_bar);

        // Title row: DNS protection + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("DNS protection"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some("Protects you from ads, trackers, and threats at the DNS level"));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        // Green link: What is DNS?
        let btn_what_is_dns = gtk4::Button::new();
        btn_what_is_dns.set_label("What is DNS?  ↗");
        btn_what_is_dns.add_css_class("green-link");
        btn_what_is_dns.set_halign(gtk4::Align::Start);
        root.append(&btn_what_is_dns);

        root.append(&spacer(16));

        // Card 1: DNS server.
        let card_dns_server = DnsServerCard::new();
        root.append(card_dns_server.root());

        root.append(&spacer(6));

        // Card 2: DNS filters.
        let row_dns_filters = DnsToggleRow::new(
            "DNS filters",
            "Allows to add DNS filters and user rules to block ads and trackers at the domain level",
            true,
        );
        root.append(row_dns_filters.root());

        root.append(&spacer(6));

        // Card 3: DNS user rules.
        let row_dns_user_rules = DnsToggleRow::new(
            "DNS user rules",
            "Block, unblock, or redirect domains with your user rules",
            true,
        );
        root.append(row_dns_user_rules.root());

        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            master_switch,
            lbl_desc,
            btn_what_is_dns,
            card_dns_server,
            row_dns_filters,
            row_dns_user_rules,
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
            server_drilldown_cb: RefCell::new(None),
            filters_drilldown_cb: RefCell::new(None),
            user_rules_drilldown_cb: RefCell::new(None),
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
        bm.connect_clicked(move |_| show_options_menu(&i2));

        let i3 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggled(&i3, val)));

        let i4 = inner.clone();
        inner.btn_what_is_dns.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard-dns.io/kb/general/dns-filtering/")
                .spawn();
            let _ = &i4;
        });

        let card = inner.card_dns_server.clone();
        let i5 = inner.clone();
        card.connect_clicked(Box::new(move || {
            if let Some(cb) = i5.server_drilldown_cb.borrow().as_ref() {
                cb();
            }
        }));

        let row_f = inner.row_dns_filters.clone();
        let i6 = inner.clone();
        row_f.connect_toggled(Box::new(move |is_active| {
            let action = if is_active { "enable" } else { "disable" };
            let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
            backend::run_cmd_async(&format!("adguard-cli dns filters {action} 1"), noop);
        }));

        let row_f2 = inner.row_dns_filters.clone();
        let i7 = inner.clone();
        row_f2.connect_clicked(Box::new(move || {
            if let Some(cb) = i7.filters_drilldown_cb.borrow().as_ref() {
                cb();
            }
        }));

        let row_u = inner.row_dns_user_rules.clone();
        let i8 = inner.clone();
        row_u.connect_toggled(Box::new(move |_is_active| {
            // Python handler is intentionally empty (pass).
            let _ = &i8;
        }));

        let row_u2 = inner.row_dns_user_rules.clone();
        let i9 = inner.clone();
        row_u2.connect_clicked(Box::new(move || {
            if let Some(cb) = i9.user_rules_drilldown_cb.borrow().as_ref() {
                cb();
            }
        }));

        let i10 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i10)));

        retranslate_ui(&inner);
        refresh_status(&inner);

        DnsProtectionView { inner }
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

    pub fn connect_dns_server_drilldown(&self, cb: Box<dyn Fn()>) {
        *self.inner.server_drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_dns_filters_drilldown(&self, cb: Box<dyn Fn()>) {
        *self.inner.filters_drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_dns_user_rules_drilldown(&self, cb: Box<dyn Fn()>) {
        *self.inner.user_rules_drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn card_dns_server(&self) -> &DnsServerCard {
        &self.inner.card_dns_server
    }

    pub fn refresh_status(&self) {
        refresh_status(&self.inner);
    }

    pub fn set_active(&self, is_active: bool) {
        self.inner.master_switch.set_checked(is_active, false);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn row_dns_filters(&self) -> &DnsToggleRow {
        &self.inner.row_dns_filters
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("dns_title"));
    inner.lbl_desc.set_text(&i18n::tr("dns_desc"));
    inner.btn_what_is_dns.set_label(&i18n::tr("dns_what_is"));
    inner
        .card_dns_server
        .update_text(&i18n::tr("dns_card_server_t"), &i18n::tr("dns_card_server_d"));
    inner
        .row_dns_filters
        .update_text(&i18n::tr("dns_card_filters_t"), &i18n::tr("dns_card_filters_d"));
    inner.row_dns_user_rules.update_text(
        &i18n::tr("dns_card_user_rules_t"),
        &i18n::tr("dns_card_user_rules_d"),
    );
}

// Read proxy.yaml to update current upstream display and master switch.
fn refresh_status(inner: &Rc<Inner>) {
    let path = proxy_yaml();
    let mut upstream = "default".to_string();
    let mut enabled = true;
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            let re_up = Regex::new(r"^\s+upstream:\s*(.*)").unwrap();
            let re_en = Regex::new(r"^\s+enabled:\s*(.*)").unwrap();
            let mut in_dns = false;
            for line in content.lines() {
                let clean = line.split('#').next().unwrap_or("").trim_end();
                if clean.starts_with("dns_filtering:") {
                    in_dns = true;
                    continue;
                }
                if in_dns {
                    if !clean.is_empty() && !clean.starts_with(' ') && !clean.starts_with('\t') {
                        in_dns = false;
                        continue;
                    }
                    if let Some(caps) = re_up.captures(clean) {
                        let g = caps[1].trim().trim_matches('\'').trim_matches('"').to_string();
                        upstream = g;
                    }
                    if let Some(caps) = re_en.captures(clean) {
                        enabled = caps[1].trim().to_lowercase() == "true";
                    }
                }
            }
        }
    }

    inner.master_switch.set_checked(enabled, false);

    // Translate upstream to human friendly name.
    let name_map: [(&str, &str); 7] = [
        ("default", "System DNS"),
        ("https://dns.adguard-dns.com/dns-query", "AdGuard DNS"),
        ("https://unfiltered.adguard-dns.com/dns-query", "AdGuard DNS Non-filtering"),
        ("https://family.adguard-dns.com/dns-query", "AdGuard DNS Family Protection"),
        ("https://cloudflare-dns.com/dns-query", "Cloudflare DNS"),
        ("https://dns.quad9.net/dns-query", "Quad9 DNS"),
        ("https://dns.google/dns-query", "Google Public DNS"),
    ];
    let display_name = match name_map.iter().find(|(k, _)| *k == upstream.as_str()) {
        Some((_, name)) => name.to_string(),
        None => {
            if upstream != "default" {
                format!("Custom ({upstream})")
            } else {
                "System DNS".to_string()
            }
        }
    };
    inner.card_dns_server.set_current_server(&display_name);
}

fn on_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(
        &format!("adguard-cli config set dns_filtering.enabled {}", if is_active { "true" } else { "false" }),
        noop,
    );

    // Update proxy.yaml.
    for p in [proxy_yaml(), root_proxy_yaml()] {
        if !p.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&p) {
            let re_en = Regex::new(r"^\s*enabled:\s*").unwrap();
            let mut out: Vec<String> = Vec::new();
            let mut in_dns = false;
            for line in content.lines() {
                if line.starts_with("dns_filtering:") {
                    in_dns = true;
                    out.push(line.to_string());
                    continue;
                }
                if in_dns {
                    if !line.trim().is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                        in_dns = false;
                    } else if re_en.is_match(line) {
                        out.push(format!("  enabled: {}", if is_active { "true" } else { "false" }));
                        continue;
                    }
                }
                out.push(line.to_string());
            }
            let _ = std::fs::write(&p, out.join("\n") + "\n");
        }
    }

    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(is_active);
    }
}

fn show_options_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("detail-menu-popover");
    popover.set_parent(&inner.btn_more);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_log = gtk4::Button::new();
    item_log.set_label(&i18n::tr("flog_win_title"));
    item_log.add_css_class("detail-menu-item");
    let p1 = popover.clone();
    item_log.connect_clicked(move |_| {
        p1.hide();
        // Deviation: QDialog.exec() -> non-modal window (no parent).
        FilteringLogDialog::new().root().present();
    });
    menu_box.append(&item_log);

    let item_editor = gtk4::Button::new();
    item_editor.set_label(&i18n::tr("fe_win_title"));
    item_editor.add_css_class("detail-menu-item");
    let p2 = popover.clone();
    item_editor.connect_clicked(move |_| {
        p2.hide();
        FilterEditorWindow::new().root().present();
    });
    menu_box.append(&item_editor);

    let item_assistant = gtk4::Button::new();
    item_assistant.set_label(&i18n::tr("menu_adguard_assistant"));
    item_assistant.add_css_class("detail-menu-item");
    let p3 = popover.clone();
    item_assistant.connect_clicked(move |_| {
        p3.hide();
        AdGuardAssistantModal::new().root.present();
    });
    menu_box.append(&item_assistant);

    popover.set_child(Some(&menu_box));
    popover.show();
}
