// Port of adguard_modern/tabs/language_filters_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::backend;
use crate::i18n;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

// ---------------------------------------------------------------------------
// LanguageItemRow
// ---------------------------------------------------------------------------
struct LangRowInner {
    filter_id: i64,
    name: String,
    updated_time: String,
    root: gtk4::Box,
    lbl_name: gtk4::Label,
    lbl_updated: gtk4::Label,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(i64, bool)>>>,
}

#[derive(Clone)]
pub struct LanguageItemRow {
    inner: Rc<LangRowInner>,
}

impl LanguageItemRow {
    pub fn new(filter_id: i64, name: &str, updated_time: &str, initial_state: bool) -> LanguageItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("language-item-row");
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

        let inner = Rc::new(LangRowInner {
            filter_id,
            name: name.to_string(),
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

        LanguageItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        // Python tr("lang_updated_prefix").format(time=...) — substitute {time}.
        let t = i18n::tr("lang_updated_prefix").replace("{time}", &self.inner.updated_time);
        self.inner.lbl_updated.set_text(&t);
    }

    pub fn set_checked(&self, checked: bool, animate: bool) {
        self.inner.switch.set_checked(checked, animate);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.switch.is_checked()
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(i64, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }
}

// ---------------------------------------------------------------------------
// LanguageFiltersView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    lbl_auto: gtk4::Label,
    auto_switch: ToggleSwitch,
    lbl_section: gtk4::Label,
    search_input: gtk4::Entry,
    items_container: gtk4::Box,
    lang_rows: RefCell<Vec<(i64, LanguageItemRow)>>,
    saved_enabled_fids: RefCell<Vec<i64>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct LanguageFiltersView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl LanguageFiltersView {
    pub fn new() -> LanguageFiltersView {
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

        let lbl_title = gtk4::Label::new(Some("Language-specific ad blocking"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some("Blocks ads on websites in specific languages"));
        lbl_desc.add_css_class("lbl-10-white");
        lbl_desc.set_xalign(0.0);
        lbl_desc.set_margin_start(12);
        lbl_desc.set_margin_end(12);
        root.append(&lbl_desc);

        root.append(&spacer(6));

        // Setting: automatically activate language-specific filters.
        let auto_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        auto_row.add_css_class("language-item-row");
        auto_row.set_margin_top(10);
        auto_row.set_margin_bottom(10);
        auto_row.set_margin_start(12);
        auto_row.set_margin_end(16);

        let lbl_auto = gtk4::Label::new(Some("Automatically activate language-specific filters"));
        lbl_auto.add_css_class("lbl-title");
        lbl_auto.set_halign(gtk4::Align::Start);
        auto_row.append(&lbl_auto);

        let auto_switch = ToggleSwitch::new(46, 26);
        auto_switch.set_checked(backend::get_auto_language_filters(), false);
        auto_switch.root().set_valign(gtk4::Align::Center);
        auto_row.append(auto_switch.root());
        root.append(&auto_row);

        root.append(&spacer(10));

        // Section header: Languages.
        let lbl_section = gtk4::Label::new(Some("Languages"));
        lbl_section.add_css_class("lbl-16-bold-white");
        lbl_section.set_halign(gtk4::Align::Start);
        lbl_section.set_margin_start(12);
        lbl_section.set_margin_end(12);
        root.append(&lbl_section);

        // Search bar.
        let search_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_container.add_css_class("search-box");
        search_container.set_margin_top(8);
        search_container.set_margin_bottom(8);
        search_container.set_margin_start(12);
        search_container.set_margin_end(12);

        let search_svg = svg::svg_image("search.svg", 18);
        search_container.append(&search_svg);

        let search_input = gtk4::Entry::new();
        search_input.set_placeholder_text(Some("Search"));
        search_input.add_css_class("search-entry");
        search_input.set_hexpand(true);
        search_container.append(&search_input);
        root.append(&search_container);

        // Scroll area for languages.
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
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            lbl_auto,
            auto_switch,
            lbl_section,
            search_input,
            items_container,
            lang_rows: RefCell::new(Vec::new()),
            saved_enabled_fids: RefCell::new(Vec::new()),
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
        inner.auto_switch.connect_toggled(Box::new(move |is_enabled| {
            backend::set_auto_language_filters(is_enabled);
            let _ = &i3;
        }));

        let i4 = inner.clone();
        inner.search_input.connect_changed(move |e| filter_languages(&i4, e.text().as_str()));

        load_live_filters(&inner);

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        retranslate_ui(&inner);

        LanguageFiltersView { inner }
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

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn master_switch(&self) -> &ToggleSwitch {
        &self.inner.master_switch
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("lang_title"));
    inner.lbl_desc.set_text(&i18n::tr("lang_desc"));
    inner.lbl_auto.set_text(&i18n::tr("lang_auto_t"));
    inner.lbl_section.set_text(&i18n::tr("lang_sec_languages"));
    inner.search_input.set_placeholder_text(Some(&i18n::tr("lang_search_ph")));
    for (_, row) in inner.lang_rows.borrow().iter() {
        row.retranslate_ui();
    }
}

// Query actual SQLite database to populate real language filters.
fn load_live_filters(inner: &Rc<Inner>) {
    while let Some(child) = inner.items_container.first_child() {
        inner.items_container.remove(&child);
    }
    inner.lang_rows.borrow_mut().clear();

    let filters = backend::query_language_filters_db();
    for f in filters {
        let row = LanguageItemRow::new(f.id, &f.name, &f.updated, f.enabled);

        let i = inner.clone();
        row.connect_toggled(Box::new(move |filter_id, is_enabled| {
            // Asynchronously toggle real filter via adguard-cli.
            backend::toggle_filter(filter_id, is_enabled);
            let _ = &i;
        }));

        inner.items_container.append(row.root());
        inner.lang_rows.borrow_mut().push((f.id, row));
    }
}

fn on_master_toggled(inner: &Rc<Inner>, val: bool) {
    if !val {
        // Save currently enabled language filters, then disable them.
        let rows: Vec<(i64, LanguageItemRow)> = inner
            .lang_rows
            .borrow()
            .iter()
            .map(|(id, r)| (*id, r.clone()))
            .collect();
        let enabled: Vec<i64> = rows.iter().filter(|(_, r)| r.is_checked()).map(|(id, _)| *id).collect();
        *inner.saved_enabled_fids.borrow_mut() = enabled.clone();
        for fid in &enabled {
            if let Some((_, row)) = rows.iter().find(|(id, _)| id == fid) {
                row.set_checked(false, true);
                backend::toggle_filter(*fid, false);
            }
        }
        inner.auto_switch.root().set_sensitive(false);
    } else {
        // Restore previously enabled language filters.
        let saved = inner.saved_enabled_fids.borrow().clone();
        if !saved.is_empty() {
            let rows: Vec<(i64, LanguageItemRow)> = inner
                .lang_rows
                .borrow()
                .iter()
                .map(|(id, r)| (*id, r.clone()))
                .collect();
            for fid in &saved {
                if let Some((_, row)) = rows.iter().find(|(id, _)| id == fid) {
                    row.set_checked(true, true);
                    backend::toggle_filter(*fid, true);
                }
            }
        }
        inner.auto_switch.root().set_sensitive(true);
    }
    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(val);
    }
}

fn filter_languages(inner: &Rc<Inner>, text: &str) {
    let query = text.trim().to_lowercase();
    for (fid, row) in inner.lang_rows.borrow().iter() {
        if query.is_empty() || row_name(row).contains(&query) || &fid.to_string() == &query {
            row.root().set_visible(true);
        } else {
            row.root().set_visible(false);
        }
    }
}

fn row_name(row: &LanguageItemRow) -> String {
    row.name().to_lowercase()
}
