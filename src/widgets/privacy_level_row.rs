// Port of adguard_modern/widgets/privacy_level_row.py
use cairo;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct RadioInner {
    root: gtk4::DrawingArea,
    checked: Cell<bool>,
}

#[derive(Clone)]
pub struct RadioCircle {
    inner: Rc<RadioInner>,
}

impl RadioCircle {
    pub fn new(checked: bool) -> RadioCircle {
        let root = gtk4::DrawingArea::new();
        root.set_size_request(22, 22);
        let inner = Rc::new(RadioInner {
            root,
            checked: Cell::new(checked),
        });
        let r = RadioCircle { inner: inner.clone() };
        let r2 = inner.clone();
        inner.root.set_draw_func(move |_a, cr, w, h| {
            cr.set_antialias(cairo::Antialias::Default);
            let cx = (w as f64) / 2.0;
            let cy = (h as f64) / 2.0;
            let radius = 9.0;
            if r2.checked.get() {
                // Active green ring
                cr.set_line_width(2.0);
                cr.set_source_rgb(0x5E as f64 / 255.0, 0xBB as f64 / 255.0, 0x6A as f64 / 255.0);
                cr.arc(cx, cy, radius, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                // Inner green filled dot
                cr.arc(cx, cy, 4.5, 0.0, std::f64::consts::PI * 2.0);
                cr.fill();
            } else {
                // Unchecked gray ring
                cr.set_line_width(1.8);
                cr.set_source_rgb(0x5A as f64 / 255.0, 0x5A as f64 / 255.0, 0x60 as f64 / 255.0);
                cr.arc(cx, cy, radius, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
            }
        });
        r
    }

    pub fn root(&self) -> &gtk4::DrawingArea {
        &self.inner.root
    }

    pub fn set_checked(&self, val: bool) {
        if self.inner.checked.get() != val {
            self.inner.checked.set(val);
            self.inner.root.queue_draw();
        }
    }

    pub fn is_checked(&self) -> bool {
        self.inner.checked.get()
    }
}

struct Inner {
    root: gtk4::Box,
    level_id: String,
    has_drilldown: bool,
    checked: Cell<bool>,
    radio: RadioCircle,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    lbl_warning: Option<gtk4::Label>,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct PrivacyLevelRow {
    inner: Rc<Inner>,
}

impl PrivacyLevelRow {
    pub fn new(
        level_id: &str,
        title: &str,
        desc: &str,
        warning_text: &str,
        has_drilldown: bool,
    ) -> PrivacyLevelRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("privacy-row");
        root.set_margin_start(12);
        root.set_margin_end(12);
        root.set_margin_top(10);
        root.set_margin_bottom(10);

        let radio = RadioCircle::new(false);
        root.append(radio.root());

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        vbox.set_hexpand(true);
        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        vbox.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(desc));
        lbl_desc.add_css_class("lbl-desc");
        lbl_desc.set_wrap(true);
        lbl_desc.set_halign(gtk4::Align::Start);
        vbox.append(&lbl_desc);

        let lbl_warning = if !warning_text.is_empty() {
            let w = gtk4::Label::new(Some(warning_text));
            w.add_css_class("lbl-warning");
            w.set_wrap(true);
            w.set_halign(gtk4::Align::Start);
            vbox.append(&w);
            Some(w)
        } else {
            None
        };
        root.append(&vbox);

        let inner = Rc::new(Inner {
            root,
            level_id: level_id.to_string(),
            has_drilldown,
            checked: Cell::new(false),
            radio,
            lbl_title,
            lbl_desc,
            lbl_warning,
            clicked_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });
        let row = PrivacyLevelRow { inner: inner.clone() };

        if has_drilldown {
            let vline = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            vline.set_size_request(1, 24);
            vline.add_css_class("divider-v");
            inner.root.append(&vline);

            let arrow = gtk4::Button::new();
            arrow.add_css_class("chevron-btn");
            arrow.set_size_request(28, 28);
            arrow.set_label("›");
            let i2 = inner.clone();
            arrow.connect_clicked(move |_| {
                if let Some(cb) = i2.drilldown_cb.borrow().as_ref() {
                    cb();
                }
            });
            inner.root.append(&arrow);
        }

        // mousePressEvent: left button emits clicked(level_id)
        let gesture = gtk4::GestureClick::new();
        let r2 = row.clone();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // A widget that is not sensitive must not react. GTK stops delivering
            // events to insensitive widgets, but a click controller attached to
            // the container keeps firing, so opting out has to be explicit:
            // with the master switch off the level rows stayed selectable.
            if !r2.inner.root.is_sensitive() {
                return;
            }
            if n == 1 {
                if let Some(cb) = r2.inner.clicked_cb.borrow().as_ref() {
                    cb(&r2.inner.level_id);
                }
            }
        });
        inner.root.add_controller(gesture);

        row
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    /// Enable/disable interaction (Python: row.setEnabled(val)).
    pub fn set_active_state(&self, active: bool) {
        self.inner.root.set_sensitive(active);
        // Python also sets `opacity: 0.5` when the master switch is off
        // (row.setStyleSheet("opacity: 0.5")). GTK does not dim insensitive
        // widgets by itself, so without this the rows stayed fully bright and
        // looked selectable while the master switch was off.
        self.inner.root.set_opacity(if active { 1.0 } else { 0.5 });
    }

    pub fn level_id(&self) -> &str {
        &self.inner.level_id
    }

    pub fn set_checked(&self, val: bool) {
        self.inner.checked.set(val);
        self.inner.radio.set_checked(val);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.checked.get()
    }

    pub fn update_text(&self, title: &str, desc: &str, warning_text: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
        if let Some(w) = &self.inner.lbl_warning {
            if !warning_text.is_empty() {
                w.set_text(warning_text);
            }
        }
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_drilldown_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }
}

