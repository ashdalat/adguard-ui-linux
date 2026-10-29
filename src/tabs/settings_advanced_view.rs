// Port of adguard_modern/tabs/settings_advanced_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;
use regex::RegexBuilder;

fn proxy_yaml() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml")
        .to_string_lossy()
        .to_string()
}

struct Config {
    http3: bool,
    java: bool,
    flash: bool,
    redirect_mode: bool,
    split_delay: i64,
    inc_packet: bool,
    cert_transparency: bool,
    ocsp: bool,
    ech: bool,
    fallbacks: String,
    bootstraps: String,
    filter_secure_dns: bool,
    filter_secure_dns_mode: String,
    block_ech: bool,
    capture_har: bool,
    show_notifications: bool,
    show_vpn: bool,
}

impl Config {
    fn defaults() -> Config {
        Config {
            http3: true,
            java: true,
            flash: true,
            redirect_mode: false,
            split_delay: 20,
            inc_packet: false,
            cert_transparency: false,
            ocsp: false,
            ech: false,
            fallbacks: "default".to_string(),
            bootstraps: "default".to_string(),
            filter_secure_dns: true,
            filter_secure_dns_mode: "transparent".to_string(),
            block_ech: false,
            capture_har: false,
            show_notifications: false,
            show_vpn: true,
        }
    }
}

struct RowInner {
    root: gtk4::Box,
    key: String,
    title_text: RefCell<String>,
    desc_text: RefCell<String>,
    lbl_t: gtk4::Label,
    lbl_d: gtk4::Label,
    lbl_extra: Option<gtk4::Label>,
    click_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct AdvancedSettingRow {
    inner: Rc<RowInner>,
}

impl AdvancedSettingRow {
    pub fn new(
        key: &str,
        title: &str,
        description: &str,
        widget: Option<&ToggleSwitch>,
        extra_info: &str,
        is_clickable: bool,
    ) -> AdvancedSettingRow {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        if is_clickable {
            root.add_css_class("adv-row-clickable");
            root.set_margin_top(8);
            root.set_margin_bottom(8);
            root.set_margin_start(8);
            root.set_margin_end(8);
        } else {
            root.add_css_class("transparent-container");
            root.set_margin_top(4);
            root.set_margin_bottom(8);
        }

        let top_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        let t_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        t_box.set_hexpand(true);

        let lbl_t = gtk4::Label::new(Some(title));
        lbl_t.add_css_class("lbl-11-bold-white");
        lbl_t.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_t);

        let lbl_d = gtk4::Label::new(Some(description));
        lbl_d.add_css_class("lbl-10");
        lbl_d.set_wrap(true);
        lbl_d.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_d);

        let lbl_extra = if !extra_info.is_empty() {
            let l = gtk4::Label::new(Some(extra_info));
            l.add_css_class("adv-extra");
            l.set_halign(gtk4::Align::Start);
            t_box.append(&l);
            Some(l)
        } else {
            None
        };

        top_row.append(&t_box);
        if let Some(w) = widget {
            w.root().set_valign(gtk4::Align::Center);
            top_row.append(w.root());
        }
        root.append(&top_row);

        let inner = Rc::new(RowInner {
            root,
            key: key.to_string(),
            title_text: RefCell::new(title.to_string()),
            desc_text: RefCell::new(description.to_string()),
            lbl_t,
            lbl_d,
            lbl_extra,
            click_cb: RefCell::new(None),
        });

