use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use gtk4::glib;
use gtk4::prelude::*;

use cairo;
use crate::backend;
use crate::i18n;
use crate::util;
use crate::widgets::svg;

// Port of AdGuardSpinner: rotating green arc, ~40 fps.
struct SpinnerInner {
    area: gtk4::DrawingArea,
    angle: RefCell<f64>,
    running: RefCell<bool>,
    size: i32,
    rgb: (f64, f64, f64),
}

#[derive(Clone)]
pub struct AdGuardSpinner {
    inner: Rc<SpinnerInner>,
}

fn parse_hex(hex: &str) -> (f64, f64, f64) {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return (0.37, 0.73, 0.42);
    }
    let r = u32::from_str_radix(&h[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u32::from_str_radix(&h[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u32::from_str_radix(&h[4..6], 16).unwrap_or(0) as f64 / 255.0;
    (r, g, b)
}

impl AdGuardSpinner {
    pub fn new(size: i32, color: &str) -> AdGuardSpinner {
        let area = gtk4::DrawingArea::new();
        area.set_size_request(size, size);
        let inner = Rc::new(SpinnerInner {
            area,
            angle: RefCell::new(0.0),
            running: RefCell::new(false),
            size,
            rgb: parse_hex(color),
        });

        let i1 = inner.clone();
        inner.area.set_draw_func(move |_area, cr, _w, _h| {
            let (r, g, b) = i1.rgb;
            let angle = *i1.angle.borrow();
            cr.set_line_width(2.2);
            cr.set_line_cap(cairo::LineCap::Round);
            cr.set_source_rgb(r, g, b);
            // Qt rect adjusted(2,2,-2,-2): centre and radius of the 18x18 box.
            let c = (i1.size as f64) / 2.0;
            let radius = (i1.size as f64) / 2.0 - 2.0;
            let start = -angle * std::f64::consts::PI / 180.0;
            let end = start + 100.0 * std::f64::consts::PI / 180.0;
            cr.arc(c, c, radius, start, end);
            let _ = cr.stroke();
        });

        // Rotation timer, started by start().
        let i2 = inner.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(25), move || {
            if *i2.running.borrow() {
                // Read into a local first: `borrow_mut() = borrow() + ...` keeps two
                // borrows of the same RefCell alive and panics with
                // "RefCell already borrowed" (the Check-for-updates crash).
                let next = (*i2.angle.borrow() + 10.0) % 360.0;
                *i2.angle.borrow_mut() = next;
                i2.area.queue_draw();
            }
            glib::ControlFlow::Continue
        });

        AdGuardSpinner { inner }
    }

    pub fn root(&self) -> &gtk4::DrawingArea {
        &self.inner.area
    }

    pub fn start(&self) {
        *self.inner.angle.borrow_mut() = 0.0;
        self.inner.area.set_visible(true);
        *self.inner.running.borrow_mut() = true;
        self.inner.area.queue_draw();
    }

    pub fn stop(&self) {
        *self.inner.running.borrow_mut() = false;
        self.inner.area.set_visible(false);
    }
}

// Port of CheckUpdateWorker results dict.
#[derive(Clone, Default)]
pub struct UpdateResults {
    filters: String,
    extensions: String,
    sec: String,
    app: String,
}

struct RowInner {
    root: gtk4::Box,
    check: gtk4::Image,
    spinner: AdGuardSpinner,
    lbl_title: gtk4::Label,
    lbl_sub: gtk4::Label,
}

#[derive(Clone)]
pub struct UpdateItemRow {
    inner: Rc<RowInner>,
}

impl UpdateItemRow {
    pub fn new(title: &str, subtitle: &str) -> UpdateItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.set_margin_top(8);
        root.set_margin_bottom(8);

        let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        icon_box.set_size_request(18, 18);
        icon_box.set_valign(gtk4::Align::Start);

        let check = svg::svg_image("green_check.svg", 18);
        icon_box.append(&check);

        let spinner = AdGuardSpinner::new(18, "#5EBB6A");
        spinner.root().set_visible(false);
        icon_box.append(spinner.root());
        root.append(&icon_box);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title-10");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_sub = gtk4::Label::new(Some(subtitle));
        lbl_sub.add_css_class("lbl-desc-9");
        lbl_sub.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_sub);

        root.append(&text_box);

        UpdateItemRow {
            inner: Rc::new(RowInner {
                root,
                check,
                spinner,
                lbl_title,
                lbl_sub,
            }),
        }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_texts(&self, title: Option<&str>, subtitle: Option<&str>) {
        if let Some(t) = title {
            if !t.is_empty() {
                self.inner.lbl_title.set_text(t);
            }
        }
        if let Some(s) = subtitle {
            if !s.is_empty() {
                self.inner.lbl_sub.set_text(s);
            }
        }
    }

    pub fn set_checking(&self) {
        self.inner.check.set_visible(false);
        self.inner.spinner.start();
        self.inner.lbl_sub.set_text("Checking for updates...");
    }

    pub fn set_done(&self, subtitle: &str) {
        self.inner.spinner.stop();
        self.inner.check.set_visible(true);
        self.inner.lbl_sub.set_text(subtitle);
    }
}

struct Inner {
    root: gtk4::Box,
    lbl_header: gtk4::Label,
    item_app: UpdateItemRow,
    item_filters: UpdateItemRow,
    item_extensions: UpdateItemRow,
    item_sec: UpdateItemRow,
    btn_check: gtk4::Button,
    lbl_result: gtk4::Label,
    worker_running: RefCell<bool>,
    app_ver_text: RefCell<String>,
}

#[derive(Clone)]
pub struct UpdatesTab {
    inner: Rc<Inner>,
}

impl UpdatesTab {
    pub fn new() -> UpdatesTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        root.add_css_class("page-bg");
        root.set_margin_top(28);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let lbl_header = gtk4::Label::new(Some("Updates"));
        lbl_header.add_css_class("page-title");
        lbl_header.set_halign(gtk4::Align::Start);
        root.append(&lbl_header);
        root.append(&spacer(10));

        let app_ver_text = detect_cli_version();
        let item_app = UpdateItemRow::new(&app_ver_text, "Up to date");
        let item_filters = UpdateItemRow::new("Filters", "Up to date");
        let item_extensions = UpdateItemRow::new("Extensions", "Up to date");
        let item_sec = UpdateItemRow::new("Browsing security database", "Updated");

        root.append(item_app.root());
        root.append(item_filters.root());
        root.append(item_extensions.root());
        root.append(item_sec.root());
        root.append(&spacer(16));

        let btn_check = gtk4::Button::new();
        btn_check.set_label("Check for updates");
        btn_check.set_size_request(160, 36);
        btn_check.add_css_class("updates-btn");
        btn_check.set_halign(gtk4::Align::Start);
        root.append(&btn_check);

        let lbl_result = gtk4::Label::new(Some(""));
        lbl_result.add_css_class("updates-result");
        lbl_result.set_halign(gtk4::Align::Start);
        root.append(&lbl_result);

        let bc = btn_check.clone();
        let inner = Rc::new(Inner {
            root,
            lbl_header,
            item_app,
            item_filters,
            item_extensions,
            item_sec,
            btn_check,
            lbl_result,
            worker_running: RefCell::new(false),
            app_ver_text: RefCell::new(app_ver_text),
        });

        let i1 = inner.clone();
        bc.connect_clicked(move |_| start_check_update(&i1));

        let i2 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i2)));

        retranslate_ui(&inner);

        UpdatesTab { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    // Python showEvent: automatically checks when the tab is opened.
    pub fn on_show(&self) {
        if !*self.inner.worker_running.borrow() {
            start_check_update(&self.inner);
        }
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

// Port of _detect_cli_version().
fn detect_cli_version() -> String {
    let cli = backend::adguard_cli_path();
    if let Ok(out) = std::process::Command::new(&cli).arg("--version").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return format!("{} for Linux", s);
        }
    }
    "AdGuard CLI for Linux".to_string()
}

