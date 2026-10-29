// Port of adguard_modern/tabs/settings_general_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::common_controls::{no_wheel_combo, BackButton};
use crate::widgets::filter_editor_window::FilterEditorWindow;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;
use regex::RegexBuilder;

fn yaml_paths() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    vec![
        std::path::Path::new(&home)
            .join(".local/share/adguard-cli/proxy.yaml")
            .to_string_lossy()
            .to_string(),
        "/root/.local/share/adguard-cli/proxy.yaml".to_string(),
    ]
}

fn cli_bin() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let home_bin = std::path::Path::new(&home).join(".local/bin/adguard-cli");
    if std::path::Path::exists(&home_bin) {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    }
}

fn get_yaml_val(pattern: &str, default: &str) -> String {
    for p in yaml_paths() {
        if !std::path::Path::exists(&std::path::PathBuf::from(&p)) {
            continue;
        }
        if let Ok(c) = std::fs::read_to_string(&p) {
            if let Ok(re) = RegexBuilder::new(pattern).case_insensitive(true).build() {
                if let Some(m) = re.captures(&c) {
                    return m[1].to_lowercase();
                }
            }
        }
    }
    default.to_string()
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    switch_startup: ToggleSwitch,
    lbl_startup: gtk4::Label,
    switch_telemetry: ToggleSwitch,
    lbl_telemetry: gtk4::Label,
    combo_lang: gtk4::DropDown,
    lbl_lang_title: gtk4::Label,
    combo_theme: gtk4::DropDown,
    lbl_theme_title: gtk4::Label,
    combo_channel: gtk4::DropDown,
    lbl_channel_title: gtk4::Label,
    combo_log: gtk4::DropDown,
    lbl_log_title: gtk4::Label,
    lbl_fe_title: gtk4::Label,
    lbl_fe_desc: gtk4::Label,
    lbl_adv_title: gtk4::Label,
    lbl_adv_desc: gtk4::Label,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_network_cb: RefCell<Option<Box<dyn Fn()>>>,
    open_advanced_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsGeneralView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn string_list(items: &[&str]) -> gtk4::StringList {
    let sl = gtk4::StringList::new(&[]);
    for s in items {
        sl.append(s);
    }
    sl
}

impl SettingsGeneralView {
    pub fn new() -> SettingsGeneralView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: back + more menu.
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

        let lbl_title = gtk4::Label::new(Some("App settings"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(26));

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 22);
        container.add_css_class("transparent-container");
        container.set_margin_end(12);
        container.set_margin_bottom(24);

        // Toggle 1: autostart.
        let is_startup = is_startup_enabled();
        let switch_startup = ToggleSwitch::new(46, 26);
        switch_startup.set_checked(is_startup, false);
        let (row_startup, lbl_startup) = create_toggle_row("Launch AdGuard at system startup", &switch_startup);
        container.append(&row_startup);

        // Toggle 2: usage data.
        let is_crash = is_crash_reports_enabled();
        let switch_telemetry = ToggleSwitch::new(46, 26);
        switch_telemetry.set_checked(is_crash, false);
        let (row_telemetry, lbl_telemetry) = create_usage_toggle_row(&switch_telemetry);
        container.append(&row_telemetry);
        container.append(&spacer(4));

        // Dropdowns.
        let curr_lang = i18n::get_current_language();
        let lang_idx = if curr_lang == i18n::LANG_EN { 0 } else { 1 };
        let (row_lang, combo_lang, lbl_lang_title) =
            create_dropdown_section("Language", &["English", "Tiếng Việt"], lang_idx);
        container.append(&row_lang);

        let (row_theme, combo_theme, lbl_theme_title) =
            create_dropdown_section("Theme", &["Dark (Default)"], 0);
        container.append(&row_theme);

        let curr_channel = get_yaml_val(r#"update_channel:\s*['"]?([a-zA-Z0-9_-]+)['"]?"#, "release");
        let mut ch_idx = 0usize;
        if curr_channel == "beta" {
            ch_idx = 1;
        } else if curr_channel == "nightly" {
            ch_idx = 2;
        }
        let (row_channel, combo_channel, lbl_channel_title) = create_dropdown_section(
            "App update channel",
            &["Release (the most stable)", "Beta", "Nightly"],
            ch_idx,
        );
        container.append(&row_channel);

        let curr_log = get_yaml_val(r#"log_level:\s*['"]?([a-zA-Z0-9_-]+)['"]?"#, "info");
        let mut log_idx = 0usize;
        if curr_log == "debug" {
            log_idx = 1;
        } else if curr_log == "trace" {
            log_idx = 2;
        }
        let (row_log, combo_log, lbl_log_title) =
            create_dropdown_section("Logging level", &["Info", "Debug", "Trace"], log_idx);
        container.append(&row_log);
        container.append(&spacer(8));

        // Navigation rows.
        let (row_fe, lbl_fe_title, lbl_fe_desc) =
            create_nav_row("Filter editor", "Manage filters and user rules");
        container.append(&row_fe);

        let (row_adv, lbl_adv_title, lbl_adv_desc) =
            create_nav_row("Advanced", "For expert users only");
        container.append(&row_adv);

        scroll.set_child(Some(&container));
        root.append(&scroll);

        let bb_root = btn_back.root.clone();
        let bm = btn_more.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            switch_startup,
            lbl_startup,
            switch_telemetry,
            lbl_telemetry,
            combo_lang,
            lbl_lang_title,
            combo_theme,
            lbl_theme_title,
            combo_channel,
            lbl_channel_title,
            combo_log,
            lbl_log_title,
            lbl_fe_title,
            lbl_fe_desc,
            lbl_adv_title,
            lbl_adv_desc,
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            open_network_cb: RefCell::new(None),
            open_advanced_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        let bm2 = bm.clone();
        bm.connect_clicked(move |_| show_more_menu(&i2, &bm2));

        let i3 = inner.clone();
        row_fe.add_controller(click_gesture(&i3, "filter_editor"));
        let i4 = inner.clone();
        row_adv.add_controller(click_gesture(&i4, "advanced"));

        inner.lbl_telemetry.connect_activate_link(move |_l: &gtk4::Label, uri: &str| {
            let _ = std::process::Command::new("xdg-open").arg(uri).spawn();
            gtk4::glib::Propagation::Stop
        });

        inner.switch_startup.connect_toggled(Box::new(move |checked| on_startup_toggled(checked)));
        inner.switch_telemetry.connect_toggled(Box::new(move |checked| on_crash_reports_toggled(checked)));

        let i8 = inner.clone();
        inner.combo_lang.connect_selected_notify(move |c| {
            if *i8.syncing.borrow() {
                return;
            }
            on_language_changed(c.selected() as usize);
        });
        let i9 = inner.clone();
        inner.combo_channel.connect_selected_notify(move |c| {
            if *i9.syncing.borrow() {
                return;
            }
            on_update_channel_changed(c.selected() as usize);
        });
        let i10 = inner.clone();
        inner.combo_log.connect_selected_notify(move |c| {
            if *i10.syncing.borrow() {
                return;
            }
            on_log_level_changed(c.selected() as usize);
        });

        let i11 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i11)));

        retranslate_ui(&inner);

        SettingsGeneralView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_network_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_network_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_advanced_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.open_advanced_cb.borrow_mut() = Some(cb);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn click_gesture(inner: &Rc<Inner>, which: &str) -> gtk4::GestureClick {
    let i = inner.clone();
    let which = which.to_string();
    let gesture = gtk4::GestureClick::new();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        if n != 1 {
            return;
        }
        if which == "filter_editor" {
            open_filter_editor(&i);
        } else {
            if let Some(cb) = i.open_advanced_cb.borrow().as_ref() {
                cb();
            }
        }
    });
    gesture
}

