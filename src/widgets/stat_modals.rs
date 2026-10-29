// Port of adguard_modern/widgets/stat_modals.py — the three statistics modals.
// Qt QDialog accept/reject is represented by a result callback plus closing the window.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use crate::i18n;

struct Inner {
    root: gtk4::Window,
    accepted_cb: RefCell<Option<Box<dyn Fn()>>>,
    rejected_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct WhySafeModal {
    inner: Rc<Inner>,
}

#[derive(Clone)]
pub struct TurnOffStatsModal {
    inner: Rc<Inner>,
}

#[derive(Clone)]
pub struct ClearStatsModal {
    inner: Rc<Inner>,
}

fn modal(title: &str, width: i32, height: i32) -> (Rc<Inner>, gtk4::Box) {
    let root = crate::util::sub_window("");
    root.set_default_size(width, height);
    root.add_css_class("stat-modal");

    let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    outer.set_margin_start(10);
    outer.set_margin_end(10);
    outer.set_margin_top(10);
    outer.set_margin_bottom(10);
    root.set_child(Some(&outer));

    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    card.add_css_class("stat-card");
    card.set_margin_start(26);
    card.set_margin_end(26);
    card.set_margin_top(22);
    card.set_margin_bottom(24);
    outer.append(&card);

    let inner = Rc::new(Inner {
        root: root.clone(),
        accepted_cb: RefCell::new(None),
        rejected_cb: RefCell::new(None),
    });

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let lbl_title = gtk4::Label::new(Some(title));
    lbl_title.add_css_class("stat-title");
    lbl_title.set_halign(gtk4::Align::Start);
    header.append(&lbl_title);
    let stretch = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    stretch.set_hexpand(true);
    header.append(&stretch);

    let i2 = inner.clone();
    let btn_close = gtk4::Button::new();
    btn_close.add_css_class("dlg-close");
    btn_close.set_size_request(28, 28);
    btn_close.set_label("✕");
    btn_close.connect_clicked(move |_| {
        fire(&i2.rejected_cb);
        i2.root.set_visible(false);
    });
    header.append(&btn_close);
    card.append(&header);

    (inner, card)
}

fn fire(cb_slot: &RefCell<Option<Box<dyn Fn()>>>) {
    if let Some(cb) = cb_slot.borrow().as_ref() {
        cb();
    }
}

fn open_url(url: &str) {
    let _ = Command::new("xdg-open").arg(url).spawn();
}

fn action_button(label: &str, css: &str, inner: &Rc<Inner>, fire_accepted: bool) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class(css);
    btn.set_size_request(-1, 42);
    btn.set_label(label);
    let i2 = inner.clone();
    btn.connect_clicked(move |_| {
        if fire_accepted {
            fire(&i2.accepted_cb);
        } else {
            fire(&i2.rejected_cb);
        }
        i2.root.set_visible(false);
    });
    btn
}

impl WhySafeModal {
    pub fn new() -> WhySafeModal {
        let (inner, card) = modal("Why it’s safe", 450, 360);

        let p1 = gtk4::Label::new(Some(
            "AdGuard filters traffic locally, right on your device. As the connection is \
             encrypted and secure, nobody can see your activity data — we only calculate your traffic usage",
        ));
        p1.set_wrap(true);
        p1.set_xalign(0.0);
        p1.add_css_class("stat-body");
        card.append(&p1);

        let p2 = gtk4::Label::new(Some(
            "All stats are collected and stored locally. We don’t have access to them",
        ));
        p2.set_wrap(true);
        p2.set_xalign(0.0);
        p2.add_css_class("stat-body");
        card.append(&p2);

        let link = gtk4::Button::new();
        link.add_css_class("green-link");
        link.set_label("Privacy policy");
        link.connect_clicked(|_| open_url("https://adguard.com/en/privacy.html"));
        card.append(&link);

        let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        card.append(&spacer);

        card.append(&action_button("Got it", "stat-green-btn", &inner, true));

        WhySafeModal { inner }
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.inner.accepted_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_rejected(&self, cb: Box<dyn Fn()>) {
        *self.inner.rejected_cb.borrow_mut() = Some(cb);
    }
}

impl TurnOffStatsModal {
    pub fn new() -> TurnOffStatsModal {
        let (inner, card) = modal("Turn off statistics?", 450, 210);

        let sub = gtk4::Label::new(Some("Your usage data won’t be collected"));
        sub.set_xalign(0.0);
        sub.add_css_class("stat-body");
        card.append(&sub);

        let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        card.append(&spacer);

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.append(&action_button("Turn off", "stat-orange-btn", &inner, true));
        row.append(&action_button(&i18n::tr("btn_cancel"), "stat-cancel-btn", &inner, false));
        card.append(&row);

        TurnOffStatsModal { inner }
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.inner.accepted_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_rejected(&self, cb: Box<dyn Fn()>) {
        *self.inner.rejected_cb.borrow_mut() = Some(cb);
    }
}

impl ClearStatsModal {
    pub fn new() -> ClearStatsModal {
        let (inner, card) = modal("Clear statistics?", 460, 230);

        let sub = gtk4::Label::new(Some(
            "All requests from the filtering log and stats on data usage will be permanently deleted",
        ));
        sub.set_wrap(true);
        sub.set_xalign(0.0);
        sub.add_css_class("stat-body");
        card.append(&sub);

        let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        card.append(&spacer);

        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row.append(&action_button("Clear", "stat-orange-btn", &inner, true));
        row.append(&action_button(&i18n::tr("btn_cancel"), "stat-cancel-btn", &inner, false));
        card.append(&row);

        ClearStatsModal { inner }
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.inner.accepted_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_rejected(&self, cb: Box<dyn Fn()>) {
        *self.inner.rejected_cb.borrow_mut() = Some(cb);
    }
}
