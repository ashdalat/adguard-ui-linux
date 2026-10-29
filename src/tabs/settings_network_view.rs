// Port of adguard_modern/tabs/settings_network_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::widgets::common_controls::BackButton;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;
use crate::widgets::filtering_log_window::show_message;
use regex::RegexBuilder;

fn yaml_paths() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    vec![
        "/root/.local/share/adguard-cli/proxy.yaml".to_string(),
        std::path::Path::new(&home)
            .join(".local/share/adguard-cli/proxy.yaml")
            .to_string_lossy()
            .to_string(),
    ]
}

fn get_yaml_val(pattern: &str, default: &str) -> String {
    for p in yaml_paths() {
        if !std::path::Path::exists(&std::path::PathBuf::from(&p)) {
            continue;
        }
        if let Ok(c) = std::fs::read_to_string(&p) {
            if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                if let Some(m) = re.captures(&c) {
                    return m[1].to_string();
                }
            }
        }
    }
    default.to_string()
}

fn set_yaml_key(pattern: &str, replacement: &str) {
    for p in yaml_paths() {
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

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    switch_http: ToggleSwitch,
    lbl_http_desc: gtk4::Label,
    lbl_port: gtk4::Label,
    input_port: gtk4::Entry,
    switch_updates_proxy: ToggleSwitch,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_mode_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_https_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_updates_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsNetworkView {
    inner: Rc<Inner>,
}

impl SettingsNetworkView {
    pub fn new() -> SettingsNetworkView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);
        root.append(&top_bar);
        root.append(&spacer(22));

        let lbl_title = gtk4::Label::new(Some("Network"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(26));

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
        container.add_css_class("transparent-container");
        container.set_margin_end(12);
        container.set_margin_bottom(24);

        // Section 1: HTTP proxy.
        let proxy_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let row_http = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);

        let lbl_http_t = gtk4::Label::new(Some("Use AdGuard as an HTTP proxy"));
        lbl_http_t.add_css_class("lbl-11-bold-white");
        lbl_http_t.set_hexpand(true);
        lbl_http_t.set_valign(gtk4::Align::Center);
        row_http.append(&lbl_http_t);

        let http_val = get_yaml_val(r"http_proxy:\s*(-?[0-9]+)", "3129");
        let is_http_on = http_val != "-1";
        let switch_http = ToggleSwitch::new(46, 26);
        switch_http.set_checked(is_http_on, false);
        switch_http.root().set_valign(gtk4::Align::Center);
        row_http.append(switch_http.root());
        proxy_box.append(&row_http);

        let lbl_http_desc = gtk4::Label::new(Some(&format!(
            "All traffic passing through this proxy will be filtered. You can also use it to filter traffic on other devices in your local network: install the <span color=\"#5EBB6A\"><a href=\"cert\">AdGuard root certificate</a></span> on them and configure the proxy"
        )));
        lbl_http_desc.add_css_class("lbl-desc-9");
        lbl_http_desc.set_use_markup(true);
        lbl_http_desc.set_wrap(true);
        lbl_http_desc.set_xalign(0.0);
        proxy_box.append(&lbl_http_desc);
        proxy_box.append(&spacer(6));

        let lbl_port = gtk4::Label::new(Some("Port"));
        lbl_port.add_css_class("lbl-10-white");
        lbl_port.set_halign(gtk4::Align::Start);
        proxy_box.append(&lbl_port);

        let port_display = if is_http_on && http_val != "-1" {
            http_val.clone()
        } else {
            "3129".to_string()
        };
        let input_port = gtk4::Entry::new();
        input_port.set_text(&port_display);
        input_port.set_size_request(-1, 44);
        input_port.add_css_class("port-input");
        proxy_box.append(&input_port);

        container.append(&proxy_box);
        container.append(&spacer(8));

        // Section 2: cards.
        let proxy_val = get_yaml_val(r"outbound_proxy:\s*\n\s*enabled:\s*(true|false)", "false");
        let switch_updates_proxy = ToggleSwitch::new(46, 26);
        switch_updates_proxy.set_checked(proxy_val.to_lowercase() == "true", false);

        let bb_root = btn_back.root.clone();
        let ip = input_port.clone();
        let desc = lbl_http_desc.clone();
        // Python: scroll.setWidget(container); main_layout.addWidget(scroll)
        scroll.set_child(Some(&container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            switch_http,
            lbl_http_desc,
            lbl_port,
            input_port,
            switch_updates_proxy,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            open_mode_cb: RefCell::new(None),
            open_https_cb: RefCell::new(None),
            open_updates_cb: RefCell::new(None),
        });

        // Card click -> open the proxy-for-updates subview (Python on_click=emit).
        let i0 = inner.clone();
        let card = create_card(
            "Proxy for app and filter updates",
            "Allows you to select a proxy server for updating the app and filters",
            &inner.switch_updates_proxy,
            Some(Box::new(move || {
                if let Some(cb) = i0.open_updates_cb.borrow().as_ref() {
                    cb();
                }
            })),
        );
        container.append(&card);

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        ip.connect_activate(move |e| {
            on_port_changed(&i2, &e.text());
        });

        let i3 = inner.clone();
        desc.connect_activate_link(move |_l: &gtk4::Label, _uri: &str| {
            on_cert_link_clicked(&i3);
            gtk4::glib::Propagation::Stop
        });

        let i4 = inner.clone();
        inner.switch_http.connect_toggled(Box::new(move |checked| on_http_toggled(&i4, checked)));

        let i5 = inner.clone();
        inner.switch_updates_proxy.connect_toggled(Box::new(move |checked| {
            on_updates_proxy_toggled(&i5, checked)
        }));

        SettingsNetworkView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_mode_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_mode_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_https_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_https_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_updates_proxy_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_updates_cb.borrow_mut() = Some(cb);
    }

    /// Python refresh_states() is `pass` — nothing to refresh here.
    pub fn refresh_states(&self) {}

}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn create_card(
    title: &str,
    desc: &str,
    switch_widget: &ToggleSwitch,
    on_click: Option<Box<dyn Fn()>>,
) -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    card.add_css_class("settings-card");
    if on_click.is_some() {
        card.add_css_class("settings-card-clickable");
    }

    let v = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    v.set_hexpand(true);

    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("lbl-11-bold-white");
    lbl_t.set_halign(gtk4::Align::Start);
    v.append(&lbl_t);

    let lbl_d = gtk4::Label::new(Some(desc));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_halign(gtk4::Align::Start);
    v.append(&lbl_d);

    card.append(&v);
    switch_widget.root().set_valign(gtk4::Align::Center);
    card.append(switch_widget.root());

    if let Some(cb) = on_click {
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                cb();
            }
        });
        card.add_controller(gesture);
    }

    card
}