fn start_check_update(inner: &Rc<Inner>) {
    if *inner.worker_running.borrow() {
        return;
    }
    *inner.worker_running.borrow_mut() = true;

    inner.btn_check.set_sensitive(false);
    inner.btn_check.set_label("Checking...");
    inner.lbl_result.set_text("Checking for updates...");

    inner.item_app.set_checking();
    inner.item_filters.set_checking();
    inner.item_extensions.set_checking();
    inner.item_sec.set_checking();

    let i2 = inner.clone();
    let i3 = inner.clone();
    let cli = backend::adguard_cli_path();
    util::spawn_stream(
        move |tx: Sender<String>| {
            let mut results = UpdateResults {
                filters: "Up to date".to_string(),
                extensions: "Up to date".to_string(),
                sec: "Updated".to_string(),
                app: "Up to date".to_string(),
            };
            let mut success = true;
            match std::process::Command::new(&cli)
                .arg("check-update")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(mut child) => {
                    if let Some(out) = child.stdout.as_mut() {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(out);
                        for line in reader.lines() {
                            let line = match line {
                                Ok(l) => l,
                                Err(_) => break,
                            };
                            let line = line.trim().to_string();
                            if line.is_empty() {
                                continue;
                            }
                            let _ = tx.send(line.clone());
                            if line.contains("filter(s) updated") {
                                results.filters = line;
                            } else if line.contains("Safebrowsing") || line.contains("Updated") {
                                results.sec = "Updated".to_string();
                            }
                        }
                    }
                    match child.wait() {
                        Ok(status) => success = status.code() == Some(0),
                        Err(_) => success = false,
                    }
                }
                Err(_) => success = false,
            }
            results
        },
        move |line| on_worker_line(&i2, line),
        move |results| on_worker_finished(&i3, results),
    );
}

