// Port of adguard_modern/tabs/settings_about_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::widgets::common_controls::BackButton;
use regex::Regex;

pub struct VersionInfo {
    pub app_title: String,
    pub cli_info: String,
    pub corelibs: String,
    pub dnslibs: String,
    pub scriptlets: String,
}

/// Port of get_real_system_adguard_versions(): real installed versions from the
/// local adguard-cli binary.
pub fn get_real_system_adguard_versions() -> VersionInfo {
    let home_bin = dirs_home_bin();
    let bin_path = if std::path::Path::exists(&home_bin) {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    };

    let mut cli_ver = "1.4.13".to_string();
    let corelibs_ver = "1.22.13";
    let dnslibs_ver = "2.0.52";
    let scriptlets_ver = "2.4.2";

    if let Ok(out) = std::process::Command::new(&bin_path).arg("-v").output() {
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        if let Some(caps) = Regex::new(r"v?([0-9]+\.[0-9]+\.[0-9]+)").unwrap().captures(&stdout) {
            cli_ver = caps[1].to_string();
        }
    }

    VersionInfo {
        app_title: "AdGuard 8.0.0 (5560)".to_string(),
        cli_info: format!("Core engine: AdGuard CLI v{} (CachyOS)", cli_ver),
        corelibs: format!("CoreLibs version: {}", corelibs_ver),
        dnslibs: format!("DnsLibs version: {}", dnslibs_ver),
        scriptlets: format!("Scriptlets version: {}", scriptlets_ver),
    }
}

fn dirs_home_bin() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    std::path::Path::new(&home).join(".local/bin/adguard-cli")
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsAboutView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsAboutView {
    pub fn new() -> SettingsAboutView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(24);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(18));

        let lbl_title = gtk4::Label::new(Some("About"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(28));

        let vers = get_real_system_adguard_versions();

        let lbl_app = gtk4::Label::new(Some(&vers.app_title));
        lbl_app.add_css_class("lbl-white-12-bold");
        lbl_app.set_halign(gtk4::Align::Start);
        root.append(&lbl_app);
        root.append(&spacer(14));

        for lib_text in [&vers.corelibs, &vers.dnslibs, &vers.scriptlets, &vers.cli_info] {
            let lbl_lib = gtk4::Label::new(Some(lib_text));
            lbl_lib.add_css_class("lbl-white-10");
            lbl_lib.set_halign(gtk4::Align::Start);
            root.append(&lbl_lib);
            root.append(&spacer(4));
        }
        root.append(&spacer(18));

        let lbl_latest = gtk4::Label::new(Some("You are using the latest version of AdGuard"));
        lbl_latest.add_css_class("lbl-white-10");
        lbl_latest.set_halign(gtk4::Align::Start);
        root.append(&lbl_latest);
        root.append(&spacer(24));

        let lbl_copy = gtk4::Label::new(Some("2009–2026 AdGuard Software Ltd. All rights reserved"));
        lbl_copy.add_css_class("lbl-white-10");
        lbl_copy.set_halign(gtk4::Align::Start);
        root.append(&lbl_copy);
        root.append(&spacer(22));

        // Green links (webbrowser.open -> xdg-open).
        let links = [
            ("Privacy policy", "https://adguard.com/privacy.html"),
            ("License agreement", "https://adguard.com/eula.html"),
            ("Official website", "https://adguard.com"),
            ("Acknowledgments", "https://adguard.com/acknowledgments.html"),
            ("Version history", "https://adguard.com/release-notes.html"),
        ];
        for (text, url) in links {
            let btn_link = gtk4::Button::new();
            btn_link.set_label(text);
            btn_link.add_css_class("about-link");
            let u = url.to_string();
            btn_link.connect_clicked(move |_| {
                let _ = std::process::Command::new("xdg-open").arg(&u).spawn();
            });
            root.append(&btn_link);
            root.append(&spacer(4));
        }

        let bb_root = btn_back.root.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            back_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        SettingsAboutView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }
}
