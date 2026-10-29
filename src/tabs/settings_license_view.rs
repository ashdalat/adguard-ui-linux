// Port of adguard_modern/tabs/settings_license_view.py
use std::cell::RefCell;
use std::rc::Rc;
use std::os::fd::FromRawFd;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use gtk4::prelude::*;
use regex::{Regex, RegexBuilder};

use crate::constants as c;
use crate::i18n;
use crate::util;
use crate::widgets::common_controls::BackButton;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;

// Stacked pages (Python PAGE_ACTIVE / PAGE_NOT_ACTIVE / PAGE_ACTIVATING).
const PAGE_ACTIVE: &str = "active";
const PAGE_NOT_ACTIVE: &str = "not_active";
const PAGE_ACTIVATING: &str = "activating";

fn cli_bin() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let home_bin = std::path::Path::new(&home).join(".local/bin/adguard-cli");
    if std::path::Path::exists(&home_bin) {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    }
}

fn is_activated() -> bool {
    // Return True if adguard-cli has an active license.
    // Deviation: Python subprocess timeout=5 not enforced.
    if let Ok(res) = std::process::Command::new(cli_bin()).arg("license").output() {
        return res.status.code() == Some(0)
            && String::from_utf8_lossy(&res.stdout).to_lowercase().contains("owner:");
    }
    false
}

