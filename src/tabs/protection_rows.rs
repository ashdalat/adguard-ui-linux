// Port of adguard_modern/tabs/protection_rows.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

// ---------------------------------------------------------------------------
// ProtectionItemRow
// ---------------------------------------------------------------------------
struct RowInner {
    key: String,
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    switch: Option<ToggleSwitch>,
    toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct ProtectionItemRow {
    inner: Rc<RowInner>,
}

impl ProtectionItemRow {
    pub fn new(
        key: &str,
        icon_name: &str,
        title: &str,
        description: &str,
        initial_state: bool,
        has_switch: bool,
    ) -> ProtectionItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("protection-item-row");
        root.set_margin_top(12);
        root.set_margin_bottom(12);
        root.set_margin_start(12);
        root.set_margin_end(16);

        // Left icon.
        let icon = svg::svg_image(icon_name, 24);
        icon.set_valign(gtk4::Align::Start);
        root.append(&icon);

        // Text container (title + description).
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        text_box.append(&lbl_desc);

        root.append(&text_box);

        let mut switch: Option<ToggleSwitch> = None;
        if has_switch {
            // Vertical divider line to indicate drill-down subpage.
            let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            divider.add_css_class("divider-v");
            divider.set_size_request(1, 32);
            divider.set_valign(gtk4::Align::Center);
            root.append(&divider);

            // Right toggle switch.
            let sw = ToggleSwitch::new(46, 26);
            sw.set_checked(initial_state, false);
            sw.root().set_valign(gtk4::Align::Center);
            root.append(sw.root());
            switch = Some(sw);
        } else {
            let chevron = gtk4::Label::new(Some("›"));
            chevron.add_css_class("chevron-label");
            chevron.set_valign(gtk4::Align::Center);
            root.append(&chevron);
        }

        let inner = Rc::new(RowInner {
            key: key.to_string(),
            root,
            lbl_title,
            lbl_desc,
            switch,
            toggled_cb: RefCell::new(None),
            clicked_cb: RefCell::new(None),
        });

        if let Some(sw) = inner.switch.as_ref() {
            let i = inner.clone();
            sw.connect_toggled(Box::new(move |val| {
                if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                    cb(&i.key, val);
                }
            }));
        }

        let i = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                    cb(&i.key);
                }
            }
        });
        inner.root.add_controller(gesture);

        ProtectionItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_text(&self, title: &str, description: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(description);
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }


    pub fn set_checked(&self, checked: bool, animate: bool) {
        if let Some(s) = self.inner.switch.as_ref() {
            s.set_checked(checked, animate);
        }
    }

    pub fn is_checked(&self) -> bool {
        self.inner.switch.as_ref().map(|s| s.is_checked()).unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// SubFilterRow
// ---------------------------------------------------------------------------
struct SubInner {
    key: String,
    warning_text: RefCell<String>,
    has_subpage: bool,
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    lbl_warning: gtk4::Label,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct SubFilterRow {
    inner: Rc<SubInner>,
}

impl SubFilterRow {
    pub fn new(
        key: &str,
        title: &str,
        description: &str,
        initial_state: bool,
        warning_text: &str,
        has_subpage: bool,
    ) -> SubFilterRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("subfilter-row");
        if has_subpage {
            root.add_css_class("subfilter-row-clickable");
        }
        root.set_margin_top(10);
        root.set_margin_bottom(10);
        root.set_margin_start(12);
        root.set_margin_end(16);

        // Text container.
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        text_box.append(&lbl_desc);

        // Warning note (shown when OFF).
        let lbl_warning = gtk4::Label::new(Some(warning_text));
        lbl_warning.add_css_class("lbl-warning");
        lbl_warning.set_wrap(true);
        lbl_warning.set_xalign(0.0);
        text_box.append(&lbl_warning);

        // Initial visibility of warning.
        if !warning_text.is_empty() && !initial_state {
            lbl_warning.set_visible(true);
        } else {
            lbl_warning.set_visible(false);
        }

        root.append(&text_box);

        // Vertical divider line (drill-down subpage).
        if has_subpage {
            let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            divider.add_css_class("divider-v-2");
            divider.set_size_request(1, 30);
            divider.set_valign(gtk4::Align::Center);
            root.append(&divider);
        }

        // Right toggle switch.
        let switch = ToggleSwitch::new(46, 26);
        switch.set_checked(initial_state, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(SubInner {
            key: key.to_string(),
            warning_text: RefCell::new(warning_text.to_string()),
            has_subpage,
            root,
            lbl_title,
            lbl_desc,
            lbl_warning,
            switch,
            toggled_cb: RefCell::new(None),
            clicked_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |is_checked| {
            if !i.warning_text.borrow().is_empty() {
                i.lbl_warning.set_visible(!is_checked);
            }
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(&i.key, is_checked);
            }
        }));

        if has_subpage {
            let i = inner.clone();
            let gesture = gtk4::GestureClick::new();
            gesture.connect_pressed(move |_g, n, _x, _y| {
                // ignore clicks while the row is disabled
                if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
                if n == 1 {
                    if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                        cb(&i.key);
                    }
                }
            });
            inner.root.add_controller(gesture);
        }

        SubFilterRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        self.inner.switch.set_checked(checked, animate);
        if !self.inner.warning_text.borrow().is_empty() {
            self.inner.lbl_warning.set_visible(!checked);
        }
    }

    pub fn is_checked(&self) -> bool {
        self.inner.switch.is_checked()
    }

    pub fn update_text(&self, title: &str, description: &str, warning_text: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(description);
        *self.inner.warning_text.borrow_mut() = warning_text.to_string();
        self.inner.lbl_warning.set_text(warning_text);
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }
}
