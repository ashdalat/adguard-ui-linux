// Port of adguard_modern/tabs/settings_https_filtering_view.py — the HTTPS
// filtering subview (certificate section + allowlist / EV rows).
use gtk4::prelude::*;
use regex::RegexBuilder;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::backend;
use crate::constants as c;
use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::toggle_switch::ToggleSwitch;

fn yaml_paths() -> Vec<PathBuf> {
    // Python order: root first, then the user directory.
    vec![
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
        c::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
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

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    lbl_sub: gtk4::Label,
    lbl_sec_t: gtk4::Label,
    lbl_cert_valid: gtk4::Label,
    btn_reinstall: gtk4::Button,
    btn_download: gtk4::Button,
    lbl_allowlist_t: gtk4::Label,
    lbl_allowlist_d: gtk4::Label,
    lbl_ev_t: gtk4::Label,
    lbl_ev_d: gtk4::Label,
    raw_cert_date: RefCell<String>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    allowlist_cb: RefCell<Option<Box<dyn Fn()>>>,
    ev_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct SettingsHttpsFilteringView {
    inner: Rc<Inner>,
}

impl SettingsHttpsFilteringView {
    pub fn new() -> SettingsHttpsFilteringView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(20);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top_bar.append(&spacer);
        root.append(&top_bar);
        root.append(&spacer_px(22));

        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("HTTPS filtering"));
        lbl_title.add_css_class("page-title");
        header_row.append(&lbl_title);
        let h_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        h_spacer.set_hexpand(true);
        header_row.append(&h_spacer);

        let is_https = get_yaml_val(r"https_filtering:\s*\n\s*enabled:\s*(true|false)", "false").to_lowercase() == "true";
        let switch_master = ToggleSwitch::new(46, 26);
        switch_master.set_checked(is_https, false);
        header_row.append(switch_master.root());
        root.append(&header_row);

        let lbl_sub = gtk4::Label::new(Some("Filters traffic sent over HTTPS"));
        lbl_sub.add_css_class("lbl-sub-11");
        lbl_sub.set_halign(gtk4::Align::Start);
        root.append(&lbl_sub);
        root.append(&spacer_px(26));

        let lbl_sec_t = gtk4::Label::new(Some("Security certificate"));
        lbl_sec_t.add_css_class("lbl-title");
        lbl_sec_t.set_halign(gtk4::Align::Start);
        root.append(&lbl_sec_t);
        root.append(&spacer_px(4));

        let raw_cert_date = "Tuesday, July 31, 2046".to_string();
        let lbl_cert_valid = gtk4::Label::new(Some(&format!("Valid until {}", raw_cert_date)));
        lbl_cert_valid.add_css_class("lbl-10");
        lbl_cert_valid.set_halign(gtk4::Align::Start);
        root.append(&lbl_cert_valid);
        root.append(&spacer_px(14));

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let btn_reinstall = outline_button("Reinstall certificate");
        let btn_download = outline_button("Download certificate");
        btn_row.append(&btn_reinstall);
        btn_row.append(&btn_download);
        let row_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        row_spacer.set_hexpand(true);
        btn_row.append(&row_spacer);
        root.append(&btn_row);
        root.append(&spacer_px(28));

        // Website allowlist row
        let switch_allowlist = ToggleSwitch::new(46, 26);
        switch_allowlist.set_checked(true, false);
        let (row_allowlist, lbl_allowlist_t, lbl_allowlist_d) =
            create_toggle_item("Website allowlist", "", &switch_allowlist, true);
        root.append(&row_allowlist);
        root.append(&spacer_px(16));

        // EV certificates row
        let is_ev = get_yaml_val(r"filter_ev_certificates:\s*(true|false)", "false").to_lowercase() == "true";
        let switch_ev = ToggleSwitch::new(46, 26);
        switch_ev.set_checked(is_ev, false);
        let (row_ev, lbl_ev_t, lbl_ev_d) = create_toggle_item(
            "Filter websites with EV certificates",
            "Filters websites with Extended Validation security certificates",
            &switch_ev,
            true,
        );
        root.append(&row_ev);

        let bb_root = btn_back.root.clone();
        let bri = btn_reinstall.clone();
        let bdl = btn_download.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            lbl_sub,
            lbl_sec_t,
            lbl_cert_valid,
            btn_reinstall,
            btn_download,
            lbl_allowlist_t,
            lbl_allowlist_d,
            lbl_ev_t,
            lbl_ev_d,
            raw_cert_date: RefCell::new(raw_cert_date),
            back_cb: RefCell::new(None),
            allowlist_cb: RefCell::new(None),
            ev_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });
        let i3 = inner.clone();
        row_allowlist.add_controller(click_gesture(&i3, "allowlist"));
        let i4 = inner.clone();
        row_ev.add_controller(click_gesture(&i4, "ev"));

        switch_master.connect_toggled(Box::new(move |checked| {
            let val_str = if checked { "true" } else { "false" };
            set_yaml_key(r"(https_filtering:\s*\n\s*enabled:\s*)(true|false)", val_str);
        }));
        switch_ev.connect_toggled(Box::new(move |checked| {
            let val_str = if checked { "true" } else { "false" };
            set_yaml_key(r"(filter_ev_certificates:\s*)(true|false)", val_str);
        }));

        let i7 = inner.clone();
        bri.connect_clicked(move |_| reinstall_cert(&i7));
        bdl.connect_clicked(move |_| download_cert());

        let i9 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i9)));

        load_real_cert_date(&inner);
        retranslate_ui(&inner);

        SettingsHttpsFilteringView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_allowlist_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.allowlist_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_open_ev_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.ev_cb.borrow_mut() = Some(cb);
    }
}

