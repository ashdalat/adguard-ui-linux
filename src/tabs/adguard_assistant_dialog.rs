// Port of adguard_modern/tabs/adguard_assistant_dialog.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::constants;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

use super::apps_scanner;

pub struct AdGuardAssistantDialog {
    root: gtk4::Window,
}

impl AdGuardAssistantDialog {
    pub fn new() -> AdGuardAssistantDialog {
        let root = crate::util::sub_window("AdGuard Assistant");
        root.set_default_size(520, 520);
        root.set_modal(true);
        root.set_decorated(false);
        root.add_css_class("assistant-dialog-root");


        // Outer container frame (card)
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        container.set_margin_start(24);
        container.set_margin_end(24);
        container.set_margin_top(24);
        container.set_margin_bottom(24);
        container.add_css_class("assistant-dialog-card");
        root.set_child(Some(&container));

        // 1. Header: title + close (✕)
        let head_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let title_lbl = gtk4::Label::new(Some("AdGuard Assistant"));
        title_lbl.add_css_class("assistant-title-22");
        head_row.append(&title_lbl);

        let close_btn = gtk4::Button::new();
        close_btn.add_css_class("assistant-close-btn");
        close_btn.set_label("✕");
        close_btn.set_size_request(24, 24);
        let root_for_close = root.clone();
        close_btn.connect_clicked(move |_| {
            root_for_close.close();
        });
        head_row.append(&close_btn);
        container.append(&head_row);

        // 2. Description + More details link
        let desc_lbl = gtk4::Label::new(Some(
            "Our companion extension allows you to manually block page elements, manage AdGuard, and report issues from your browser.",
        ));
        desc_lbl.add_css_class("assistant-desc-13");
        desc_lbl.set_wrap(true);
        container.append(&desc_lbl);

        let more_btn = gtk4::Button::new();
        more_btn.add_css_class("assistant-more-btn");
        more_btn.set_label("More details");
        more_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/assistant/")
                .spawn();
        });
        container.append(&more_btn);

        let spacer6 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer6.set_size_request(-1, 6);
        container.append(&spacer6);

        // 3. Real installed browsers detection
        let (installed_browsers, default_browser) = Self::scan_system_browsers();

        if let Some(db) = &default_browser {
            let def_row = Self::create_browser_row(&db.icon, &db.name, "Default browser", "Installed");
            container.append(&def_row);
        }

        let spacer4 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer4.set_size_request(-1, 4);
        container.append(&spacer4);

        // 4. Collapsible "Other browsers" section
        let other_expanded = Rc::new(RefCell::new(true));
        let other_browsers: Vec<apps_scanner::AppEntry> = if let Some(db) = &default_browser {
            installed_browsers
                .into_iter()
                .filter(|b| b.name != db.name)
                .collect()
        } else {
            installed_browsers
        };

        if !other_browsers.is_empty() {
            let toggle_btn = gtk4::Button::new();
            toggle_btn.add_css_class("assistant-other-toggle");
            toggle_btn.set_label("⌃  Other browsers");

            let other_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
            other_box.add_css_class("assistant-other-container");

            for b in &other_browsers {
                let row = Self::create_browser_row(&b.icon, &b.name, "", "Installed");
                other_box.append(&row);
            }

            let ex = other_expanded.clone();
            let ob = other_box.clone();
            let tb = toggle_btn.clone();
            toggle_btn.connect_clicked(move |_| {
                let expanded = !*ex.borrow();
                *ex.borrow_mut() = expanded;
                ob.set_visible(expanded);
                tb.set_label(if expanded { "⌃  Other browsers" } else { "⌄  Other browsers" });
            });

            container.append(&toggle_btn);
            container.append(&other_box);
        }

        // Python: card_layout.addStretch() — pushes the legacy card to the bottom.
        let stretch = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        stretch.set_vexpand(true);
        container.append(&stretch);

        // 5. Legacy Assistant card at bottom
        let legacy_card = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        legacy_card.add_css_class("assistant-legacy-card");
        legacy_card.set_margin_start(16);
        legacy_card.set_margin_end(16);
        legacy_card.set_margin_top(14);
        legacy_card.set_margin_bottom(14);

        let legacy_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        let legacy_title = gtk4::Label::new(Some("Legacy Assistant"));
        legacy_title.add_css_class("assistant-legacy-title");
        legacy_vbox.append(&legacy_title);
        let legacy_sub = gtk4::Label::new(Some(
            "Lets you manage AdGuard directly on the webpage. Works in all browsers but offers fewer options than the extension",
        ));
        legacy_sub.add_css_class("assistant-legacy-sub");
        legacy_sub.set_wrap(true);
        legacy_vbox.append(&legacy_sub);
        legacy_card.append(&legacy_vbox);

        let legacy_switch = ToggleSwitch::new(46, 26);
        legacy_switch.set_checked(true, false);
        legacy_switch.connect_toggled(Box::new(|is_active| {
            Self::on_legacy_assistant_toggled(is_active);
        }));
        legacy_card.append(legacy_switch.root());
        container.append(&legacy_card);

        AdGuardAssistantDialog { root }
    }

    fn scan_system_browsers() -> (Vec<apps_scanner::AppEntry>, Option<apps_scanner::AppEntry>) {
        let (browsers, _, _) = apps_scanner::scan_real_system_apps();

        // Deduplicate by browser name
        let mut unique_browsers: Vec<apps_scanner::AppEntry> = Vec::new();
        let mut seen_names: Vec<String> = Vec::new();
        for b in &browsers {
            if !seen_names.contains(&b.name) {
                seen_names.push(b.name.clone());
                unique_browsers.push(b.clone());
            }
        }

        // Detect default browser from xdg-settings
        let mut default_browser: Option<apps_scanner::AppEntry> = None;
        if let Ok(out) = std::process::Command::new("xdg-settings")
            .args(["get", "default-web-browser"])
            .output()
        {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let out_low = s.to_lowercase();
                for b in &unique_browsers {
                    let b_exec = b.exec.to_lowercase();
                    let file_base = std::path::Path::new(&b.file)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if (!b_exec.is_empty() && out_low.contains(&b_exec))
                        || (!file_base.is_empty() && out_low.contains(&file_base))
                    {
                        default_browser = Some(b.clone());
                        break;
                    }
                }
            }
        }

        if default_browser.is_none() {
            if let Some(first) = unique_browsers.first() {
                default_browser = Some(first.clone());
            }
        }

        (unique_browsers, default_browser)
    }

    fn resolve_browser_icon(icon_hint: &str, name: &str) -> gtk4::Image {
        let img = gtk4::Image::new();
        img.set_size_request(26, 26);

        // 1. System icon theme
        if !icon_hint.is_empty() {
            if let Some(display) = gtk4::gdk::Display::default() {
                let theme = gtk4::IconTheme::for_display(&display);
                if theme.has_icon(icon_hint) {
                    img.set_icon_name(Some(icon_hint));
                    return img;
                }
            }
        }

        // 2. Match local SVG icons in assets/icons
        let low_name = name.to_lowercase();
        let specific = if low_name.contains("brave") {
            Some("app_brave.svg")
        } else if low_name.contains("firefox") {
            Some("app_fire.svg")
        } else if low_name.contains("chrome") {
            Some("app_chrome.svg")
        } else if low_name.contains("edge") {
            Some("app_edge.svg")
        } else {
            None
        };
        if let Some(fname) = specific {
            let p = constants::icon_path(fname);
            if p.exists() {
                if let Some(tex) = svg::svg_texture(&p, 26) {
                    img.set_paintable(Some(&tex));
                    return img;
                }
            }
        }

        // 3. Generic browser SVG
        let p_def = constants::icon_path("app_browser.svg");
        if p_def.exists() {
            if let Some(tex) = svg::svg_texture(&p_def, 26) {
                img.set_paintable(Some(&tex));
                return img;
            }
        }

        img
    }

    fn create_browser_row(
        icon_hint: &str,
        name: &str,
        subtitle: &str,
        status_text: &str,
    ) -> gtk4::Box {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.set_margin_top(4);
        row.set_margin_bottom(4);
        row.add_css_class("assistant-browser-row");

        let icon_lbl = Self::resolve_browser_icon(icon_hint, name);
        row.append(&icon_lbl);

        let text_vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let name_lbl = gtk4::Label::new(Some(if name.is_empty() { "Browser" } else { name }));
        name_lbl.add_css_class("assistant-browser-name");
        text_vbox.append(&name_lbl);
        if !subtitle.is_empty() {
            let sub_lbl = gtk4::Label::new(Some(subtitle));
            sub_lbl.add_css_class("assistant-browser-sub");
            text_vbox.append(&sub_lbl);
        }
        row.append(&text_vbox);

        let status_lbl = gtk4::Label::new(Some(status_text));
        status_lbl.add_css_class("assistant-browser-status");
        status_lbl.set_halign(gtk4::Align::End);
        status_lbl.set_hexpand(true);
        row.append(&status_lbl);

        row
    }

    fn on_legacy_assistant_toggled(is_active: bool) {
        let home_bin = dirs_home().join(".local/bin/adguard-cli");
        let bin_path = if home_bin.exists() {
            home_bin.to_string_lossy().to_string()
        } else {
            "adguard-cli".to_string()
        };
        let cmd = if is_active { "enable" } else { "disable" };
        let _ = std::process::Command::new(&bin_path)
            .args(["userscripts", cmd, "adguard-assistant"])
            .output();
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.root
    }

    pub fn present(&self) {
        self.root.present();
    }
}

fn dirs_home() -> std::path::PathBuf {
    constants::home_dir()
}