        if is_clickable {
            let i1 = inner.clone();
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_g, n, _x, _y| {
                // ignore clicks while the row is disabled
                if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
                if n == 1 {
                    if let Some(cb) = i1.click_cb.borrow().as_ref() {
                        cb();
                    }
                }
            });
            inner.root.add_controller(gesture);
        }

        AdvancedSettingRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.inner.title_text.borrow().to_lowercase().contains(&q)
            || self.inner.desc_text.borrow().to_lowercase().contains(&q)
    }

    pub fn update_text(&self, title: &str, description: &str, extra_info: &str) {
        *self.inner.title_text.borrow_mut() = title.to_string();
        *self.inner.desc_text.borrow_mut() = description.to_string();
        self.inner.lbl_t.set_text(title);
        self.inner.lbl_d.set_text(description);
        if !extra_info.is_empty() {
            if let Some(l) = self.inner.lbl_extra.as_ref() {
                l.set_text(extra_info);
            }
        }
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.click_cb.borrow_mut() = Some(cb);
    }
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    lbl_warn: gtk4::Label,
    input_search: gtk4::Entry,
    setting_rows: RefCell<Vec<AdvancedSettingRow>>,
    category_widgets: RefCell<Vec<(gtk4::Label, Vec<AdvancedSettingRow>)>>,
    category_headers: RefCell<Vec<(gtk4::Label, String)>>,
    cfg: RefCell<Config>,
    sw_redirect: ToggleSwitch,
    sw_http3: ToggleSwitch,
    sw_java: ToggleSwitch,
    sw_flash: ToggleSwitch,
    sw_inc_packet: ToggleSwitch,
    sw_ct: ToggleSwitch,
    sw_ocsp: ToggleSwitch,
    sw_ech: ToggleSwitch,
    sw_block_ech: ToggleSwitch,
    sw_notif: ToggleSwitch,
    sw_har: ToggleSwitch,
    sw_vpn: ToggleSwitch,
    row_frag: RefCell<Option<AdvancedSettingRow>>,
    row_fb: RefCell<Option<AdvancedSettingRow>>,
    row_bs: RefCell<Option<AdvancedSettingRow>>,
    row_sec_dns: RefCell<Option<AdvancedSettingRow>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_frag_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_fallback_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_bootstrap_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_secure_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsAdvancedView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn write_key(pattern: &str, replacement: &str) {
    let home = std::env::var("HOME").unwrap_or_default();
    let paths = [
        "/root/.local/share/adguard-cli/proxy.yaml".to_string(),
        std::path::Path::new(&home)
            .join(".local/share/adguard-cli/proxy.yaml")
            .to_string_lossy()
            .to_string(),
    ];
    for p in paths {
        if !std::path::Path::exists(&std::path::PathBuf::from(&p)) {
            continue;
        }
        if let Ok(mut c) = std::fs::read_to_string(&p) {
            if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                c = re.replace_all(&c, replacement).to_string();
                let _ = std::fs::write(&p, c);
            }
        }
    }
}

impl SettingsAdvancedView {
    pub fn new() -> SettingsAdvancedView {
        let cfg = load_config();

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);

        // Python: top_bar.addStretch() — keeps ⋮ pinned to the right edge
        let top_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_spacer.set_hexpand(true);
        top_bar.append(&top_spacer);

        let btn_more = gtk4::Button::new();
        btn_more.set_size_request(30, 30);
        btn_more.add_css_class("flat-icon-btn");
        btn_more.set_child(Some(&svg::svg_image("stat_more.svg", 16)));
        top_bar.append(&btn_more);
        root.append(&top_bar);
        root.append(&spacer(22));

        let lbl_title = gtk4::Label::new(Some("Advanced settings"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(6));

        let lbl_warn = gtk4::Label::new(Some(
            "Changing these settings may affect AdGuard’s stability, security settings, and performance",
        ));
        lbl_warn.add_css_class("adv-warn");
        lbl_warn.set_wrap(true);
        lbl_warn.set_halign(gtk4::Align::Start);
        root.append(&lbl_warn);
        root.append(&spacer(20));

        // Search bar.
        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_box.add_css_class("search-box");
        search_box.set_size_request(-1, 42);
        search_box.set_margin_start(12);
        search_box.set_margin_end(12);
        search_box.append(&svg::svg_image("search.svg", 16));

        let input_search = gtk4::Entry::new();
        input_search.add_css_class("search-input");
        input_search.set_hexpand(true);
        search_box.append(&input_search);
        root.append(&search_box);
        root.append(&spacer(16));

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 20);
        container.add_css_class("transparent-container");
        container.set_margin_end(8);
        container.set_margin_bottom(24);

        // Switches (created before rows so rows can hold them).
        let sw_redirect = ToggleSwitch::new(46, 26);
        sw_redirect.set_checked(cfg.redirect_mode, false);
        let sw_http3 = ToggleSwitch::new(46, 26);
        sw_http3.set_checked(cfg.http3, false);
        let sw_java = ToggleSwitch::new(46, 26);
        sw_java.set_checked(cfg.java, false);
        let sw_flash = ToggleSwitch::new(46, 26);
        sw_flash.set_checked(cfg.flash, false);
        let sw_inc_packet = ToggleSwitch::new(46, 26);
        sw_inc_packet.set_checked(cfg.inc_packet, false);
        let sw_ct = ToggleSwitch::new(46, 26);
        sw_ct.set_checked(cfg.cert_transparency, false);
        let sw_ocsp = ToggleSwitch::new(46, 26);
        sw_ocsp.set_checked(cfg.ocsp, false);
        let sw_ech = ToggleSwitch::new(46, 26);
        sw_ech.set_checked(cfg.ech, false);
        let sw_block_ech = ToggleSwitch::new(46, 26);
        sw_block_ech.set_checked(cfg.block_ech, false);
        let sw_notif = ToggleSwitch::new(46, 26);
        sw_notif.set_checked(cfg.show_notifications, false);
        let sw_har = ToggleSwitch::new(46, 26);
        sw_har.set_checked(cfg.capture_har, false);
        let sw_vpn = ToggleSwitch::new(46, 26);
        sw_vpn.set_checked(cfg.show_vpn, false);