fn create_toggle_row(title: &str, switch_widget: &ToggleSwitch) -> (gtk4::Box, gtk4::Label) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    box_.add_css_class("transparent-container");

    let lbl = gtk4::Label::new(Some(title));
    lbl.add_css_class("lbl-10-bold-white");
    lbl.set_hexpand(true);
    box_.append(&lbl);

    switch_widget.root().set_valign(gtk4::Align::Center);
    box_.append(switch_widget.root());

    (box_, lbl)
}

fn create_usage_toggle_row(switch_widget: &ToggleSwitch) -> (gtk4::Box, gtk4::Label) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    box_.add_css_class("transparent-container");

    let lbl = gtk4::Label::new(Some(
        "Send anonymized <span color=\"#5EBB6A\"><a href=\"https://adguard.com/privacy.html\">app usage data</a></span>",
    ));
    lbl.add_css_class("lbl-10-bold-white");
    lbl.set_use_markup(true);
    lbl.set_hexpand(true);
    box_.append(&lbl);

    switch_widget.root().set_valign(gtk4::Align::Center);
    box_.append(switch_widget.root());

    (box_, lbl)
}

fn create_dropdown_section(
    label_text: &str,
    options: &[&str],
    default_index: usize,
) -> (gtk4::Box, gtk4::DropDown, gtk4::Label) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    box_.add_css_class("transparent-container");

    let lbl = gtk4::Label::new(Some(label_text));
    lbl.add_css_class("lbl-10-white");
    lbl.set_halign(gtk4::Align::Start);
    box_.append(&lbl);

    let combo = no_wheel_combo();
    combo.set_model(Some(&string_list(options)));
    combo.set_selected(default_index as u32);
    combo.set_size_request(-1, 44);
    combo.add_css_class("combo-general");
    box_.append(&combo);

    (box_, combo, lbl)
}

