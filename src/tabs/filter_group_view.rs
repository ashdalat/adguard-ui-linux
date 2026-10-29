// Port of adguard_modern/tabs/filter_group_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::backend;
use crate::i18n;
use crate::widgets::toggle_switch::ToggleSwitch;

// ---------------------------------------------------------------------------
// FilterItemRow
// ---------------------------------------------------------------------------
struct FilterRowInner {
    filter_id: i64,
    updated_time: String,
    root: gtk4::Box,
    lbl_name: gtk4::Label,
    lbl_updated: gtk4::Label,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(i64, bool)>>>,
}

#[derive(Clone)]
pub struct FilterItemRow {
    inner: Rc<FilterRowInner>,
}

impl FilterItemRow {
    pub fn new(filter_id: i64, name: &str, updated_time: &str, initial_state: bool) -> FilterItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("filter-item-row");
        root.set_margin_top(10);
        root.set_margin_bottom(10);
        root.set_margin_start(12);
        root.set_margin_end(16);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let lbl_name = gtk4::Label::new(Some(name));
        lbl_name.add_css_class("lbl-title");
        lbl_name.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_name);

        let lbl_updated = gtk4::Label::new(Some(""));
        lbl_updated.add_css_class("lbl-desc-9");
        lbl_updated.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_updated);

        root.append(&text_box);

        let switch = ToggleSwitch::new(46, 26);
        switch.set_checked(initial_state, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(FilterRowInner {
            filter_id,
            updated_time: updated_time.to_string(),
            root,
            lbl_name,
            lbl_updated,
            switch,
            toggled_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(i.filter_id, val);
            }
        }));

        let t = i18n::tr("lang_updated_prefix").replace("{time}", &inner.updated_time);
        inner.lbl_updated.set_text(&t);

        FilterItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        let t = i18n::tr("lang_updated_prefix").replace("{time}", &self.inner.updated_time);
        self.inner.lbl_updated.set_text(&t);
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        self.inner.switch.set_checked(checked, animate);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.switch.is_checked()
    }

    pub fn filter_id(&self) -> i64 {
        self.inner.filter_id
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(i64, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// FilterGroupDetailView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    group_id: RefCell<i64>,
    btn_back: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    lbl_section: gtk4::Label,
    items_container: gtk4::Box,
    filter_rows: RefCell<Vec<(i64, FilterItemRow)>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

pub struct FilterGroupDetailView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl FilterGroupDetailView {
    pub fn new() -> FilterGroupDetailView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("transparent-container");
        root.set_margin_top(20);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: '< Back' left.
        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_bar.set_margin_start(8);
        top_bar.set_margin_end(8);

        let btn_back = gtk4::Button::new();
        btn_back.set_label("<  Back");
        btn_back.add_css_class("detail-back-btn");
        top_bar.append(&btn_back);
        root.append(&top_bar);

        // Title + master toggle header row.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        title_row.set_margin_top(6);
        title_row.set_margin_start(12);
        title_row.set_margin_end(12);

        let lbl_title = gtk4::Label::new(Some("Basic protection"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some("Removes most ads from websites"));
        lbl_desc.add_css_class("lbl-10-white");
        lbl_desc.set_xalign(0.0);
        lbl_desc.set_margin_start(12);
        lbl_desc.set_margin_end(12);
        root.append(&lbl_desc);

        root.append(&spacer(10));

        // Section header: Filters.
        let lbl_section = gtk4::Label::new(Some("Filters"));
        lbl_section.add_css_class("lbl-16-bold-white");
        lbl_section.set_halign(gtk4::Align::Start);
        lbl_section.set_margin_start(12);
        lbl_section.set_margin_end(12);
        root.append(&lbl_section);

        // Scroll area for filters.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let items_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        items_container.add_css_class("transparent-container");
        items_container.set_margin_top(4);
        items_container.set_margin_end(12);
        items_container.set_margin_bottom(16);
        scroll.set_child(Some(&items_container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            group_id: RefCell::new(1),
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            lbl_section,
            items_container,
            filter_rows: RefCell::new(Vec::new()),
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggled(&i2, val)));

        let i3 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i3)));

        retranslate_ui(&inner);

        FilterGroupDetailView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_master_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.master_cb.borrow_mut() = Some(cb);
    }

    // Query real filters from SQLite for the given group.
    pub fn load_group(&self, group_id: i64, title: &str, description: &str, is_active: bool) {
        let inner = &self.inner;
        *inner.group_id.borrow_mut() = group_id;
        inner.lbl_title.set_text(title);
        inner.lbl_desc.set_text(description);
        inner.master_switch.set_checked(is_active, false);

        // Clear existing items.
        while let Some(child) = inner.items_container.first_child() {
            inner.items_container.remove(&child);
        }
        inner.filter_rows.borrow_mut().clear();

        let filters = backend::query_group_filters_db(group_id);
        for f in filters {
            let row = FilterItemRow::new(f.id, &f.name, &f.updated, f.enabled);

            let i = inner.clone();
            row.connect_toggled(Box::new(move |filter_id, is_enabled| {
                backend::toggle_filter(filter_id, is_enabled);
                let _ = &i;
            }));

            inner.items_container.append(row.root());
            inner.filter_rows.borrow_mut().push((f.id, row));
        }
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_section.set_text(&i18n::tr("fg_sec_filters"));
    if *inner.group_id.borrow() == 1 {
        inner.lbl_title.set_text(&i18n::tr("fg_basic_title"));
        inner.lbl_desc.set_text(&i18n::tr("fg_basic_desc"));
    }
    for (_, row) in inner.filter_rows.borrow().iter() {
        row.retranslate_ui();
    }
}

fn on_master_toggled(inner: &Rc<Inner>, val: bool) {
    let rows: Vec<(i64, FilterItemRow)> = inner
        .filter_rows
        .borrow()
        .iter()
        .map(|(id, r)| (*id, r.clone()))
        .collect();
    for (fid, row) in &rows {
        row.set_checked(val, true);
        backend::toggle_filter(*fid, val);
    }
    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(val);
    }
}