#[allow(dead_code)]
fn create_nav_row(title: &str, subtitle: &str, on_click: Box<dyn Fn()>) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    row.add_css_class("nav-row");
    row.set_margin_top(10);
    row.set_margin_bottom(10);
    row.set_margin_start(8);
    row.set_margin_end(12);

    let v = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
    v.set_hexpand(true);

    let lbl_t = gtk4::Label::new(Some(title));
    lbl_t.add_css_class("lbl-title-10");
    lbl_t.set_halign(gtk4::Align::Start);
    v.append(&lbl_t);

    let lbl_sub = gtk4::Label::new(Some(subtitle));
    lbl_sub.add_css_class("lbl-desc-9");
    lbl_sub.set_halign(gtk4::Align::Start);
    v.append(&lbl_sub);

    row.append(&v);

    let chev = svg::svg_image("chevron_right.svg", 14);
    chev.set_valign(gtk4::Align::Center);
    row.append(&chev);

    let cb = Rc::new(on_click);
    let cb2 = cb.clone();
    let gesture = gtk4::GestureClick::new();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        // ignore clicks while the row is disabled
        if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
        if n == 1 {
            cb2();
        }
    });
    row.add_controller(gesture);

    row
}

fn on_http_toggled(inner: &Rc<Inner>, checked: bool) {
    let port_val = inner.input_port.text().trim().to_string();
    let port_val = if port_val.is_empty() { "3129".to_string() } else { port_val };
    let val_str = if checked { port_val } else { "-1".to_string() };
    set_yaml_key(
        r"(http_proxy:\s*)-?[0-9]+",
        &format!("${{1}}{}", val_str),
    );
}

fn on_port_changed(inner: &Rc<Inner>, text: &str) {
    if inner.switch_http.is_checked() {
        let port_val = text.trim().to_string();
        let port_val = if port_val.is_empty() { "3129".to_string() } else { port_val };
        set_yaml_key(r"(http_proxy:\s*)-?[0-9]+", &format!("${{1}}{}", port_val));
    }
}

#[allow(dead_code)]
fn on_https_toggled(checked: bool) {
    let val_str = if checked { "true" } else { "false" };
    set_yaml_key(
        r"(https_filtering:\s*\n\s*enabled:\s*)(true|false)",
        &format!("${{1}}{}", val_str),
    );
}

fn on_updates_proxy_toggled(inner: &Rc<Inner>, checked: bool) {
    if checked {
        // Python refuses to enable outbound_proxy to localhost without an upstream
        // proxy running and redirects the user to the configuration sub-page.
        let host = get_yaml_val(r#"host:\s*['"]?([^'"\n]+)['"]?"#, "");
        if host.is_empty() || host == "localhost" || host == "127.0.0.1" {
            *inner.syncing.borrow_mut() = true;
            inner.switch_updates_proxy.set_checked(false, true);
            *inner.syncing.borrow_mut() = false;
            if let Some(cb) = inner.open_updates_cb.borrow().as_ref() {
                cb();
            }
            return;
        }
    }
    let val_str = if checked { "true" } else { "false" };
    set_yaml_key(
        r"(outbound_proxy:\s*\n\s*enabled:\s*)(true|false)",
        &format!("${{1}}{}", val_str),
    );
    let home = std::env::var("HOME").unwrap_or_default();
    let home_bin = std::path::Path::new(&home).join(".local/bin/adguard-cli");
    let bin_path = if std::path::Path::exists(&home_bin) {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    };
    let _ = std::process::Command::new("sudo")
        .args(["-n", &bin_path, "config", "set", "outbound_proxy.enabled", val_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::process::Command::new(&bin_path)
        .args(["config", "set", "outbound_proxy.enabled", val_str])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn on_cert_link_clicked(_inner: &Rc<Inner>) {
    let home = std::env::var("HOME").unwrap_or_default();
    let cert_path = std::path::Path::new(&home)
        .join(".local/share/adguard-cli/AdGuard CLI CA.pem");
    if std::path::Path::exists(&cert_path) {
        show_message(
            None,
            "AdGuard Root Certificate",
            &format!(
                "The AdGuard Root CA certificate is located at:\n{}\n\nInstall this certificate on your devices or browsers to filter HTTPS traffic.",
                cert_path.display()
            ),
        );
    } else {
        show_message(
            None,
            "AdGuard Root Certificate",
            "Generate the certificate by running:\nadguard-cli cert",
        );
    }
}