        let bb_root = btn_back.root.clone();
        let bm = btn_more.clone();
        let is = input_search.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            lbl_warn,
            input_search,
            setting_rows: RefCell::new(Vec::new()),
            category_widgets: RefCell::new(Vec::new()),
            category_headers: RefCell::new(Vec::new()),
            cfg: RefCell::new(cfg),
            sw_redirect,
            sw_http3,
            sw_java,
            sw_flash,
            sw_inc_packet,
            sw_ct,
            sw_ocsp,
            sw_ech,
            sw_block_ech,
            sw_notif,
            sw_har,
            sw_vpn,
            row_frag: RefCell::new(None),
            row_fb: RefCell::new(None),
            row_bs: RefCell::new(None),
            row_sec_dns: RefCell::new(None),
            back_cb: RefCell::new(None),
            open_frag_cb: RefCell::new(None),
            open_fallback_cb: RefCell::new(None),
            open_bootstrap_cb: RefCell::new(None),
            open_secure_cb: RefCell::new(None),
        });

        // Group 1: Filtering.
        add_category_header(&inner, &container, "Filtering", "adv_cat_filtering");
        add_row(&inner, &container, "redirect_mode", "Use redirect mode",
            "Redirects all traffic to the local proxy server via iptables for filtering. When off, AdGuard filters on manual port",
            Some(&inner.sw_redirect), "", false);
        add_row(&inner, &container, "http3", "Filter HTTP/3",
            "Filters traffic sent over HTTP/3, the latest version of the HTTP protocol based on QUIC",
            Some(&inner.sw_http3), "", false);
        add_row(&inner, &container, "java", "Block Java",
            "Disables JAVA plugins on websites and web services for safety reasons. Does not disable JavaScript",
            Some(&inner.sw_java), "", false);
        add_row(&inner, &container, "flash", "Block Flash",
            "Enhances protection and speeds up website loading by blocking Flash Player support in browsers",
            Some(&inner.sw_flash), "", false);

        // Group 2: Tracking protection (Anti-DPI).
        add_category_header(&inner, &container, "Tracking protection", "feat_tracking");
        let extra_frag = format!("Delay: {} ms", inner.cfg.borrow().split_delay);
        let row_frag = add_row(&inner, &container, "frag_delay", "Fragmentation delay",
            "Adds a delay before the second fragment of a ClientHello or HTTP request to trick DPI",
            None, &extra_frag, true);
        *inner.row_frag.borrow_mut() = Some(row_frag.clone());
        let i1 = inner.clone();
        row_frag.connect_clicked(Box::new(move || {
            if let Some(cb) = i1.open_frag_cb.borrow().as_ref() {
                cb();
            }
        }));

        add_row(&inner, &container, "inc_packet", "Increase first packet size",
            "Increases the size of the first packet in a connection to several kilobytes to bypass DPI filters",
            Some(&inner.sw_inc_packet), "", false);

        // Group 3: HTTPS filtering.
        add_category_header(&inner, &container, "HTTPS filtering", "https_filtering_title");
        add_row(&inner, &container, "cert_transparency", "Check certificate transparency",
            "Validates the authenticity of all domain certificates against the Chrome Certificate Transparency Policy",
            Some(&inner.sw_ct), "", false);
        add_row(&inner, &container, "ocsp", "OCSP checking",
            "Automatically checks the revocation status of SSL certificates",
            Some(&inner.sw_ocsp), "", false);
        add_row(&inner, &container, "ech", "Encrypted Client Hello",
            "Uses the local DNS proxy to look up ECH configs. Encrypts ClientHellos if necessary",
            Some(&inner.sw_ech), "", false);

        // Group 4: DNS protection.
        add_category_header(&inner, &container, "DNS protection", "feat_dns");
        let extra_fb = format!("Servers: {}", inner.cfg.borrow().fallbacks);
        let row_fb = add_row(&inner, &container, "fallbacks", "Use fallback DNS servers",
            "DNS servers used for timed-out requests or DNS exclusions",
            None, &extra_fb, true);
        *inner.row_fb.borrow_mut() = Some(row_fb.clone());
        let i2 = inner.clone();
        row_fb.connect_clicked(Box::new(move || {
            if let Some(cb) = i2.open_fallback_cb.borrow().as_ref() {
                cb();
            }
        }));

        let extra_bs = format!("Servers: {}", inner.cfg.borrow().bootstraps);
        let row_bs = add_row(&inner, &container, "bootstraps", "Custom bootstrap addresses",
            "IP addresses of intermediate DNS servers used to resolve the domain name of the secure DNS server",
            None, &extra_bs, true);
        *inner.row_bs.borrow_mut() = Some(row_bs.clone());
        let i3 = inner.clone();
        row_bs.connect_clicked(Box::new(move || {
            if let Some(cb) = i3.open_bootstrap_cb.borrow().as_ref() {
                cb();
            }
        }));

        let mode = inner.cfg.borrow().filter_secure_dns_mode.clone();
        let sec_extra = if mode == "off" {
            "Filtering method: Off".to_string()
        } else if mode == "redirect" {
            "Filtering method: Redirect to a custom DNS server".to_string()
        } else {
            "Filtering method: Filter on the fly".to_string()
        };
        let row_sec = add_row(&inner, &container, "filter_secure_dns", "Filter secure DNS",
            "Filters encrypted DNS requests in addition to plain DNS requests",
            None, &sec_extra, true);
        *inner.row_sec_dns.borrow_mut() = Some(row_sec.clone());
        let i4 = inner.clone();
        row_sec.connect_clicked(Box::new(move || {
            if let Some(cb) = i4.open_secure_cb.borrow().as_ref() {
                cb();
            }
        }));

        add_row(&inner, &container, "block_ech", "Block Encrypted Client Hello",
            "Strips Encrypted Client Hello (ECH) parameters from responses",
            Some(&inner.sw_block_ech), "", false);

        // Group 5: Miscellaneous.
        add_category_header(&inner, &container, "Miscellaneous", "adv_cat_misc");
        add_row(&inner, &container, "show_notifications", "Receive AdGuard pop-up notifications",
            "Displays system notifications for protection status and updates",
            Some(&inner.sw_notif), "", false);
        add_row(&inner, &container, "capture_har", "Capture HAR",
            "Logs all filtered HTTP requests in HAR 1.2 format for debugging. Can be analyzed with Fiddler",
            Some(&inner.sw_har), "May slow down your device. Use for debugging purposes only", false);
        add_row(&inner, &container, "show_vpn", "Show AdGuard VPN in Protection",
            "Displays the AdGuard VPN integration card in the Protection tab",
            Some(&inner.sw_vpn), "", false);

        let i5 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i5.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i6 = inner.clone();
        let bm2 = bm.clone();
        bm.connect_clicked(move |_| show_more_menu(&i6, &bm2));

        let i7 = inner.clone();
        is.connect_changed(move |e| on_search_changed(&i7, &e.text()));

        inner.sw_redirect.connect_toggled(Box::new(move |chk| {
            let val = if chk { "auto" } else { "manual" };
            write_key(
                r#"(proxy_mode:\s*['"]?)(auto|manual)(['"]?)"#,
                &format!("${{1}}{}${{3}}", val),
            );
        }));

        for (sw, pattern) in [
            (&inner.sw_http3, r"(http3_filtering_enabled:\s*)(true|false)"),
            (&inner.sw_java, r"(block_browser_java:\s*)(true|false)"),
            (&inner.sw_flash, r"(block_browser_flash:\s*)(true|false)"),
            (&inner.sw_inc_packet, r"(increase_first_packet_size:\s*)(true|false)"),
            (&inner.sw_ct, r"(enforce_certificate_transparency:\s*)(true|false)"),
            (&inner.sw_ocsp, r"(ocsp_check_enabled:\s*)(true|false)"),
            (&inner.sw_ech, r"(encrypted_client_hello:\s*)(true|false)"),
            (&inner.sw_block_ech, r"(block_ech:\s*)(true|false)"),
        ] {
            let pat = pattern.to_string();
            sw.connect_toggled(Box::new(move |chk| {
                let val = if chk { "true" } else { "false" };
                write_key(&pat, &format!("${{1}}{}", val));
            }));
        }

        inner.sw_notif.connect_toggled(Box::new(move |chk| {
            let val = if chk { "true" } else { "false" };
            write_key(r"(show_notifications:\s*)(true|false)", &format!("${{1}}{}", val));
        }));
        inner.sw_har.connect_toggled(Box::new(move |chk| {
            let val = if chk { "true" } else { "false" };
            write_key(
                r"(har_writer:\s*\n\s*enabled:\s*)(true|false)",
                &format!("${{1}}{}", val),
            );
        }));

        let i11 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i11)));

        retranslate_ui(&inner);

        scroll.set_child(Some(&container));
        inner.root.append(&scroll);

        SettingsAdvancedView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_frag_delay_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_frag_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_fallback_dns_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_fallback_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_bootstrap_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_bootstrap_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_secure_dns_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_secure_cb.borrow_mut() = Some(cb);
    }

    pub fn update_frag_delay(&self, val: i64) {
        let i = &self.inner;
        i.cfg.borrow_mut().split_delay = val;
        if let Some(r) = i.row_frag.borrow().as_ref() {
            if let Some(l) = r.inner.lbl_extra.as_ref() {
                l.set_text(&i18n::tr("adv_delay_info").replace("{val}", &val.to_string()));
            }
        }
    }

    pub fn update_fallbacks(&self, val: &str) {
        let i = &self.inner;
        i.cfg.borrow_mut().fallbacks = val.to_string();
        if let Some(r) = i.row_fb.borrow().as_ref() {
            if let Some(l) = r.inner.lbl_extra.as_ref() {
                l.set_text(&i18n::tr("adv_servers_info").replace("{val}", val));
            }
        }
    }

    pub fn update_bootstraps(&self, val: &str) {
        let i = &self.inner;
        i.cfg.borrow_mut().bootstraps = val.to_string();
        if let Some(r) = i.row_bs.borrow().as_ref() {
            if let Some(l) = r.inner.lbl_extra.as_ref() {
                l.set_text(&i18n::tr("adv_servers_info").replace("{val}", val));
            }
        }
    }

    pub fn update_secure_dns(&self, mode: &str) {
        update_secure_dns(&self.inner, mode);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn update_secure_dns(inner: &Rc<Inner>, mode: &str) {
    inner.cfg.borrow_mut().filter_secure_dns_mode = mode.to_string();
    let text = if mode == "off" {
        i18n::tr("adv_method_off")
    } else if mode == "redirect" {
        i18n::tr("adv_method_redirect")
    } else {
        i18n::tr("adv_method_fly")
    };
    if let Some(r) = inner.row_sec_dns.borrow().as_ref() {
        if let Some(l) = r.inner.lbl_extra.as_ref() {
            l.set_text(&text);
        }
    }
}

fn load_config() -> Config {
    let mut cfg = Config::defaults();
    let home = std::env::var("HOME").unwrap_or_default();
    let home_yaml = std::path::Path::new(&home)
        .join(".local/share/adguard-cli/proxy.yaml");
    let root_yaml = std::path::PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml");
    let cfg_file = if std::path::Path::exists(&root_yaml) {
        Some(root_yaml)
    } else if std::path::Path::exists(&home_yaml) {
        Some(home_yaml)
    } else {
        None
    };
    let Some(f) = cfg_file else { return cfg };
    let c = match std::fs::read_to_string(&f) {
        Ok(c) => c,
        Err(_) => return cfg,
    };

    fn flag(c: &str, pattern: &str, cur: bool) -> bool {
        match RegexBuilder::new(pattern).case_insensitive(true).build() {
            Ok(re) => match re.captures(c) {
                Some(m) => m[1].to_lowercase() == "true",
                None => cur,
            },
            Err(_) => cur,
        }
    }

    cfg.http3 = flag(&c, r"http3_filtering_enabled:\s*(true|false)", cfg.http3);
    cfg.java = flag(&c, r"block_browser_java:\s*(true|false)", cfg.java);
    cfg.flash = flag(&c, r"block_browser_flash:\s*(true|false)", cfg.flash);
    match RegexBuilder::new(r#"(proxy_mode:\s*['"]?)(auto|manual)(['"]?)"#)
        .case_insensitive(true)
        .build()
    {
        Ok(re) => {
            if let Some(m) = re.captures(&c) {
                cfg.redirect_mode = m[2].to_lowercase() == "auto";
            }
        }
        Err(_) => {}
    }
    if let Ok(re) = RegexBuilder::new(r"split_delay_ms:\s*([0-9]+)").build() {
        if let Some(m) = re.captures(&c) {
            cfg.split_delay = m[1].parse().unwrap_or(20);
        }
    }
    cfg.inc_packet = flag(&c, r"increase_first_packet_size:\s*(true|false)", cfg.inc_packet);
    cfg.cert_transparency = flag(
        &c,
        r"enforce_certificate_transparency:\s*(true|false)",
        cfg.cert_transparency,
    );
    cfg.ocsp = flag(&c, r"ocsp_check_enabled:\s*(true|false)", cfg.ocsp);
    cfg.ech = flag(&c, r"encrypted_client_hello:\s*(true|false)", cfg.ech);
    if let Ok(re) = RegexBuilder::new(r"fallbacks:\s*([^\n]+)").build() {
        if let Some(m) = re.captures(&c) {
            cfg.fallbacks = m[1].trim().trim_matches('\'').trim_matches('"').to_string();
        }
    }
    if let Ok(re) = RegexBuilder::new(r"bootstraps:\s*([^\n]+)").build() {
        if let Some(m) = re.captures(&c) {
            cfg.bootstraps = m[1].trim().trim_matches('\'').trim_matches('"').to_string();
        }
    }
    if let Ok(re) = RegexBuilder::new(r#"filter_secure_dns_mode:\s*['"]?(transparent|redirect|off)['"]?"#)
        .case_insensitive(true)
        .build()
    {
        if let Some(m) = re.captures(&c) {
            let mode = m[1].to_lowercase();
            cfg.filter_secure_dns = mode != "off";
            cfg.filter_secure_dns_mode = mode;
        } else {
            cfg.filter_secure_dns_mode = "transparent".to_string();
        }
    }
    cfg.block_ech = flag(&c, r"block_ech:\s*(true|false)", cfg.block_ech);
    cfg.show_notifications = flag(&c, r"show_notifications:\s*(true|false)", cfg.show_notifications);
    cfg.capture_har = flag(&c, r"har_writer:\s*\n\s*enabled:\s*(true|false)", cfg.capture_har);

    cfg
}

fn add_category_header(inner: &Rc<Inner>, container: &gtk4::Box, title: &str, i18n_key: &str) {
    let lbl = gtk4::Label::new(Some(title));
    lbl.add_css_class("adv-cat-header");
    lbl.set_halign(gtk4::Align::Start);
    container.append(&lbl);
    inner.category_widgets.borrow_mut().push((lbl.clone(), Vec::new()));
    if !i18n_key.is_empty() {
        inner
            .category_headers
            .borrow_mut()
            .push((lbl, i18n_key.to_string()));
    }
}

fn add_row(
    inner: &Rc<Inner>,
    container: &gtk4::Box,
    key: &str,
    title: &str,
    desc: &str,
    widget: Option<&ToggleSwitch>,
    extra_info: &str,
    is_clickable: bool,
) -> AdvancedSettingRow {
    let row = AdvancedSettingRow::new(key, title, desc, widget, extra_info, is_clickable);
    container.append(row.root());
    inner.setting_rows.borrow_mut().push(row.clone());
    if let Some(last) = inner.category_widgets.borrow_mut().last_mut() {
        last.1.push(row.clone());
    }
    row
}

fn on_search_changed(inner: &Rc<Inner>, text: &str) {
    let q = text.trim().to_lowercase();
    let cats = inner.category_widgets.borrow().clone();
    for (cat_lbl, rows) in cats {
        let mut has_match = false;
        for r in rows {
            let matched = if q.is_empty() { true } else { r.matches_search(&q) };
            r.root().set_visible(matched);
            if matched {
                has_match = true;
            }
        }
        cat_lbl.set_visible(has_match);
    }
}

fn show_more_menu(inner: &Rc<Inner>, anchor: &gtk4::Button) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("menu-popover");
    popover.set_parent(anchor);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let item = gtk4::Button::new();
    item.set_label(&i18n::tr("menu_reset_default"));
    item.add_css_class("menu-item-red");
    let i1 = inner.clone();
    let p1 = popover.clone();
    item.connect_clicked(move |_| {
        p1.hide();
        reset_to_default(&i1);
    });
    menu_box.append(&item);

    popover.set_child(Some(&menu_box));
    popover.show();
}

fn reset_to_default(inner: &Rc<Inner>) {
    // QMessageBox with Yes/No (Yes styled red, default No) -> GTK window.
    let dlg = crate::util::sub_window("");
    dlg.set_title(Some(&i18n::tr("adv_reset_confirm_t")));
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    box_.add_css_class("dlg-dark");
    box_.set_margin_top(24);
    box_.set_margin_bottom(20);
    box_.set_margin_start(24);
    box_.set_margin_end(24);

    let lbl = gtk4::Label::new(Some(&i18n::tr("adv_reset_confirm_msg")));
    lbl.add_css_class("dlg-title");
    lbl.set_wrap(true);
    lbl.set_xalign(0.0);
    box_.append(&lbl);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::End);

    let btn_no = gtk4::Button::new();
    btn_no.set_label(&i18n::tr_default("btn_cancel", "Cancel"));
    btn_no.add_css_class("outline-btn");
    let d1 = dlg.clone();
    btn_no.connect_clicked(move |_| d1.close());
    row.append(&btn_no);

    let btn_yes = gtk4::Button::new();
    btn_yes.set_label(&i18n::tr_default("btn_reset", "Reset"));
    btn_yes.add_css_class("red-btn");
    let i1 = inner.clone();
    let d2 = dlg.clone();
    btn_yes.connect_clicked(move |_| {
        d2.close();
        apply_factory_defaults(&i1);
    });
    row.append(&btn_yes);

    box_.append(&row);
    dlg.set_child(Some(&box_));
    dlg.present();
}

fn apply_factory_defaults(inner: &Rc<Inner>) {
    let path = proxy_yaml();
    if std::path::Path::exists(&std::path::PathBuf::from(&path)) {
        if let Ok(mut c) = std::fs::read_to_string(&path) {
            let subs: [(&str, &str); 14] = [
                (r#"(proxy_mode:\s*['"]?)(auto|manual)(['"]?)"#, "${1}manual${3}"),
                (r"(http3_filtering_enabled:\s*)(true|false)", "${1}true"),
                (r"(block_browser_java:\s*)(true|false)", "${1}true"),
                (r"(block_browser_flash:\s*)(true|false)", "${1}true"),
                (r"(split_delay_ms:\s*)([0-9]+)", "${1}20"),
                (r"(increase_first_packet_size:\s*)(true|false)", "${1}false"),
                (r"(enforce_certificate_transparency:\s*)(true|false)", "${1}false"),
                (r"(ocsp_check_enabled:\s*)(true|false)", "${1}false"),
                (r"(encrypted_client_hello:\s*)(true|false)", "${1}false"),
                (r"(fallbacks:\s*)([^\n]+)", "${1}default"),
                (r"(bootstraps:\s*)([^\n]+)", "${1}default"),
                (
                    r#"(filter_secure_dns_mode:\s*['"]?)(transparent|redirect|off)(['"]?)"#,
                    "${1}transparent${3}",
                ),
                (r"(block_ech:\s*)(true|false)", "${1}false"),
                (r"(show_notifications:\s*)(true|false)", "${1}false"),
            ];
            for (pat, rep) in subs {
                if let Ok(re) = RegexBuilder::new(pat).case_insensitive(true).build() {
                    c = re.replace_all(&c, rep).to_string();
                }
            }
            // har_writer is applied last, matching Python order.
            if let Ok(re) = RegexBuilder::new(r"(har_writer:\s*\n\s*enabled:\s*)(true|false)")
                .case_insensitive(true)
                .build()
            {
                c = re.replace_all(&c, "${1}false").to_string();
            }
            let _ = std::fs::write(&path, c);
        }
    }

    {
        let mut cfg = inner.cfg.borrow_mut();
        cfg.redirect_mode = false;
        cfg.http3 = true;
        cfg.java = true;
        cfg.flash = true;
        cfg.split_delay = 20;
        cfg.inc_packet = false;
        cfg.cert_transparency = false;
        cfg.ocsp = false;
        cfg.ech = false;
        cfg.fallbacks = "default".to_string();
        cfg.bootstraps = "default".to_string();
        cfg.filter_secure_dns = true;
        cfg.block_ech = false;
        cfg.show_notifications = false;
        cfg.capture_har = false;
        cfg.show_vpn = true;
    }

    inner.sw_redirect.set_checked(false, true);
    inner.sw_http3.set_checked(true, true);
    inner.sw_java.set_checked(true, true);
    inner.sw_flash.set_checked(true, true);
    if let Some(r) = inner.row_frag.borrow().as_ref() {
        if let Some(l) = r.inner.lbl_extra.as_ref() {
            l.set_text("Delay: 20 ms");
        }
    }
    inner.sw_inc_packet.set_checked(false, true);
    inner.sw_ct.set_checked(false, true);
    inner.sw_ocsp.set_checked(false, true);
    inner.sw_ech.set_checked(false, true);
    if let Some(r) = inner.row_fb.borrow().as_ref() {
        if let Some(l) = r.inner.lbl_extra.as_ref() {
            l.set_text("Servers: default");
        }
    }
    if let Some(r) = inner.row_bs.borrow().as_ref() {
        if let Some(l) = r.inner.lbl_extra.as_ref() {
            l.set_text("Servers: default");
        }
    }
    update_secure_dns(inner, "transparent");
    inner.sw_block_ech.set_checked(false, true);
    inner.sw_notif.set_checked(false, true);
    inner.sw_har.set_checked(false, true);
    inner.sw_vpn.set_checked(true, true);
}

#[allow(dead_code)]
fn open_yaml() {
    let path = proxy_yaml();
    match std::process::Command::new("xdg-open").arg(&path).spawn() {
        Ok(_) => {}
        Err(e) => show_message(None, "Error", &e.to_string()),
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("adv_title"));
    inner.lbl_warn.set_text(&i18n::tr("adv_warn"));
    inner.input_search.set_placeholder_text(Some(&i18n::tr_default("flog_search_placeholder", "Search")));

    let headers = inner.category_headers.borrow().clone();
    for (lbl, key) in headers {
        lbl.set_text(&i18n::tr(&key));
    }

    let map: HashMap<&str, (&str, &str)> = [
        ("redirect_mode", ("adv_redirect_mode_t", "adv_redirect_mode_d")),
        ("http3", ("adv_http3_t", "adv_http3_d")),
        ("java", ("adv_java_t", "adv_java_d")),
        ("flash", ("adv_flash_t", "adv_flash_d")),
        ("frag_delay", ("adv_frag_delay_t", "adv_frag_delay_d")),
        ("inc_packet", ("adv_inc_packet_t", "adv_inc_packet_d")),
        ("cert_transparency", ("adv_cert_transparency_t", "adv_cert_transparency_d")),
        ("ocsp", ("adv_ocsp_t", "adv_ocsp_d")),
        ("ech", ("adv_ech_t", "adv_ech_d")),
        ("fallbacks", ("adv_fallbacks_t", "adv_fallbacks_d")),
        ("bootstraps", ("adv_bootstraps_t", "adv_bootstraps_d")),
        ("filter_secure_dns", ("adv_secure_dns_t", "adv_secure_dns_d")),
        ("block_ech", ("adv_block_ech_t", "adv_block_ech_d")),
        ("show_notifications", ("adv_notif_t", "adv_notif_d")),
        ("capture_har", ("adv_logging_t", "adv_logging_d")),
        ("show_vpn", ("adv_vpn_t", "adv_vpn_d")),
    ]
    .into_iter()
    .collect();

    let rows = inner.setting_rows.borrow().clone();
    for row in rows {
        let key = row.inner.key.clone();
        if let Some((t_k, d_k)) = map.get(key.as_str()) {
            let extra = {
                let cfg = inner.cfg.borrow();
                if key == "frag_delay" {
                    i18n::tr("adv_delay_info").replace("{val}", &cfg.split_delay.to_string())
                } else if key == "fallbacks" || key == "bootstraps" {
                    let val = if key == "fallbacks" { &cfg.fallbacks } else { &cfg.bootstraps };
                    i18n::tr("adv_servers_info").replace("{val}", val)
                } else if key == "filter_secure_dns" {
                    let mode = &cfg.filter_secure_dns_mode;
                    if mode == "off" {
                        i18n::tr("adv_method_off")
                    } else if mode == "redirect" {
                        i18n::tr("adv_method_redirect")
                    } else {
                        i18n::tr("adv_method_fly")
                    }
                } else if key == "capture_har" {
                    i18n::tr("adv_har_extra")
                } else {
                    String::new()
                }
            };
            row.update_text(&i18n::tr(t_k), &i18n::tr(d_k), &extra);
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::RegexBuilder;

    /// Every replacement template used by write_key()/apply_factory_defaults()
    /// must keep its literal text: in the regex crate an unbraced `$1true` is read
    /// as a group named `1true` and silently replaced with an empty string, which
    /// wiped whole values in proxy.yaml. Braced `${1}` is the correct form.
    #[test]
    fn yaml_replacement_templates_keep_their_values() {
        let cases: [(&str, &str, &str, &str); 6] = [
            (
                r#"(proxy_mode:\s*['"]?)(auto|manual)(['"]?)"#,
                "proxy_mode: auto\n",
                "${1}manual${3}",
                "proxy_mode: manual",
            ),
            (
                r"(http3_filtering_enabled:\s*)(true|false)",
                "http3_filtering_enabled: false\n",
                "${1}true",
                "http3_filtering_enabled: true",
            ),
            (
                r"(split_delay_ms:\s*)([0-9]+)",
                "split_delay_ms: 5\n",
                "${1}20",
                "split_delay_ms: 20",
            ),
            (
                r"(fallbacks:\s*)([^\n]+)",
                "fallbacks: 1.1.1.1\n",
                "${1}default",
                "fallbacks: default",
            ),
            (
                r"(show_notifications:\s*)(true|false)",
                "show_notifications: true\n",
                "${1}false",
                "show_notifications: false",
            ),
            (
                r#"(filter_secure_dns_mode:\s*['"]?)(transparent|redirect|off)(['"]?)"#,
                "filter_secure_dns_mode: off\n",
                "${1}transparent${3}",
                "filter_secure_dns_mode: transparent",
            ),
        ];
        for (pat, input, rep, expected) in cases {
            let re = RegexBuilder::new(pat).case_insensitive(true).build().unwrap();
            let out = re.replace_all(input, rep).to_string();
            assert_eq!(out.trim_end(), expected, "pattern {pat:?} -> {out:?}");
        }
    }

    /// The har_writer block spans two lines; the nested `enabled:` value must be
    /// replaced while both lines are preserved.
    #[test]
    fn har_writer_replacement_preserves_both_lines() {
        let input = "har_writer:\n  enabled: true\nother: 1\n";
        let re = RegexBuilder::new(r"(har_writer:\s*\n\s*enabled:\s*)(true|false)")
            .case_insensitive(true)
            .build()
            .unwrap();
        let out = re.replace_all(input, "${1}false").to_string();
        assert_eq!(out, "har_writer:\n  enabled: false\nother: 1\n");
    }
}
