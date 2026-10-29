// Port of adguard_modern/widgets/filter_editor_item.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::constants as c;

struct Inner {
    root: gtk4::Box,
    pub filter_id: i64,
    pub title: String,
    checkbox: gtk4::CheckButton,
    active: RefCell<bool>,
    toggled_cb: RefCell<Option<Box<dyn Fn(i64, bool)>>>,
    selected_cb: RefCell<Option<Box<dyn Fn(i64)>>>,
}

#[derive(Clone)]
pub struct FilterListItem {
    inner: Rc<Inner>,
}

impl FilterListItem {
    pub fn new(
        filter_id: i64,
        title: &str,
        subtitle: &str,
        tags: &[String],
        is_checked: bool,
    ) -> FilterListItem {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        root.add_css_class("fe-item");
        root.set_margin_start(10);
        root.set_margin_end(10);
        root.set_margin_top(8);
        root.set_margin_bottom(8);

        let checkbox = gtk4::CheckButton::new();
        checkbox.set_active(is_checked);
        root.append(&checkbox);

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text.set_hexpand(true);
        let lbl_t = gtk4::Label::new(Some(title));
        lbl_t.add_css_class("lbl-title-10");
        lbl_t.set_wrap(true);
        lbl_t.set_halign(gtk4::Align::Start);
        text.append(&lbl_t);

        if !subtitle.is_empty() {
            let lbl_sub = gtk4::Label::new(Some(subtitle));
            lbl_sub.add_css_class("lbl-sub-8");
            lbl_sub.set_halign(gtk4::Align::Start);
            text.append(&lbl_sub);
        }

        if !tags.is_empty() {
            let tag_str = tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            let lbl_tags = gtk4::Label::new(Some(&tag_str));
            lbl_tags.add_css_class("lbl-tags-8");
            lbl_tags.set_halign(gtk4::Align::Start);
            text.append(&lbl_tags);
        }
        root.append(&text);

        let inner = Rc::new(Inner {
            root,
            filter_id,
            title: title.to_string(),
            checkbox: checkbox.clone(),
            active: RefCell::new(false),
            toggled_cb: RefCell::new(None),
            selected_cb: RefCell::new(None),
        });
        let item = FilterListItem { inner: inner.clone() };

        let i2 = inner.clone();
        checkbox.connect_toggled(move |cb| {
            if let Some(cb2) = i2.toggled_cb.borrow().as_ref() {
                cb2(i2.filter_id, cb.is_active());
            }
        });

        let i3 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i3.selected_cb.borrow().as_ref() {
                    cb(i3.filter_id);
                }
            }
        });
        inner.root.add_controller(gesture);

        item
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn filter_id(&self) -> i64 {
        self.inner.filter_id
    }

    pub fn title(&self) -> String {
        self.inner.title.clone()
    }

    pub fn set_active(&self, active: bool) {
        *self.inner.active.borrow_mut() = active;
        if active {
            self.inner.root.add_css_class("fe-item-active");
        } else {
            self.inner.root.remove_css_class("fe-item-active");
        }
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(i64, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_selected(&self, cb: Box<dyn Fn(i64)>) {
        *self.inner.selected_cb.borrow_mut() = Some(cb);
    }
}

// Python emits `selected` with the widget object itself; callers use filter_id().
pub fn check_icon_path() -> std::path::PathBuf {
    c::icons_dir().join("check_white.svg")
}
