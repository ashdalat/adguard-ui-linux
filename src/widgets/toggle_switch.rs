// Port of adguard_modern/widgets/toggle_switch.py — custom-painted animated switch.
// Pattern used across widgets/tabs: an Rc-wrapped inner state so clones share it.
use cairo;
use gtk4::glib;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::constants as c;

fn hex_rgb(hex: &str) -> (f64, f64, f64) {
    let r = i64::from_str_radix(&hex[1..3], 16).unwrap_or(0) as f64;
    let g = i64::from_str_radix(&hex[3..5], 16).unwrap_or(0) as f64;
    let b = i64::from_str_radix(&hex[5..7], 16).unwrap_or(0) as f64;
    (r / 255.0, g / 255.0, b / 255.0)
}

struct Inner {
    root: gtk4::DrawingArea,
    w: i32,
    h: i32,
    checked: Cell<bool>,
    position: Cell<f64>,
    cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct ToggleSwitch {
    inner: Rc<Inner>,
}

impl ToggleSwitch {
    pub fn new(width: i32, height: i32) -> ToggleSwitch {
        let root = gtk4::DrawingArea::new();
        root.set_size_request(width, height);

        let inner = Rc::new(Inner {
            root,
            w: width,
            h: height,
            checked: Cell::new(true), // Python: self._checked = True
            position: Cell::new(1.0), // 0.0 (left/off) to 1.0 (right/on)
            cb: RefCell::new(None),
        });
        let sw = ToggleSwitch { inner: inner.clone() };

        let s2 = inner.clone();
        inner.root.set_draw_func(move |_area, cr, _w, _h| {
            draw(&s2, cr);
        });

        // mousePressEvent: left button toggles
        let gesture = gtk4::GestureClick::new();
        let s3 = inner.clone();
        gesture.connect_pressed(move |_g, _n, _x, _y| {
            set_checked(&s3, !s3.checked.get(), true);
        });
        inner.root.add_controller(gesture);

        sw
    }

    pub fn root(&self) -> &gtk4::DrawingArea {
        &self.inner.root
    }

    pub fn is_checked(&self) -> bool {
        self.inner.checked.get()
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        set_checked(&self.inner, checked, animate);
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.cb.borrow_mut() = Some(cb);
    }
}

fn set_checked(inner: &Rc<Inner>, checked: bool, animate: bool) {
    let changed = inner.checked.get() != checked;
    inner.checked.set(checked);
    let target = if inner.checked.get() { 1.0 } else { 0.0 };

    if animate {
        // QPropertyAnimation: 220 ms, InOutCubic
        let start = inner.position.get();
        let dur = 220.0; // ms
        let t0 = std::time::Instant::now();
        let s = inner.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            let t = (std::time::Instant::now().duration_since(t0).as_secs_f64() * 1000.0) / dur;
            let e = if t >= 1.0 {
                1.0
            } else if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-(2.0 * t + 2.0).powi(3)) / 2.0
            };
            s.position.set(start + (target - start) * e);
            s.root.queue_draw();
            if t >= 1.0 {
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    } else {
        inner.position.set(target);
        inner.root.queue_draw();
    }

    if changed {
        if let Some(cb) = inner.cb.borrow().as_ref() {
            cb(checked);
        }
    }
}

fn draw(s: &Rc<Inner>, cr: &cairo::Context) {
    cr.set_antialias(cairo::Antialias::Default);

    let is_large = s.h >= 36;
    let off_hex = if is_large { c::COLOR_SWITCH_TRACK_OFF } else { "#353539" };
    let off_border = if is_large { c::COLOR_SWITCH_TRACK_OFF } else { "#45454B" };
    let (ro, go, bo) = hex_rgb(off_hex);
    let (ron, gon, bon) = hex_rgb(c::COLOR_SWITCH_TRACK_ON);
    let pos = s.position.get();

    let cur = (ro + (ron - ro) * pos, go + (gon - go) * pos, bo + (bon - bo) * pos);
    let mut track = cur;
    // Muted in-progress color while the CLI is running (Qt isEnabled() == false)
    if !s.root.is_sensitive() {
        track = if pos > 0.5 {
            (48.0 / 255.0, 85.0 / 255.0, 54.0 / 255.0)
        } else {
            (42.0 / 255.0, 42.0 / 255.0, 45.0 / 255.0)
        };
    }

    // Track
    let radius = (s.h as f64) / 2.0;
    let border = if pos < 0.5 { hex_rgb(off_border) } else { (ron, gon, bon) };
    cr.set_line_width(1.0);
    cr.set_source_rgb(border.0, border.1, border.2);
    rounded_rect(cr, 0.0, 0.0, s.w as f64, s.h as f64, radius);
    cr.stroke_preserve();
    cr.set_source_rgb(track.0, track.1, track.2);
    cr.fill();

    // Knob
    let margin = 4.0;
    let knob_dia = (s.h as f64) - margin * 2.0;
    let start_x = margin;
    let end_x = (s.w as f64) - knob_dia - margin;
    let knob_x = start_x + (end_x - start_x) * pos;
    let knob_y = margin;

    let knob = if is_large {
        if pos > 0.5 {
            hex_rgb(c::COLOR_SWITCH_KNOB_ON)
        } else {
            hex_rgb(c::COLOR_SWITCH_KNOB_OFF)
        }
    } else if pos > 0.5 {
        (1.0, 1.0, 1.0)
    } else {
        hex_rgb("#B0B0B5")
    };

    let cx = knob_x + knob_dia / 2.0;
    let cy = knob_y + knob_dia / 2.0;

    cr.set_source_rgb(knob.0, knob.1, knob.2);
    cr.arc(cx, cy, knob_dia / 2.0, 0.0, std::f64::consts::PI * 2.0);
    cr.fill();

    if pos > 0.3 && is_large {
        // Checkmark inside the knob when ON (large switches, e.g. Home)
        let alpha = 255.0 * ((pos - 0.3) / 0.7);
        cr.set_line_width(3.2);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);
        cr.set_source_rgba(94.0 / 255.0, 187.0 / 255.0, 106.0 / 255.0, alpha / 255.0);
        cr.move_to(cx - 6.0, cy);
        cr.line_to(cx - 2.0, cy + 5.0);
        cr.line_to(cx + 7.0, cy - 5.0);
        cr.stroke();
    } else if pos <= 0.3 && is_large {
        // Circular orange ring inside the knob when OFF
        let alpha = 255.0 * (1.0 - (pos / 0.3));
        cr.set_line_width(3.2);
        cr.set_source_rgba(239.0 / 255.0, 90.0 / 255.0, 59.0 / 255.0, alpha / 255.0);
        cr.arc(cx, cy, 5.5, 0.0, std::f64::consts::PI * 2.0);
        cr.stroke();
    }
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.move_to(x + r, y);
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(x + r, y + r, r, std::f64::consts::PI, std::f64::consts::FRAC_PI_2 * 3.0);
    cr.close_path();
}
