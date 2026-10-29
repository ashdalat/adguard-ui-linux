// Port of adguard_modern/widgets/assistant_modal.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use crate::constants as c;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

#[derive(Clone)]
pub struct BrowserInfo {
    pub name: String,
    pub icon: String,
    pub path: String,
    pub is_default: bool,
}

/// Port of detect_system_browsers()
pub fn detect_system_browsers() -> (BrowserInfo, Vec<BrowserInfo>) {
    let mut default_cmd = String::new();
    if let Ok(out) = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .stderr(std::process::Stdio::null())
        .output()
    {
        default_cmd = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
    }

    let supported: [(&str, &[&str], &str, &str); 6] = [
        ("Brave", &["brave", "brave-browser"], "app_brave.svg", "brave"),
        ("Firefox", &["firefox"], "app_fire.svg", "firefox"),
        ("Mullvad Browser", &["mullvad-browser"], "app_browser.svg", "mullvad"),
        ("Google Chrome", &["google-chrome", "google-chrome-stable"], "app_browser.svg", "chrome"),
        ("Chromium", &["chromium"], "app_browser.svg", "chromium"),
        ("Microsoft Edge", &["microsoft-edge", "microsoft-edge-stable"], "app_edge.svg", "edge"),
    ];

    let mut installed: Vec<BrowserInfo> = Vec::new();
    for (name, bins, icon, _id) in supported {
        let mut path = String::new();
        for bin_name in bins {
            if let Some(p) = which(bin_name) {
                path = p;
                break;
            }
        }
        if !path.is_empty() {
            let is_default = bins.iter().any(|b| default_cmd.contains(b));
            installed.push(BrowserInfo {
                name: name.to_string(),
                icon: icon.to_string(),
                path,
                is_default,
            });
        }
    }

    let mut default_b: Option<BrowserInfo> = None;
    let mut others: Vec<BrowserInfo> = Vec::new();
    for b in installed.iter() {
        if b.is_default && default_b.is_none() {
            default_b = Some(b.clone());
        } else {
            others.push(b.clone());
        }
    }
    if default_b.is_none() && !installed.is_empty() {
        default_b = Some(installed[0].clone());
        others = installed[1..].to_vec();
    }
    if default_b.is_none() {
        default_b = Some(BrowserInfo {
            name: "Brave".to_string(),
            icon: "app_brave.svg".to_string(),
            path: String::new(),
            is_default: true,
        });
    }

    (default_b.unwrap(), others)
}

fn which(name: &str) -> Option<String> {
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(':') {
        let cand = std::path::Path::new(dir).join(name);
        if cand.is_file() {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(m) = std::fs::metadata(&cand) {
                if m.permissions().mode() & 0o111 != 0 {
                    return Some(cand.display().to_string());
                }
            }
        }
    }
    None
}

/// Port of BrowserRow
pub fn browser_row(icon_name: &str, name: &str, subtitle: &str, status_text: &str) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    row.add_css_class("browser-row");
    row.set_margin_start(8);
    row.set_margin_end(8);
    row.set_margin_top(8);
    row.set_margin_bottom(8);

    let mut icon = c::icons_dir().join(icon_name);
    if !icon.exists() {
        icon = c::icons_dir().join("app_browser.svg");
    }
    if let Some(tex) = svg::svg_texture(&icon, 22) {
        let img = gtk4::Image::new();
        img.set_paintable(Some(&tex));
        img.set_size_request(22, 22);
        row.append(&img);
    }

    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    text.set_hexpand(true);
    let lbl_name = gtk4::Label::new(Some(name));
    lbl_name.add_css_class("lbl-title-10");
    lbl_name.set_halign(gtk4::Align::Start);
    text.append(&lbl_name);
    if !subtitle.is_empty() {
        let lbl_sub = gtk4::Label::new(Some(subtitle));
        lbl_sub.add_css_class("lbl-sub-8");
        lbl_sub.set_halign(gtk4::Align::Start);
        text.append(&lbl_sub);
    }
    row.append(&text);

    if !status_text.is_empty() {
        let lbl_status = gtk4::Label::new(Some(status_text));
        lbl_status.add_css_class("lbl-desc-9");
        row.append(&lbl_status);
    }

    row
}