fn parse_license(out: &str) -> (String, String, String) {
    let mut owner = String::new();
    let mut key = String::new();
    let mut status = String::new();
    for line in out.lines() {
        let ll = line.to_lowercase();
        if ll.contains("owner:") {
            owner = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if ll.contains("key:") {
            key = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        } else if ll.contains("status:") {
            status = line.splitn(2, ':').nth(1).unwrap_or("").trim().to_string();
        }
    }
    (owner, key, status)
}

// Worker events delivered to the main loop (Python signals url_ready /
// countdown / speed_up_available; finished is the spawn_stream result).
enum Ev {
    Url(String),
    Countdown(u64),
    SpeedUp,
}

// Control messages to the worker thread (Python send_key / cancel).
enum Ctrl {
    Key(String),
    Cancel,
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    stack: gtk4::Stack,
    // Active page
    lbl_status: gtk4::Label,
    lbl_owner: gtk4::Label,
    lbl_code: gtk4::Label,
    // Not-active page
    btn_activate: gtk4::Button,
    // Activating page
    lbl_act_status: gtk4::Label,
    lbl_act_url: gtk4::Label,
    lbl_act_countdown: gtk4::Label,
    btn_open_browser: gtk4::Button,
    btn_speed_up: gtk4::Button,
    btn_cancel_act: gtk4::Button,
    auth_url: RefCell<String>,
    worker_tx: RefCell<Option<Sender<Ctrl>>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsLicenseView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn once(ms: u64, f: impl FnOnce() + 'static) {
    let mut f = Some(f);
    gtk4::glib::timeout_add_local(Duration::from_millis(ms), move || {
        if let Some(f) = f.take() {
            f();
        }
        gtk4::glib::ControlFlow::Break
    });
}

impl SettingsLicenseView {
    pub fn new() -> SettingsLicenseView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(24);
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

        let lbl_title = gtk4::Label::new(Some("License"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(20));

        let stack = gtk4::Stack::new();
        root.append(&stack);

        let (page_active, lbl_status, lbl_owner, lbl_code) = build_active_page();
        stack.add_named(&page_active, Some(PAGE_ACTIVE));

        let (page_not_active, btn_activate) = build_not_active_page();
        stack.add_named(&page_not_active, Some(PAGE_NOT_ACTIVE));

        let (
            page_activating,
            lbl_act_status,
            lbl_act_url,
            lbl_act_countdown,
            btn_open_browser,
            btn_speed_up,
            btn_cancel_act,
        ) = build_activating_page();
        stack.add_named(&page_activating, Some(PAGE_ACTIVATING));

        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            stack,
            lbl_status,
            lbl_owner,
            lbl_code,
            btn_activate,
            lbl_act_status,
            lbl_act_url,
            lbl_act_countdown,
            btn_open_browser,
            btn_speed_up,
            btn_cancel_act,
            auth_url: RefCell::new(String::new()),
            worker_tx: RefCell::new(None),
            back_cb: RefCell::new(None),
        });

        let bb_root = inner.btn_back.root.clone();
        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            // Back: cancel the worker if running, then emit back_requested.
            if let Some(tx) = i1.worker_tx.borrow().as_ref() {
                let _ = tx.send(Ctrl::Cancel);
                *i1.worker_tx.borrow_mut() = None;
            }
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let bm = inner.btn_more.clone();
        let i2 = inner.clone();
        bm.connect_clicked(move |_| show_more_menu(&i2));

        let i3 = inner.clone();
        inner.btn_activate.connect_clicked(move |_| on_activate_clicked(&i3));

        let i4 = inner.clone();
        inner.btn_open_browser.connect_clicked(move |_| {
            send_key(&i4, "b");
            let url = i4.auth_url.borrow().clone();
            if !url.is_empty() {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
        });

        let i5 = inner.clone();
        inner.btn_speed_up.connect_clicked(move |_| {
            send_key(&i5, "s");
            i5.lbl_act_countdown.set_text("Checking…");
            i5.btn_speed_up.set_sensitive(false);
        });

        let i6 = inner.clone();
        inner.btn_cancel_act.connect_clicked(move |_| {
            if let Some(tx) = i6.worker_tx.borrow().as_ref() {
                let _ = tx.send(Ctrl::Cancel);
                *i6.worker_tx.borrow_mut() = None;
            }
            i6.stack.set_visible_child_name(PAGE_NOT_ACTIVE);
        });

        inner.lbl_act_url.connect_activate_link(move |_l: &gtk4::Label, uri: &str| {
            let _ = std::process::Command::new("xdg-open").arg(uri).spawn();
            gtk4::glib::Propagation::Stop
        });

        load_live_license(&inner);

        SettingsLicenseView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn load_live_license(&self) {
        load_live_license(&self.inner);
    }
}

// ------------------------------------------------------------------
// Pages
// ------------------------------------------------------------------
fn build_active_page() -> (gtk4::Box, gtk4::Label, gtk4::Label, gtk4::Label) {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

    let lbl_status = gtk4::Label::new(Some("Status: Active"));
    lbl_status.add_css_class("lbl-11-bold-white");
    lbl_status.set_halign(gtk4::Align::Start);
    lbl_status.set_use_markup(true);
    lbl_status.set_markup(&format!(
        "<span color=\"{}\">Status: Active</span>",
        c::COLOR_ADGUARD_GREEN
    ));
    page.append(&lbl_status);

    let lbl_owner = gtk4::Label::new(Some("Owner: —"));
    lbl_owner.add_css_class("lbl-10-white");
    lbl_owner.set_halign(gtk4::Align::Start);
    page.append(&lbl_owner);

    page.append(&spacer(20));

    // Key row with icon.
    let code_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    let ksvg = svg::svg_image("key_green.svg", 20);
    ksvg.set_valign(gtk4::Align::Center);
    code_row.append(&ksvg);

    let lbl_code = gtk4::Label::new(Some("Activation code: —"));
    lbl_code.add_css_class("lbl-11-bold-white");
    lbl_code.set_valign(gtk4::Align::Center);
    code_row.append(&lbl_code);
    page.append(&code_row);

    (page, lbl_status, lbl_owner, lbl_code)
}

fn build_not_active_page() -> (gtk4::Box, gtk4::Button) {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    card.add_css_class("license-card");
    card.set_margin_top(20);
    card.set_margin_bottom(20);
    card.set_margin_start(24);
    card.set_margin_end(24);

    let lbl_title = gtk4::Label::new(Some("No active license"));
    lbl_title.add_css_class("lbl-13-bold-white");
    lbl_title.set_halign(gtk4::Align::Start);
    card.append(&lbl_title);

    let lbl_hint = gtk4::Label::new(Some(
        "Sign in with your AdGuard account to activate your license.\nAn authorization link will open in your browser.",
    ));
    lbl_hint.add_css_class("lbl-12-muted");
    lbl_hint.set_wrap(true);
    lbl_hint.set_xalign(0.0);
    card.append(&lbl_hint);
    card.append(&spacer(8));

    let btn_activate = gtk4::Button::new();
    btn_activate.set_label("Sign in & Activate");
    btn_activate.add_css_class("license-btn");
    btn_activate.set_size_request(-1, 42);
    card.append(&btn_activate);

    page.append(&card);
    (page, btn_activate)
}

fn build_activating_page() -> (
    gtk4::Box,
    gtk4::Label,
    gtk4::Label,
    gtk4::Label,
    gtk4::Button,
    gtk4::Button,
    gtk4::Button,
) {
    let page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    card.add_css_class("license-card");
    card.set_margin_top(20);
    card.set_margin_bottom(20);
    card.set_margin_start(24);
    card.set_margin_end(24);

    let lbl_act_status = gtk4::Label::new(Some("Waiting for browser authorization…"));
    lbl_act_status.add_css_class("lbl-12-bold-white");
    lbl_act_status.set_wrap(true);
    lbl_act_status.set_xalign(0.0);
    card.append(&lbl_act_status);

    let lbl_act_url = gtk4::Label::new(Some(""));
    lbl_act_url.add_css_class("lbl-12-green");
    lbl_act_url.set_wrap(true);
    lbl_act_url.set_xalign(0.0);
    card.append(&lbl_act_url);

    let lbl_act_countdown = gtk4::Label::new(Some(""));
    lbl_act_countdown.add_css_class("lbl-12-muted");
    lbl_act_countdown.set_xalign(0.0);
    card.append(&lbl_act_countdown);

    card.append(&spacer(6));

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let btn_open_browser = gtk4::Button::new();
    btn_open_browser.set_label("Open browser link");
    btn_open_browser.add_css_class("license-btn-small");
    btn_open_browser.set_size_request(-1, 36);
    btn_open_browser.set_visible(false);
    btn_row.append(&btn_open_browser);

    let btn_speed_up = gtk4::Button::new();
    btn_speed_up.set_label("⚡ Speed up check");
    btn_speed_up.add_css_class("license-btn-dim");
    btn_speed_up.set_size_request(-1, 36);
    btn_speed_up.set_visible(false);
    btn_row.append(&btn_speed_up);

    let btn_cancel_act = gtk4::Button::new();
    btn_cancel_act.set_label("Cancel");
    btn_cancel_act.add_css_class("license-red-btn");
    btn_cancel_act.set_size_request(-1, 36);
    btn_row.append(&btn_cancel_act);

    card.append(&btn_row);
    page.append(&card);

    (
        page,
        lbl_act_status,
        lbl_act_url,
        lbl_act_countdown,
        btn_open_browser,
        btn_speed_up,
        btn_cancel_act,
    )
}

// ------------------------------------------------------------------
// Load / refresh license
// ------------------------------------------------------------------
fn load_live_license(inner: &Rc<Inner>) {
    // Deviation: Python subprocess timeout=5 not enforced.
    let res = std::process::Command::new(cli_bin()).arg("license").output();
    match res {
        Ok(res) if res.status.code() == Some(0) => {
            let out = String::from_utf8_lossy(&res.stdout).trim().to_string();
            if out.is_empty() {
                inner.stack.set_visible_child_name(PAGE_NOT_ACTIVE);
                return;
            }
            let (owner, key, status) = parse_license(&out);

            inner.lbl_owner.set_text(&format!("Owner: {}", if owner.is_empty() { "Not available" } else { &owner }));

            if status.to_lowercase().contains("active") {
                inner.lbl_status.set_markup(&format!(
                    "<span color=\"{}\">Status: Active</span>",
                    c::COLOR_ADGUARD_GREEN
                ));
            } else {
                let st = if status.is_empty() { "Unknown".to_string() } else { status.clone() };
                inner.lbl_status.set_markup(&format!(
                    "<span color=\"#EF5A3B\">Status: {}</span>",
                    st
                ));
            }

            if !key.is_empty() {
                let masked = if key.chars().count() >= 4 {
                    let chars: Vec<char> = key.chars().collect();
                    let head: String = chars[..2].iter().collect();
                    let tail: String = chars[chars.len() - 2..].iter().collect();
                    format!("{}{}{}", head, "*".repeat(chars.len() - 4), tail)
                } else {
                    key.clone()
                };
                inner.lbl_code.set_text(&format!("Activation code: {}", masked));
            } else {
                inner.lbl_code.set_text("Activation code: Not activated");
            }

            inner.stack.set_visible_child_name(PAGE_ACTIVE);
        }
        _ => inner.stack.set_visible_child_name(PAGE_NOT_ACTIVE),
    }
}

// ------------------------------------------------------------------
// Activate flow
// ------------------------------------------------------------------
fn on_activate_clicked(inner: &Rc<Inner>) {
    *inner.auth_url.borrow_mut() = String::new();
    inner.lbl_act_status.set_text("Waiting for browser authorization…");
    inner.lbl_act_url.set_text("");
    inner.lbl_act_countdown.set_text("");
    inner.btn_open_browser.set_visible(false);
    inner.btn_speed_up.set_visible(false);
    inner.stack.set_visible_child_name(PAGE_ACTIVATING);

    start_activate_worker(inner);
}

fn send_key(inner: &Rc<Inner>, ch: &str) {
    if let Some(tx) = inner.worker_tx.borrow().as_ref() {
        let _ = tx.send(Ctrl::Key(ch.to_string()));
    }
}

fn start_activate_worker(inner: &Rc<Inner>) {
    // Spawn `adguard-cli activate` in a pseudo-TTY so the TUI menus work.
    // Deviation: Python signals url_ready/countdown/speed_up_available/finished
    // map to util::spawn_stream items + result; send_key/cancel go through a
    // control channel (adds up to 0.3 s latency vs direct os.write).
    let mut master: libc::c_int = -1;
    let mut slave: libc::c_int = -1;
    if unsafe { libc::openpty(&mut master, &mut slave, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut()) } != 0 {
        on_activate_finished(inner, false);
        return;
    }

    let cli = cli_bin();
    let in_fd = unsafe { libc::dup(slave) };
    let out_fd = unsafe { libc::dup(slave) };
    let err_fd = unsafe { libc::dup(slave) };
    let spawn_res = std::process::Command::new(&cli)
        .arg("activate")
        .stdin(std::process::Stdio::from(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(in_fd)
        }))
        .stdout(std::process::Stdio::from(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(out_fd)
        }))
        .stderr(std::process::Stdio::from(unsafe {
            std::os::fd::OwnedFd::from_raw_fd(err_fd)
        }))
        .spawn();
    unsafe { libc::close(slave) };

    let child = match spawn_res {
        Ok(c) => c,
        Err(_) => {
            unsafe { libc::close(master) };
            on_activate_finished(inner, false);
            return;
        }
    };

    let (tx, rx) = std::sync::mpsc::channel::<Ctrl>();
    *inner.worker_tx.borrow_mut() = Some(tx);

    let i1 = inner.clone();
    let i2 = inner.clone();
    util::spawn_stream(
        move |ev_tx: Sender<Ev>| {
            let ansi_re = match Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]") {
                Ok(r) => r,
                Err(_) => {
                    unsafe { libc::close(master) };
                    return false;
                }
            };
            let url_re = Regex::new(r"(https?://\S+)").unwrap();
            let speed_re = RegexBuilder::new(r"s\s*[-–]\s*speed up")
                .case_insensitive(true)
                .build()
                .unwrap();
            let cd_re = RegexBuilder::new(r"next check in\s+(\d+)s")
                .case_insensitive(true)
                .build()
                .unwrap();

            let mut buf: Vec<u8> = Vec::new();
            let mut url_emitted = false;
            let mut speed_emitted = false;
            let mut menu_selected = false;
            let mut cancelled = false;
            let mut child = child;

            while !cancelled {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        Ctrl::Key(c) => {
                            let data = format!("{}\r", c);
                            unsafe {
                                libc::write(
                                    master,
                                    data.as_ptr() as *const libc::c_void,
                                    data.len(),
                                )
                            };
                        }
                        Ctrl::Cancel => {
                            cancelled = true;
                            let data = "x\r";
                            unsafe {
                                libc::write(
                                    master,
                                    data.as_ptr() as *const libc::c_void,
                                    data.len(),
                                )
                            };
                            let _ = child.kill();
                            break;
                        }
                    }
                }
                if cancelled {
                    break;
                }

                let mut pfd = libc::pollfd {
                    fd: master,
                    events: libc::POLLIN,
                    revents: 0,
                };
                let n = unsafe { libc::poll(&mut pfd, 1, 300) };
                if n > 0 {
                    let mut chunk = [0u8; 4096];
                    let r = unsafe {
                        libc::read(master, chunk.as_mut_ptr() as *mut libc::c_void, 4096)
                    };
                    if r <= 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..r as usize]);

                    // Strip ANSI for parsing.
                    let lossy = String::from_utf8_lossy(&buf);
                    let clean = ansi_re.replace_all(&lossy, "");
                    let text = clean.replace('\r', "");

                    // Auto-select first menu option (Enter).
                    if !menu_selected && text.contains("How do you want") {
                        std::thread::sleep(Duration::from_millis(200));
                        unsafe {
                            libc::write(master, "\r".as_ptr() as *const libc::c_void, 1)
                        };
                        menu_selected = true;
                    }

                    // URL.
                    if !url_emitted {
                        if let Some(m) = url_re.captures(&text) {
                            let url = m[1].trim_end_matches('.').to_string();
                            let _ = ev_tx.send(Ev::Url(url.clone()));
                            url_emitted = true;
                            let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                        }
                    }

                    // Speed up.
                    if !speed_emitted && speed_re.is_match(&text) {
                        let _ = ev_tx.send(Ev::SpeedUp);
                        speed_emitted = true;
                    }

                    // Countdown.
                    if let Some(m) = cd_re.captures(&text) {
                        if let Ok(sec) = m[1].parse::<u64>() {
                            let _ = ev_tx.send(Ev::Countdown(sec));
                        }
                    }
                } else {
                    // No data — check if process has ended.
                    if child.try_wait().ok().flatten().is_some() {
                        break;
                    }
                }
            }

            // Python: self._proc.wait(timeout=5); timeout => finished(False).
            let deadline = Instant::now() + Duration::from_secs(5);
            let status = loop {
                if let Some(s) = child.try_wait().ok().flatten() {
                    break Some(s);
                }
                if Instant::now() >= deadline {
                    break None;
                }
                std::thread::sleep(Duration::from_millis(50));
            };
            let mut ok = status.map(|s| s.code() == Some(0)).unwrap_or(false);

            // Fallback verify via license command.
            if !ok && !cancelled {
                ok = is_activated();
            }

            unsafe { libc::close(master) };
            ok && !cancelled
        },
        move |ev| {
            match ev {
                Ev::Url(url) => {
                    *i1.auth_url.borrow_mut() = url.clone();
                    i1.lbl_act_url.set_markup(&format!(
                        "<a href=\"{url}\" style=\"color:{green};\">{url}</a>",
                        url = url,
                        green = c::COLOR_ADGUARD_GREEN
                    ));
                    i1.btn_open_browser.set_visible(true);
                }
                Ev::SpeedUp => i1.btn_speed_up.set_visible(true),
                Ev::Countdown(sec) => {
                    let txt = if *sec > 0 {
                        format!("Auto-checking in {}s…", sec)
                    } else {
                        "Checking…".to_string()
                    };
                    i1.lbl_act_countdown.set_text(&txt);
                }
            }
        },
        move |ok| {
            on_activate_finished(&i2, *ok);
        },
    );
}