fn click_gesture(inner: &Rc<Inner>, which: &str) -> gtk4::GestureClick {
    let gesture = gtk4::GestureClick::new();
    let w = which.to_string();
    let i = inner.clone();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        if n != 1 {
            return;
        }
        if w == "allowlist" {
            if let Some(f) = i.allowlist_cb.borrow().as_ref() {
                f();
            }
        } else if let Some(f) = i.ev_cb.borrow().as_ref() {
            f();
        }
    });
    gesture
}

fn create_toggle_item(
    title: &str,
    desc: &str,
    switch: &ToggleSwitch,
    clickable: bool,
) -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
    box_.add_css_class("toggle-item");
    if clickable {
        box_.add_css_class("toggle-item-hover");
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
    box_.append(&col);
    box_.append(switch.root());
    (box_, lbl_t, lbl_d)
}

fn outline_button(text: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.set_label(text);
    btn.add_css_class("outline-btn-10");
    btn.set_size_request(-1, 38);
    btn
}

fn spacer_px(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn count_exclusions() -> i64 {
    let bak_p = c::home_dir().join(".local/share/adguard-cli/https_exclusions.txt.bak");
    let txt_p = c::home_dir().join(".local/share/adguard-cli/https_exclusions.txt");
    let p = if bak_p.exists() { bak_p } else { txt_p };
    if p.exists() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            return content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .count() as i64;
        }
    }
    8537
}

fn load_real_cert_date(inner: &Rc<Inner>) {
    let cert_p = c::home_dir().join(".local/share/adguard-cli/AdGuard CLI CA.pem");
    if cert_p.exists() {
        if let Ok(out) = std::process::Command::new("openssl")
            .args(["x509", "-enddate", "-noout", "-in"])
            .arg(&cert_p)
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if stdout.contains("notAfter=") {
                let raw = stdout.replace("notAfter=", "").trim().to_string();
                *inner.raw_cert_date.borrow_mut() = raw.clone();
                inner.lbl_cert_valid.set_text(&i18n::tr("https_cert_valid_until").replace("{date}", &raw));
            }
        }
    }
}

fn retranslate_ui(inner: &RcInner) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("https_filtering_title"));
    inner.lbl_sub.set_text(&i18n::tr("https_filtering_desc"));
    inner.lbl_sec_t.set_text(&i18n::tr("https_sec_cert"));
    let raw_d = inner.raw_cert_date.borrow().clone();
    inner.lbl_cert_valid.set_text(&i18n::tr("https_cert_valid_until").replace("{date}", &raw_d));
    inner.btn_reinstall.set_label(&i18n::tr("https_btn_reinstall"));
    inner.btn_download.set_label(&i18n::tr("https_btn_download"));
    inner.lbl_allowlist_t.set_text(&i18n::tr("https_allowlist_t"));
    let count = count_exclusions();
    inner.lbl_allowlist_d.set_text(&i18n::tr("https_allowlist_d").replace("{count}", &count.to_string()));
    inner.lbl_ev_t.set_text(&i18n::tr("https_ev_t"));
    inner.lbl_ev_d.set_text(&i18n::tr("https_ev_d"));
}

