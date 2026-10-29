// Port of adguard_modern/tabs/home_network_section.py — the network cards on the
// Home tab (HTTP proxy switch, HTTPS filtering card, filtering-mode nav row,
// browser traffic routing card).
use gtk4::prelude::*;
use regex::RegexBuilder;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::backend;
use crate::constants as c;
use crate::i18n;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

const DEFAULT_HTTP_DESC: &str = "All traffic passing through this proxy will be filtered. You can also use it to filter traffic on other devices in your local network: install the <a href='cert' style='color: #5EBB6A; text-decoration: none;'>AdGuard root certificate</a> on them and configure the proxy";

/// Pango markup for the rich-text description label. Pango's <a> tag only
/// understands href, so the Qt inline style is dropped (the colour comes from the
/// surrounding span).
/// Convert the Python/Qt rich-text flavour of these strings into Pango markup.
///
/// Qt accepts `style='color: ...; text-decoration: none;'` on <a>; Pango ignores
/// that attribute and renders the tag as literal text, which is what made every
/// label with a link show raw HTML. Pango also cannot colour a link per-element,
/// so the colour comes from the surrounding <span> (Python's own colour choice).
fn link_markup(raw: &str) -> String {
    to_pango_link_markup(raw, c::COLOR_TEXT_MUTED)
}

/// Shared converter: strips Qt-only attributes and wraps the result in a span of
/// the given colour. `text-decoration` is dropped (Pango underlines <a> always).
pub fn to_pango_link_markup(raw: &str, color: &str) -> String {
    let s = strip_qt_style_attrs(raw);
    format!("<span color=\"{}\">{}</span>", color, s)
}

/// Remove Qt-only `style='...'` / `style="..."` attributes (both the attribute
/// name and its whole quoted value) from a rich-text string so Pango accepts it.
fn strip_qt_style_attrs(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if raw[i..].starts_with("style=") {
            let rest = &raw[i + 6..];
            let quote = rest.chars().next();
            match quote {
                Some(q @ ('\'' | '"')) => {
                    // Skip the opening quote and everything up to the closing one.
                    match rest[1..].find(q) {
                        Some(end) => {
                            i += 6 + 1 + end + 1;
                            continue;
                        }
                        None => {
                            i = bytes.len();
                            continue;
                        }
                    }
                }
                _ => {}
            }
        }
        // Copy one UTF-8 character.
        let ch = raw[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    // Collapse whitespace left in front of the closing bracket: `<a href='x' >`.
    out.replace("  >", ">").replace(" >", ">")
}

fn yaml_paths() -> Vec<PathBuf> {
    vec![
        c::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
    ]
}

fn get_yaml_val(pattern: &str, default: &str) -> String {
    let re = match RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(r) => r,
        Err(_) => return default.to_string(),
    };
    for p in yaml_paths() {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Some(m) = re.captures(&content) {
                    return m.get(1).map(|s| s.as_str().to_string()).unwrap_or_else(|| default.to_string());
                }
            }
        }
    }
    default.to_string()
}

/// Python: re.sub(pattern, r"\g<1>{replacement}", content) then write back.
fn set_yaml_key(pattern: &str, replacement: &str) {
    let re = match RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(r) => r,
        Err(_) => return,
    };
    for p in yaml_paths() {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                let updated = re.replace_all(&content, |caps: &regex::Captures| format!("{}{}", &caps[1], replacement));
                let _ = std::fs::write(&p, updated.as_ref());
            }
        }
    }
}

fn cli_bin() -> String {
    let home_bin = c::home_dir().join(".local/bin/adguard-cli");
    if home_bin.exists() {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    }
}