fn on_activate_finished(inner: &Rc<Inner>, ok: bool) {
    *inner.worker_tx.borrow_mut() = None;
    if ok {
        inner.lbl_act_status.set_text("✓ Activation successful!");
        inner.lbl_act_countdown.set_text("");
        // Reload license info after short delay.
        let i = inner.clone();
        once(1000, move || load_live_license(&i));
    } else {
        inner.lbl_act_status.set_text("Activation failed or was cancelled.");
        inner.btn_open_browser.set_visible(false);
        inner.btn_speed_up.set_visible(false);
    }
}

// ------------------------------------------------------------------
// ⋮ More menu
// ------------------------------------------------------------------
fn show_more_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("menu-popover");
    popover.set_parent(&inner.btn_more);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_refresh = gtk4::Button::new();
    item_refresh.set_label(&i18n::tr("menu_refresh_license"));
    item_refresh.add_css_class("menu-item");
    let i1 = inner.clone();
    let p1 = popover.clone();
    item_refresh.connect_clicked(move |_| {
        p1.hide();
        load_live_license(&i1);
    });
    menu_box.append(&item_refresh);

    let item_manage = gtk4::Button::new();
    item_manage.set_label(&i18n::tr("menu_manage_license"));
    item_manage.add_css_class("menu-item");
    let p2 = popover.clone();
    item_manage.connect_clicked(move |_| {
        p2.hide();
        let _ = std::process::Command::new("xdg-open")
            .arg("https://adguard.com/license.html")
            .spawn();
    });
    menu_box.append(&item_manage);

    let sep = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep.add_css_class("menu-separator");
    menu_box.append(&sep);

    let item_reset = gtk4::Button::new();
    item_reset.set_label(&i18n::tr("menu_reset_license"));
    item_reset.add_css_class("menu-item");
    let i2 = inner.clone();
    let p3 = popover.clone();
    item_reset.connect_clicked(move |_| {
        p3.hide();
        on_reset_license(&i2);
    });
    menu_box.append(&item_reset);

    popover.set_child(Some(&menu_box));
    popover.show();
}

fn on_reset_license(inner: &Rc<Inner>) {
    // QMessageBox.question -> GTK window with Yes/No buttons.
    let dlg = crate::util::sub_window("Reset License");
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    box_.add_css_class("dlg-dark");
    box_.set_margin_top(24);
    box_.set_margin_bottom(20);
    box_.set_margin_start(24);
    box_.set_margin_end(24);

    let lbl = gtk4::Label::new(Some(
        "Are you sure you want to reset and deactivate the AdGuard license on this machine?",
    ));
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
        // Deviation: Python subprocess timeout=10 not enforced.
        match std::process::Command::new(cli_bin()).arg("reset-license").output() {
            Ok(_) => {
                show_message(None, "Reset License", "License has been reset.");
                load_live_license(&i1);
            }
            Err(e) => show_message(None, "Error", &e.to_string()),
        }
    });
    row.append(&btn_yes);

    box_.append(&row);
    dlg.set_child(Some(&box_));
    dlg.present();
}