fn create_nav_row(title: &str, description: &str) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
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

    let lbl_d = gtk4::Label::new(Some(description));
    lbl_d.add_css_class("lbl-desc-9");
    lbl_d.set_halign(gtk4::Align::Start);
    v.append(&lbl_d);

    row.append(&v);

    let chev = svg::svg_image("chevron_right.svg", 14);
    chev.set_valign(gtk4::Align::Center);
    row.append(&chev);

    (row, lbl_t, lbl_d)
}

fn set_cli_config(key: &str, val: &str) {
    let bin = cli_bin();
    let _ = std::process::Command::new(&bin)
        .args(["config", "set", key, val])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::process::Command::new("sudo")
        .args(["-n", &bin, "config", "set", key, val])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Also write directly to both yaml files to ensure immediate sync.
    let formatted_val = if val.to_lowercase() == "true" || val.to_lowercase() == "false" {
        val.to_lowercase()
    } else {
        format!("'{}'", val)
    };

    for p in yaml_paths() {
        if !std::path::Path::exists(&std::path::PathBuf::from(&p)) {
            continue;
        }
        if let Ok(mut c) = std::fs::read_to_string(&p) {
            let re = match RegexBuilder::new(&format!(r#"^[ \t]*{}:[ \t]*.*$"#, regex::escape(key)))
                .multi_line(true)
                .build()
            {
                Ok(re) => re,
                Err(_) => continue,
            };
            if re.is_match(&c) {
                c = re.replace_all(&c, format!("{}: {}", key, formatted_val)).to_string();
            } else {
                c.push_str(&format!("\n{}: {}\n", key, formatted_val));
            }
            let _ = std::fs::write(&p, c);
        }
    }
}

fn on_language_changed(idx: usize) {
    let lang = if idx == 0 { i18n::LANG_EN } else { i18n::LANG_VI };
    i18n::set_language(lang);
}

fn on_update_channel_changed(index: usize) {
    let channels = ["release", "beta", "nightly"];
    let val = channels.get(index).copied().unwrap_or("release");
    set_cli_config("update_channel", val);
}

fn on_log_level_changed(index: usize) {
    let levels = ["info", "debug", "trace"];
    let val = levels.get(index).copied().unwrap_or("info");
    set_cli_config("log_level", val);
}

fn open_filter_editor(_inner: &Rc<Inner>) {
    // Deviation: QDialog.exec() -> non-modal GTK window.
    let dlg = FilterEditorWindow::new();
    dlg.root().present();
}

fn show_more_menu(inner: &Rc<Inner>, anchor: &gtk4::Button) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("menu-popover");
    popover.set_parent(anchor);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_reset = gtk4::Button::new();
    item_reset.set_label(&i18n::tr("menu_reset_settings"));
    item_reset.add_css_class("menu-item");
    let i1 = inner.clone();
    let p1 = popover.clone();
    item_reset.connect_clicked(move |_| {
        p1.hide();
        reset_defaults(&i1);
    });
    menu_box.append(&item_reset);

    let sep = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep.add_css_class("menu-separator");
    menu_box.append(&sep);

    let item_export = gtk4::Button::new();
    item_export.set_label(&i18n::tr("menu_export_settings"));
    item_export.add_css_class("menu-item");
    let p2 = popover.clone();
    item_export.connect_clicked(move |_| {
        p2.hide();
        export_settings();
    });
    menu_box.append(&item_export);

    let item_import = gtk4::Button::new();
    item_import.set_label(&i18n::tr("menu_import_settings"));
    item_import.add_css_class("menu-item");
    let p3 = popover.clone();
    item_import.connect_clicked(move |_| {
        p3.hide();
        import_settings();
    });
    menu_box.append(&item_import);

    popover.set_child(Some(&menu_box));
    popover.show();
}

fn is_startup_enabled() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let autostart_p = std::path::Path::new(&home).join(".config/autostart/adguard-gui.desktop");
    if std::path::Path::exists(&autostart_p) {
        if let Ok(content) = std::fs::read_to_string(&autostart_p) {
            if content.contains("X-GNOME-Autostart-enabled=false") {
                return false;
            }
            return true;
        }
    }
    if let Ok(out) = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "adguard.service"])
        .output()
    {
        return String::from_utf8_lossy(&out.stdout).contains("enabled");
    }
    false
}