fn run_cli_quiet(args: &[String]) {
    let bin = cli_bin();
    let _ = std::process::Command::new(&bin)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

struct Inner {
    root: gtk4::Box,
    lbl_http_t: gtk4::Label,
    lbl_http_desc: gtk4::Label,
    switch_http: ToggleSwitch,
    lbl_https_t: gtk4::Label,
    lbl_https_desc: gtk4::Label,
    switch_https: ToggleSwitch,
    lbl_mode_t: gtk4::Label,
    lbl_mode_sub: gtk4::Label,
    lbl_browser_t: gtk4::Label,
    lbl_browser_sub: gtk4::Label,
    browser_switch: ToggleSwitch,
    open_mode_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_https_cb: RefCell<Option<Box<dyn Fn()>>>,
    /// True while the switches are synced from disk (Python's blockSignals).
    syncing: RefCell<bool>,
}

#[derive(Clone)]
pub struct HomeNetworkSection {
    inner: Rc<Inner>,
}

impl HomeNetworkSection {
    pub fn new() -> HomeNetworkSection {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        // Python: self.setFixedWidth(500) — a HARD width. GTK's set_size_request
        // is only a minimum, so the column grew to ~640px (driven by label
        // natural widths). A CSS class provides the matching max-width.
        root.set_size_request(500, -1);
        root.add_css_class("home-net-fixed");

        // ---- Item 1: Use AdGuard as an HTTP proxy ----
        let proxy_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let row_http = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        let lbl_http_t = gtk4::Label::new(Some("Use AdGuard as an HTTP proxy"));
        lbl_http_t.add_css_class("lbl-title");
        lbl_http_t.set_hexpand(true);
        lbl_http_t.set_halign(gtk4::Align::Start);
        row_http.append(&lbl_http_t);

        let http_val = get_yaml_val(r"http_proxy:\s*(-?[0-9]+)", "3129");
        let is_http_on = http_val != "-1";
        let switch_http = ToggleSwitch::new(46, 26);
        switch_http.set_checked(is_http_on, false);
        row_http.append(switch_http.root());
        proxy_box.append(&row_http);

        let lbl_http_desc = gtk4::Label::new(None);
        lbl_http_desc.add_css_class("lbl-desc-9");
        // Python sets a 500px fixed width on the whole section and lets Qt wrap
        // inside it. A wrapping GTK label reports its full text as its natural
        // width instead, which stretched this column (and every card below it)
        // to ~640px. Constraining the wrap width keeps the 500px column.
        lbl_http_desc.set_wrap(true);
        lbl_http_desc.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        lbl_http_desc.set_width_chars(1);
        lbl_http_desc.set_max_width_chars(1);
        lbl_http_desc.set_halign(gtk4::Align::Start);
        lbl_http_desc.set_xalign(0.0);
        lbl_http_desc.set_markup(&link_markup(DEFAULT_HTTP_DESC));
        proxy_box.append(&lbl_http_desc);
        root.append(&proxy_box);
        root.append(&spacer(4));

        // ---- Item 2: HTTPS filtering ----
        let https_val = get_yaml_val(r"https_filtering:\s*\n\s*enabled:\s*(true|false)", "false");
        let switch_https = ToggleSwitch::new(46, 26);
        switch_https.set_checked(https_val.to_lowercase() == "true", false);

        let (card_https, lbl_https_t, lbl_https_desc) = create_card(
            "HTTPS filtering",
            "Filters traffic sent over HTTPS",
            &switch_https,
            true,
        );
        root.append(&card_https);
        root.append(&spacer(4));

        // ---- Item 3: Filtering mode ----
        let info = backend::get_filtering_mode_info();
        let (mode_row, lbl_mode_t, lbl_mode_sub) = create_nav_row("Filtering mode", &info.title);
        root.append(&mode_row);
        root.append(&spacer(4));

        // ---- Item 4: Browser Traffic Routing (Proxy) ----
        let is_bproxy = backend::is_browser_proxy_enabled();
        let browser_switch = ToggleSwitch::new(46, 26);
        browser_switch.set_checked(is_bproxy, false);
        let lbl_browser_sub = gtk4::Label::new(Some(
            if is_bproxy {
                "Brave, Mullvad, Firefox, Chrome (Active 127.0.0.1:3129)"
            } else {
                "Direct connection (Bypassing AdGuard)"
            },
        ));
        lbl_browser_sub.add_css_class("lbl-desc-9");
        lbl_browser_sub.set_halign(gtk4::Align::Start);
        // Keep the natural width from driving the column: this label reports the
        // full sentence, which pushed the whole section past its 500px designed
        // width (Python's setFixedWidth doesn't let content grow it).
        lbl_browser_sub.set_ellipsize(gtk4::pango::EllipsizeMode::End);

        let card_browser = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        card_browser.add_css_class("settings-card");
        let browser_col = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        browser_col.set_hexpand(true);
        let lbl_browser_t = gtk4::Label::new(Some("Browser Traffic Routing (Proxy)"));
        lbl_browser_t.add_css_class("lbl-title");
        lbl_browser_t.set_halign(gtk4::Align::Start);
        browser_col.append(&lbl_browser_t);
        browser_col.append(&lbl_browser_sub);
        card_browser.append(&browser_col);
        card_browser.append(browser_switch.root());
        root.append(&card_browser);

        let lbl_desc_c = lbl_http_desc.clone();
        let sw1 = switch_http.clone();
        let sw2 = switch_https.clone();
        let sw3 = browser_switch.clone();
        let inner = Rc::new(Inner {
            root,
            lbl_http_t,
            lbl_http_desc,
            switch_http,
            lbl_https_t,
            lbl_https_desc,
            switch_https,
            lbl_mode_t,
            lbl_mode_sub,
            lbl_browser_t,
            lbl_browser_sub,
            browser_switch,
            open_mode_cb: RefCell::new(None),
            open_https_cb: RefCell::new(None),
            syncing: RefCell::new(false),
        });

        // A disk-driven refresh must not trigger the real handlers.
        let sc = inner.clone();
        sw1.connect_toggled(Box::new(move |checked| {
            if *sc.syncing.borrow() {
                return;
            }
            on_http_toggled(checked)
        }));
        let sc2 = inner.clone();
        sw2.connect_toggled(Box::new(move |checked| {
            if *sc2.syncing.borrow() {
                return;
            }
            on_https_toggled(checked)
        }));
        let sc3 = inner.clone();
        sw3.connect_toggled(Box::new(move |checked| {
            if *sc3.syncing.borrow() {
                return;
            }
            on_browser_switch_toggled(&sc3, checked)
        }));

        // HTTPS card click → open_https_requested
        let i5 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i5.open_https_cb.borrow().as_ref() {
                    cb();
                }
            }
        });
        card_https.add_controller(gesture);

        // Filtering mode row click → open_mode_requested
        let i6 = inner.clone();
        let gesture2 = gtk4::GestureClick::new();
        gesture2.connect_pressed(move |_g, n, _x, _y| {
            if n == 1 {
                if let Some(cb) = i6.open_mode_cb.borrow().as_ref() {
                    cb();
                }
            }
        });
        mode_row.add_controller(gesture2);

        // The "AdGuard root certificate" link inside the HTTP proxy description
        lbl_desc_c.connect_activate_link(Box::new(move |_l: &gtk4::Label, _uri: &str| {
            on_cert_link_clicked();
            gtk4::glib::Propagation::Stop
        }));

        let i11 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i11)));
        retranslate_ui(&inner);

        HomeNetworkSection { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    pub fn connect_open_mode_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_mode_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_https_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_https_cb.borrow_mut() = Some(cb);
    }

    pub fn refresh_states(&self) {
        refresh_states(&self.inner);
    }
}