pub struct AdGuardAssistantModal {
    pub root: gtk4::Window,
    other_expanded: RefCell<bool>,
    other_container: gtk4::Box,
    btn_other_header: gtk4::Button,
}

impl AdGuardAssistantModal {
    pub fn new() -> Rc<AdGuardAssistantModal> {
        let root = crate::util::sub_window("AdGuard Assistant");
        root.set_default_size(520, 500);

        let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        outer.set_margin_start(10);
        outer.set_margin_end(10);
        outer.set_margin_top(10);
        outer.set_margin_bottom(10);
        root.set_child(Some(&outer));

        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        card.add_css_class("assistant-card");
        card.set_margin_start(24);
        card.set_margin_end(24);
        card.set_margin_top(20);
        card.set_margin_bottom(20);
        outer.append(&card);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("AdGuard Assistant"));
        lbl_title.add_css_class("assistant-title");
        lbl_title.set_halign(gtk4::Align::Start);
        header.append(&lbl_title);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(28, 28);
        btn_close.set_label("✕");
        let r2 = root.clone();
        btn_close.connect_clicked(move |_| r2.set_visible(false));
        header.append(&btn_close);
        card.append(&header);

        let lbl_desc = gtk4::Label::new(Some(
            "Our companion extension allows you to manually block page elements, \
             manage AdGuard, and report issues from your browser.",
        ));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_halign(gtk4::Align::Start);
        card.append(&lbl_desc);

        let btn_link = gtk4::Button::new();
        btn_link.add_css_class("green-link");
        btn_link.set_label("More details");
        btn_link.connect_clicked(|_| {
            let _ = Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/adguard-assistant/")
                .spawn();
        });
        card.append(&btn_link);

        let (default_b, others) = detect_system_browsers();
        card.append(&browser_row(&default_b.icon, &default_b.name, "Default browser", "Installed"));

        let btn_other_header = gtk4::Button::new();
        btn_other_header.add_css_class("collapse-btn");
        btn_other_header.set_label("⌄   Other browsers");
        card.append(&btn_other_header);

        let other_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        if !others.is_empty() {
            for ob in &others {
                let status = if ob.path.is_empty() { "Available" } else { "Installed" };
                other_container.append(&browser_row(&ob.icon, &ob.name, status, ""));
            }
        } else {
            other_container.append(&browser_row("app_fire.svg", "Firefox", "Available", ""));
            other_container.append(&browser_row("app_edge.svg", "Microsoft Edge", "Available", ""));
        }
        card.append(&other_container);

        let modal = Rc::new(AdGuardAssistantModal {
            root: root.clone(),
            other_expanded: RefCell::new(true),
            other_container: other_container.clone(),
            btn_other_header: btn_other_header.clone(),
        });

        let m3 = modal.clone();
        btn_other_header.connect_clicked(move |_| {
            let expanded = !*m3.other_expanded.borrow();
            *m3.other_expanded.borrow_mut() = expanded;
            m3.other_container.set_visible(expanded);
            let arrow = if expanded { "⌄" } else { "›" };
            m3.btn_other_header.set_label(&format!("{}   Other browsers", arrow));
        });

        // Legacy Assistant row
        let legacy = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        legacy.set_margin_start(8);
        legacy.set_margin_end(8);
        legacy.set_margin_top(8);
        legacy.set_margin_bottom(8);

        let leg_text = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        leg_text.set_hexpand(true);
        let lbl_leg_title = gtk4::Label::new(Some("Legacy Assistant"));
        lbl_leg_title.add_css_class("lbl-title-10");
        lbl_leg_title.set_halign(gtk4::Align::Start);
        leg_text.append(&lbl_leg_title);
        let lbl_leg_sub = gtk4::Label::new(Some(
            "Lets you manage AdGuard directly on the webpage. \
             Works in all browsers but offers fewer options than the extension",
        ));
        lbl_leg_sub.add_css_class("lbl-sub-8");
        lbl_leg_sub.set_wrap(true);
        lbl_leg_sub.set_halign(gtk4::Align::Start);
        leg_text.append(&lbl_leg_sub);
        legacy.append(&leg_text);

        let switch_legacy = ToggleSwitch::new(46, 26);
        switch_legacy.set_checked(true, false);
        legacy.append(switch_legacy.root());
        card.append(&legacy);

        modal
    }
}