fn on_startup_toggled(checked: bool) {
    let home = std::env::var("HOME").unwrap_or_default();
    let autostart_p = std::path::Path::new(&home).join(".config/autostart/adguard-gui.desktop");
    let legacy_p = std::path::Path::new(&home).join(".config/autostart/adguard_gui.desktop");
    if std::path::Path::exists(&legacy_p) {
        let _ = std::fs::remove_file(&legacy_p);
    }

    let val_str = if checked { "true" } else { "false" };
    if std::path::Path::exists(&autostart_p) {
        if let Ok(mut content) = std::fs::read_to_string(&autostart_p) {
            if content.contains("X-GNOME-Autostart-enabled=") {
                let re = RegexBuilder::new(r"X-GNOME-Autostart-enabled=(true|false)")
                    .case_insensitive(true)
                    .build()
                    .unwrap();
                content = re
                    .replace_all(&content, |_caps: &regex::Captures| {
                        format!("X-GNOME-Autostart-enabled={}", val_str)
                    })
                    .to_string();
            } else {
                content.push_str(&format!("\nX-GNOME-Autostart-enabled={}\n", val_str));
            }
            let _ = std::fs::write(&autostart_p, content);
        }
    } else {
        let dir = autostart_p.parent();
        if let Some(d) = dir {
            let _ = std::fs::create_dir_all(d);
        }
        let bin_path = std::path::Path::new(&home).join(".local/bin/adguard-gui");
        let icon_path = std::path::Path::new(&home)
            .join(".local/share/adguard_modern_gui/adguard_blocker_icon.png");
        let _ = std::fs::write(
            &autostart_p,
            format!(
                "[Desktop Entry]\nName=AdGuard (Autostart)\nComment=AdGuard AdBlocker Modern GUI\nExec={} --autostart\nIcon={}\nTerminal=false\nType=Application\nCategories=Network;Security;\nX-GNOME-Autostart-enabled={}\n",
                bin_path.display(),
                icon_path.display(),
                val_str
            ),
        );
    }

    let action = if checked { "enable" } else { "disable" };
    let _ = std::process::Command::new("systemctl")
        .args(["--user", action, "adguard.service"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn is_crash_reports_enabled() -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    let proxy_yaml = std::path::Path::new(&home).join(".local/share/adguard-cli/proxy.yaml");
    if std::path::Path::exists(&proxy_yaml) {
        if let Ok(c) = std::fs::read_to_string(&proxy_yaml) {
            if let Ok(re) = RegexBuilder::new(r"send_crash_reports:\s*(true|false)")
                .case_insensitive(true)
                .build()
            {
                if let Some(m) = re.captures(&c) {
                    return m[1].to_lowercase() == "true";
                }
            }
        }
    }
    false
}

fn on_crash_reports_toggled(checked: bool) {
    let val_str = if checked { "true" } else { "false" };
    set_cli_config("send_crash_reports", val_str);
}

fn reset_defaults(inner: &Rc<Inner>) {
    // QMessageBox.question -> GTK window with Yes/No buttons.
    let dlg = crate::util::sub_window("Reset Settings");
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    box_.add_css_class("dlg-dark");
    box_.set_margin_top(24);
    box_.set_margin_bottom(20);
    box_.set_margin_start(24);
    box_.set_margin_end(24);

    let lbl = gtk4::Label::new(Some("Are you sure you want to reset all app settings to defaults?"));
    lbl.add_css_class("dlg-title");
    lbl.set_wrap(true);
    lbl.set_xalign(0.0);
    box_.append(&lbl);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::End);

    let btn_no = gtk4::Button::new();
    btn_no.set_label("No");
    btn_no.add_css_class("outline-btn");
    let d1 = dlg.clone();
    btn_no.connect_clicked(move |_| d1.close());
    row.append(&btn_no);

    let btn_yes = gtk4::Button::new();
    btn_yes.set_label("Yes");
    btn_yes.add_css_class("green-btn");
    let i1 = inner.clone();
    let d2 = dlg.clone();
    btn_yes.connect_clicked(move |_| {
        d2.close();
        // Python sets the switches (firing the toggled handlers).
        i1.switch_startup.set_checked(true, true);
        i1.switch_telemetry.set_checked(false, true);
    });
    row.append(&btn_yes);

    box_.append(&row);
    dlg.set_child(Some(&box_));
    dlg.present();
}

fn export_settings() {
    let bin = cli_bin();
    let dlg = gtk4::FileChooserDialog::new(
        Some("Export AdGuard Settings"),
        None::<&gtk4::Window>,
        gtk4::FileChooserAction::Save,
        &[
            (&i18n::tr("btn_cancel"), gtk4::ResponseType::Cancel),
            ("Save", gtk4::ResponseType::Accept),
        ],
    );
    dlg.set_current_name("adguard-settings.zip");
    let b = bin.clone();
    dlg.connect_response(move |d, response| {
        if response != gtk4::ResponseType::Accept {
            return;
        }
        let path = match d.file().and_then(|f| f.path()) {
            Some(p) => p,
            None => return,
        };
        match std::process::Command::new(&b)
            .args(["export-settings", "-o"]).arg(&path)
            .output()
        {
            Ok(res) => {
                if res.status.code() == Some(0) {
                    show_message(
                        None,
                        "Export Settings",
                        &format!("Settings exported successfully to:\n{}", path.display()),
                    );
                } else {
                    let err = String::from_utf8_lossy(&res.stderr).trim().to_string();
                    let out = String::from_utf8_lossy(&res.stdout).trim().to_string();
                    show_message(
                        None,
                        "Error",
                        &format!("Failed to export settings:\n{}", if err.is_empty() { out } else { err }),
                    );
                }
            }
            Err(e) => show_message(None, "Error", &e.to_string()),
        }
    });
    dlg.present();
}

fn import_settings() {
    let bin = cli_bin();

    let dlg = gtk4::FileChooserDialog::new(
        Some("Import AdGuard Settings"),
        None::<&gtk4::Window>,
        gtk4::FileChooserAction::Open,
        &[
            (&i18n::tr("btn_cancel"), gtk4::ResponseType::Cancel),
            ("Open", gtk4::ResponseType::Accept),
        ],
    );
    let b = bin.clone();
    dlg.connect_response(move |d, response| {
        if response != gtk4::ResponseType::Accept {
            return;
        }
        let path = match d.file().and_then(|f| f.path()) {
            Some(p) => p,
            None => return,
        };
        match std::process::Command::new(&b)
            .args(["import-settings"]).arg(&path)
            .output()
        {
            Ok(res) => {
                if res.status.code() == Some(0) {
                    show_message(None, "Import Settings", "Settings imported successfully!");
                } else {
                    let err = String::from_utf8_lossy(&res.stderr).trim().to_string();
                    let out = String::from_utf8_lossy(&res.stdout).trim().to_string();
                    show_message(
                        None,
                        "Error",
                        &format!("Failed to import settings:\n{}", if err.is_empty() { out } else { err }),
                    );
                }
            }
            Err(e) => show_message(None, "Error", &e.to_string()),
        }
    });
    dlg.present();
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("settings_app_title"));
    inner.lbl_startup.set_text(&i18n::tr("launch_at_system_startup"));
    inner.lbl_telemetry.set_markup(&crate::tabs::home_network_section::to_pango_link_markup(
        &i18n::tr("send_usage_data"),
        crate::constants::COLOR_TEXT_MUTED,
    ));
    inner.lbl_lang_title.set_text(&i18n::tr("setting_language"));
    inner.lbl_theme_title.set_text(&i18n::tr("setting_theme"));

    // Python setItemText(): rebuild each StringList with translations, keep index.
    *inner.syncing.borrow_mut() = true;
    let theme_idx = inner.combo_theme.selected() as usize;
    inner
        .combo_theme
        .set_model(Some(&string_list(&[&i18n::tr("theme_dark_default")])));
    inner.combo_theme.set_selected(theme_idx as u32);

    let ch_idx = inner.combo_channel.selected() as usize;
    inner.combo_channel.set_model(Some(&string_list(&[
        &i18n::tr("channel_release"),
        &i18n::tr("channel_beta"),
        &i18n::tr("channel_nightly"),
    ])));
    inner.combo_channel.set_selected(ch_idx as u32);

    let log_idx = inner.combo_log.selected() as usize;
    inner.combo_log.set_model(Some(&string_list(&[
        &i18n::tr("log_info"),
        &i18n::tr("log_debug"),
        &i18n::tr("log_trace"),
    ])));
    inner.combo_log.set_selected(log_idx as u32);

    inner.lbl_fe_title.set_text(&i18n::tr("setting_filter_editor"));
    inner.lbl_fe_desc.set_text(&i18n::tr("setting_filter_editor_desc"));
    inner.lbl_adv_title.set_text(&i18n::tr("nav_advanced_title"));
    inner.lbl_adv_desc.set_text(&i18n::tr("nav_advanced_desc"));

    // Python only syncs the language combo index here (its items are not
    // translated), without firing the handler.
    let lang_idx = if i18n::get_current_language() == i18n::LANG_EN { 0 } else { 1 };
    inner.combo_lang.set_selected(lang_idx as u32);
    *inner.syncing.borrow_mut() = false;
}