fn create_card(
    title: &str,
    desc: &str,
    switch: &ToggleSwitch,
    clickable: bool,
) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let card = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    card.add_css_class("settings-card");
    if clickable {
        card.add_css_class("settings-card-clickable");
    }

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    col.set_hexpand(true);
    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("lbl-title");
    lbl_t.set_halign(gtk4::Align::Start);
    col.append(&lbl_t);
    let lbl_d = gtk4::Label::new(Some(desc));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_halign(gtk4::Align::Start);
    col.append(&lbl_d);
    card.append(&col);
    card.append(switch.root());
    (card, lbl_t, lbl_d)
}

fn create_nav_row(title: &str, subtitle: &str) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    row.add_css_class("nav-row");

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
    col.set_hexpand(true);
    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("nav-row-title");
    lbl_t.set_halign(gtk4::Align::Start);
    col.append(&lbl_t);
    let lbl_sub = gtk4::Label::new(Some(subtitle));
    lbl_sub.add_css_class("lbl-desc-9");
    lbl_sub.set_halign(gtk4::Align::Start);
    lbl_sub.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    col.append(&lbl_sub);
    row.append(&col);
    row.append(&svg::svg_image("chevron_right.svg", 14));
    (row, lbl_t, lbl_sub)
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn on_http_toggled(checked: bool) {
    let val_str = if checked { "3129" } else { "-1" };
    set_yaml_key(r"(http_proxy:\s*)-?[0-9]+", val_str);
    run_cli_quiet(&["config".into(), "set".into(), "http_proxy".into(), val_str.to_string()]);

    if checked {
        backend::configure_browser_proxy();
    } else {
        backend::disable_browser_proxy();
    }
}

fn on_https_toggled(checked: bool) {
    let val_str = if checked { "true" } else { "false" };
    set_yaml_key(r"(https_filtering:\s*\n\s*enabled:\s*)(true|false)", val_str);
    let bin = cli_bin();
    let args: Vec<String> = vec!["config".into(), "set".into(), "https_filtering.enabled".into(), val_str.to_string()];
    let _ = std::process::Command::new("sudo")
        .arg("-n")
        .arg(&bin)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    run_cli_quiet(&args);
    if checked {
        backend::install_certificate();
    }
}

fn on_cert_link_clicked() {
    backend::install_certificate();
    let cert_path = c::home_dir().join(".local/share/adguard-cli/AdGuard CLI CA.pem");
    if cert_path.exists() {
        show_message(
            None,
            "AdGuard Root Certificate",
            &format!(
                "The AdGuard Root CA certificate is located at:\n{}\n\nIt has been installed and configured for system trust and browsers (Mullvad Browser, Firefox, Brave, Chrome).",
                cert_path.display()
            ),
        );
    } else {
        show_message(None, "AdGuard Root Certificate", "Generate the certificate by running:\nadguard-cli cert");
    }
}