// Port of _on_worker_line: progressively resolve items as adguard-cli logs output.
fn on_worker_line(inner: &Rc<Inner>, line: &str) {
    let lower = line.to_lowercase();
    if lower.contains("dns filter") || lower.contains("filter(s) updated") {
        let sub = if line.contains("updated") { line } else { "Up to date" };
        inner.item_filters.set_done(sub);
    } else if lower.contains("userscripts") {
        inner.item_extensions.set_done("Up to date");
    } else if lower.contains("safebrowsing") || lower.contains("crlite") {
        inner.item_sec.set_done("Updated");
    } else if lower.contains("app update") {
        inner.item_app.set_done("Up to date");
    }
}

// Port of _on_worker_finished: staggered final statuses (QTimer.singleShot).
fn on_worker_finished(inner: &Rc<Inner>, results: &UpdateResults) {
    *inner.worker_running.borrow_mut() = false;

    let i1 = inner.clone();
    let r1 = results.filters.clone();
    once(200, move || i1.item_filters.set_done(&r1));
    let i2 = inner.clone();
    let r2 = results.extensions.clone();
    once(450, move || i2.item_extensions.set_done(&r2));
    let i3 = inner.clone();
    let r3 = results.sec.clone();
    once(700, move || i3.item_sec.set_done(&r3));
    let i4 = inner.clone();
    let r4 = results.app.clone();
    once(950, move || finalize_check(&i4, &r4));
}

fn finalize_check(inner: &Rc<Inner>, app_status: &str) {
    inner.item_app.set_done(app_status);
    inner.btn_check.set_sensitive(true);
    inner.btn_check.set_label("Check for updates");
    inner.lbl_result.set_text("All filters and modules are up to date.");
}

fn once(ms: u64, f: impl FnOnce() + 'static) {
    let mut f = Some(f);
    glib::timeout_add_local(std::time::Duration::from_millis(ms), move || {
        if let Some(f) = f.take() {
            f();
        }
        glib::ControlFlow::Break
    });
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.lbl_header.set_text(&i18n::tr("updates_header"));
    inner.btn_check.set_label(&i18n::tr("nav_updates"));
    inner.item_app.set_texts(None, Some(&i18n::tr("up_to_date")));
    inner.item_filters.set_texts(Some(&i18n::tr("filters_title")), Some(&i18n::tr("up_to_date")));
    inner.item_extensions.set_texts(
        Some(&i18n::tr("extensions_title")),
        Some(&i18n::tr("up_to_date")),
    );
    inner.item_sec.set_texts(Some(&i18n::tr("sec_db_title")), Some(&i18n::tr("updated_status")));
}