type RcInner = Rc<Inner>;

fn reinstall_cert(inner: &RcInner) {
    let dlg = crate::util::sub_window("Reinstall Certificate");
    dlg.set_default_size(460, 260);
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    dlg.set_child(Some(&col));
    let l = gtk4::Label::new(Some("How would you like to reinstall the AdGuard Root Certificate?"));
    l.set_wrap(true);
    l.set_halign(gtk4::Align::Start);
    col.append(&l);
    let info = gtk4::Label::new(Some(
        "• Install to System & Firefox: Opens a terminal so you can enter your sudo password and install the CA certificate into the system trust store and Firefox.\n\n\
         • Regenerate Cert Only: Creates a fresh certificate key and PEM file locally in ~/.local/share/adguard-cli/ without touching system stores.",
    ));
    info.set_wrap(true);
    info.add_css_class("c-muted");
    info.set_halign(gtk4::Align::Start);
    col.append(&info);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let target = inner.clone();
    for (label, action) in [
        ("Install to System", 0i32),
        ("Regenerate Local Cert", 1i32),
        ("Cancel", 2i32),
    ] {
        let b = gtk4::Button::new();
        b.set_label(label);
        b.add_css_class(if action == 2 { "outline-btn" } else { "green-btn" });
        let t = target.clone();
        let d = dlg.clone();
        b.connect_clicked(move |_| {
            d.set_visible(false);
            match action {
                0 => {
                    let res = backend::install_certificate();
                    load_real_cert_date(&t);
                    if res.0 {
                        show_message(None, "Success", "Installed HTTPS Certificate system-wide successfully!\nConfigured for System Trust, Brave, Chrome, Firefox, and Mullvad Browser.");
                    } else {
                        show_message(None, "Warning", &format!("Certificate installation finished with warnings:\n{}", if res.1.is_empty() { "" } else { &res.1 }));
                    }
                }
                1 => {
                    let bin = if c::home_dir().join(".local/bin/adguard-cli").exists() {
                        c::home_dir().join(".local/bin/adguard-cli").to_string_lossy().to_string()
                    } else {
                        "adguard-cli".to_string()
                    };
                    let out = run_with_input(&bin, &["cert"], "no\n");
                    load_real_cert_date(&t);
                    match out {
                        Ok(_) => show_message(
                            None,
                            "Success",
                            &format!(
                                "AdGuard Root Certificate regenerated successfully!\n\nPath: {}",
                                c::home_dir().join(".local/share/adguard-cli/AdGuard CLI CA.pem").display()
                            ),
                        ),
                        Err(e) => show_message(None, "Error", &e),
                    }
                }
                _ => {}
            }
        });
        row.append(&b);
    }
    col.append(&row);
    dlg.present();
}

fn run_with_input(bin: &str, args: &[&str], input: &str) -> Result<(), String> {
    use std::io::Write;
    let mut child = std::process::Command::new(bin)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let _ = child.wait_with_output();
    Ok(())
}

fn download_cert() {
    let cert_p = c::home_dir().join(".local/share/adguard-cli/AdGuard CLI CA.pem");
    if !cert_p.exists() {
        show_message(None, "Download Certificate", "Certificate file not found.");
        return;
    }
    let dlg = gtk4::FileChooserDialog::new(
        Some("Save AdGuard Root Certificate"),
        None::<&gtk4::Window>,
        gtk4::FileChooserAction::Save,
        &[
            (&i18n::tr("btn_cancel"), gtk4::ResponseType::Cancel),
            ("Save", gtk4::ResponseType::Accept),
        ],
    );
    dlg.set_current_name("AdGuard_Root_CA.pem");
    let source = cert_p.clone();
    dlg.connect_response(move |d, response| {
        if response != gtk4::ResponseType::Accept {
            return;
        }
        let path = match d.file().and_then(|f| f.path()) {
            Some(p) => p,
            None => return,
        };
        match std::fs::copy(&source, &path) {
            Ok(_) => show_message(None, "Success", &format!("Certificate saved to:\n{}", path.display())),
            Err(e) => show_message(None, "Error", &e.to_string()),
        }
    });
    dlg.present();
}