fn on_browser_switch_toggled(inner: &Rc<Inner>, is_checked: bool) {
    if is_checked {
        backend::configure_browser_proxy();
        inner.lbl_browser_sub.set_text(&i18n::tr("browser_proxy_active"));
    } else {
        backend::disable_browser_proxy();
        inner.lbl_browser_sub.set_text(&i18n::tr("browser_proxy_direct"));
    }
}

fn mode_sub_text() -> String {
    let info = backend::get_filtering_mode_info();
    if info.mode == "manual" {
        format!("{} • {} {}", i18n::tr("mode_manual_t"), i18n::tr("mode_port_label"), info.port)
    } else {
        i18n::tr("mode_auto_t")
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.lbl_http_t.set_text(&i18n::tr("http_proxy_title"));
    inner.lbl_http_desc.set_markup(&link_markup(&i18n::tr("http_proxy_desc")));
    inner.lbl_https_t.set_text(&i18n::tr("https_filtering_title"));
    inner.lbl_https_desc.set_text(&i18n::tr("https_filtering_desc"));
    inner.lbl_mode_t.set_text(&i18n::tr("filtering_mode_title"));
    inner.lbl_mode_sub.set_text(&mode_sub_text());
    inner.lbl_browser_t.set_text(&i18n::tr("browser_proxy_title"));
    let is_bproxy = backend::is_browser_proxy_enabled();
    inner.lbl_browser_sub.set_text(&if is_bproxy {
        i18n::tr("browser_proxy_active")
    } else {
        i18n::tr("browser_proxy_direct")
    });
}

fn refresh_states(inner: &Rc<Inner>) {
    // Python wraps every setChecked in blockSignals(True/False) here: refreshing
    // the switches from disk must not look like a user click. Without the guard
    // the periodic 4 s status refresh rewrote proxy.yaml and invoked adguard-cli
    // on every tick.
    *inner.syncing.borrow_mut() = true;
    let http_val = get_yaml_val(r"http_proxy:\s*(-?[0-9]+)", "3129");
    let is_http_on = http_val != "-1";
    inner.switch_http.set_checked(is_http_on, false);

    let https_val = get_yaml_val(r"https_filtering:\s*\n\s*enabled:\s*(true|false)", "false");
    inner.switch_https.set_checked(https_val.to_lowercase() == "true", false);

    let is_bproxy = backend::is_browser_proxy_enabled();
    inner.browser_switch.set_checked(is_bproxy, false);
    *inner.syncing.borrow_mut() = false;

    inner.lbl_mode_sub.set_text(&mode_sub_text());

    inner.lbl_browser_sub.set_text(&if is_bproxy {
        i18n::tr("browser_proxy_active")
    } else {
        i18n::tr("browser_proxy_direct")
    });
}

#[cfg(test)]
mod markup_tests {
    use super::*;

    /// The Qt strings carry `style='...'` on the anchor; Pango rejects it and the
    /// label then shows nothing (or raw HTML). Verified against all 4 real keys.
    #[test]
    fn qt_style_attributes_are_removed_for_pango() {
        let cases = [
            "<a href='cert' style='color: #5EBB6A; text-decoration: none;'>AdGuard root certificate</a>",
            "<a href='https://adguard.com' style='color: #68BC71; text-decoration: none;'>Learn about app compatibility</a>",
            "<a href='https://adguard.com/privacy.html' style='color: #5EBB6A; text-decoration: underline;'>send usage data</a> anonymous",
        ];
        for raw in cases {
            let cleaned = strip_qt_style_attrs(raw);
            assert!(!cleaned.contains("style="), "style attr left in {cleaned:?}");
            assert!(!cleaned.contains("color:"), "colour leftover in {cleaned:?}");
            assert!(!cleaned.contains("text-decoration"), "decoration left in {cleaned:?}");
            assert!(cleaned.contains("<a href="), "anchor lost in {cleaned:?}");
            assert!(cleaned.contains("</a>"), "closing anchor lost in {cleaned:?}");
            assert!(!cleaned.contains(" >"), "stray space before '>' in {cleaned:?}");
        }
    }

    /// Every i18n value that contains HTML must survive conversion.
    #[test]
    fn real_i18n_link_keys_convert_cleanly() {
        let table: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
            serde_json::from_str(crate::i18n::raw_data()).expect("valid json");
        for (lang, entries) in table {
            for (key, text) in entries {
                if !text.contains("<a href") && !text.contains("<span") {
                    continue;
                }
                let out = to_pango_link_markup(&text, "#8E8E93");
                assert!(!out.contains("style="), "{lang}/{key} still has style=");
                assert!(out.starts_with("<span"), "{lang}/{key} not wrapped");
                assert!(out.ends_with("</span>"), "{lang}/{key} not closed");
            }
        }
    }
}
