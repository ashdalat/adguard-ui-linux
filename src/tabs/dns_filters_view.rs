// Port of adguard_modern/tabs/dns_filters_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use rusqlite::Connection;

use crate::backend;
use crate::constants;
use crate::i18n;
use crate::tabs::dns_filter_editor::DnsFilterEditorWindow;
use crate::widgets::common_controls::BackButton;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn dns_db_path() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/agflm_dns.db")
}

// ---------------------------------------------------------------------------
// DnsFilterItemRow
// ---------------------------------------------------------------------------
struct FilterRowInner {
    filter_id: i64,
    name: String,
    root: gtk4::Box,
    switch: ToggleSwitch,
    toggled_cb: RefCell<Option<Box<dyn Fn(i64, bool)>>>,
}

#[derive(Clone)]
pub struct DnsFilterItemRow {
    inner: Rc<FilterRowInner>,
}

impl DnsFilterItemRow {
    pub fn new(filter_id: i64, name: &str, is_enabled: bool) -> DnsFilterItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("dns-filter-row");
        root.set_margin_top(10);
        root.set_margin_bottom(10);
        root.set_margin_start(14);
        root.set_margin_end(16);

        let t_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        t_box.set_hexpand(true);

        let lbl_name = gtk4::Label::new(Some(name));
        lbl_name.add_css_class("lbl-title-10");
        lbl_name.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_name);

        let lbl_id = gtk4::Label::new(Some(&format!("ID: {filter_id}")));
        lbl_id.add_css_class("lbl-8-muted");
        lbl_id.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_id);

        root.append(&t_box);

        let switch = ToggleSwitch::new(44, 24);
        switch.set_checked(is_enabled, false);
        switch.root().set_valign(gtk4::Align::Center);
        root.append(switch.root());

        let inner = Rc::new(FilterRowInner {
            filter_id,
            name: name.to_string(),
            root,
            switch,
            toggled_cb: RefCell::new(None),
        });

        let i = inner.clone();
        inner.switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i.toggled_cb.borrow().as_ref() {
                cb(i.filter_id, val);
            }
        }));

        DnsFilterItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn name(&self) -> &str {
        &self.inner.name
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
}

// ---------------------------------------------------------------------------
// DnsFiltersView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    search_input: gtk4::Entry,
    btn_open_editor: gtk4::Button,
    container: gtk4::Box,
    rows: RefCell<HashMap<i64, DnsFilterItemRow>>,
    editor_win: RefCell<Option<DnsFilterEditorWindow>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct DnsFiltersView {
    inner: Rc<Inner>,
}

impl DnsFiltersView {
    pub fn new() -> DnsFiltersView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("dns-protection-view");
        root.set_margin_top(24);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // 1. Back button.
        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);

        // 2. Title row + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("DNS filters"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // 3. Subtitle.
        let lbl_desc = gtk4::Label::new(Some(
            "Block ads and tracking domains at the DNS level with pre-configured filter lists",
        ));
        lbl_desc.add_css_class("lbl-10");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        root.append(&lbl_desc);

        // 4. Search bar.
        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        search_box.add_css_class("dns-search-box");
        search_box.set_margin_top(0);
        search_box.set_margin_bottom(0);
        search_box.set_margin_start(10);
        search_box.set_margin_end(10);

        // Python: only add the icon widget when the file exists.
        let search_icon_path = constants::icon_path("icon_search.svg");
        if search_icon_path.exists() {
            search_box.append(&svg::svg_image("icon_search.svg", 14));
        }

        let search_input = gtk4::Entry::new();
        search_input.set_placeholder_text(Some("Search DNS filters..."));
        search_input.add_css_class("search-entry");
        search_input.set_hexpand(true);
        search_box.append(&search_input);
        root.append(&search_box);

        // Action: open DNS filter editor.
        let btn_open_editor = gtk4::Button::new();
        btn_open_editor.set_label("+   Open DNS filter editor");
        btn_open_editor.add_css_class("dns-green-action");
        btn_open_editor.set_halign(gtk4::Align::Start);
        root.append(&btn_open_editor);

        // 5. Scroll area for filters.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        container.add_css_class("transparent-container");
        container.set_margin_end(8);
        container.set_margin_bottom(16);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            lbl_title,
            master_switch,
            lbl_desc,
            search_input,
            btn_open_editor,
            container,
            rows: RefCell::new(HashMap::new()),
            editor_win: RefCell::new(None),
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.root.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggled(&i2, val)));

        let i3 = inner.clone();
        inner.search_input.connect_changed(move |e| filter_items(&i3, e.text().as_str()));

        let i4 = inner.clone();
        inner.btn_open_editor.connect_clicked(move |_| open_dns_filter_editor(&i4));

        let i5 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i5)));

        retranslate_ui(&inner);
        load_filters(&inner);

        DnsFiltersView { inner }
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

    pub fn load_filters(&self) {
        load_filters(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("dns_f_title"));
    inner.lbl_desc.set_text(&i18n::tr("dns_f_desc"));
    inner.search_input.set_placeholder_text(Some(&i18n::tr("dns_f_search_ph")));
    inner.btn_open_editor.set_label(&i18n::tr("dns_f_btn_editor"));
}

fn load_filters(inner: &Rc<Inner>) {
    // Clear existing.
    while let Some(child) = inner.container.first_child() {
        inner.container.remove(&child);
    }
    inner.rows.borrow_mut().clear();

    let path = dns_db_path();
    if !path.exists() {
        return;
    }

    let conn = match Connection::open(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut stmt = match conn.prepare(
        "SELECT filter_id, title, is_enabled FROM filter WHERE filter_id > 0 ORDER BY is_enabled DESC, title ASC",
    ) {
        Ok(s) => s,
        Err(_) => return,
    };
    let fetched: Vec<(i64, String, bool)> = stmt
        .query_map([], |r| {
            let fid: i64 = r.get(0)?;
            let title: String = r.get(1)?;
            let is_en: i64 = r.get(2)?;
            Ok((fid, title, is_en != 0))
        })
        .map(|rows| rows.filter_map(|x| x.ok()).collect())
        .unwrap_or_default();

    let mut has_enabled = false;
    let mut rows_map = inner.rows.borrow_mut();
    for (fid, title, en) in fetched {
        if en {
            has_enabled = true;
        }
        let row = DnsFilterItemRow::new(fid, &title, en);

        let i = inner.clone();
        row.connect_toggled(Box::new(move |filter_id, is_enabled| on_filter_toggled(&i, filter_id, is_enabled)));

        inner.container.append(row.root());
        rows_map.insert(fid, row);
    }
    drop(rows_map);

    inner.master_switch.set_checked(has_enabled, false);
}

fn filter_items(inner: &Rc<Inner>, text: &str) {
    let query = text.trim().to_lowercase();
    for row in inner.rows.borrow().values() {
        if query.is_empty() || row.name().to_lowercase().contains(&query) {
            row.root().set_visible(true);
        } else {
            row.root().set_visible(false);
        }
    }
}

fn on_filter_toggled(inner: &Rc<Inner>, fid: i64, is_enabled: bool) {
    let action = if is_enabled { "enable" } else { "disable" };
    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    backend::run_cmd_async(&format!("adguard-cli dns filters {action} {fid}"), noop);

    // Update master switch state.
    let has_any = inner.rows.borrow().values().any(|r| r.is_checked());
    inner.master_switch.set_checked(has_any, true);
}

fn on_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    let noop: Box<dyn Fn(bool, String)> = Box::new(|_ok, _out| ());
    if !is_active {
        let rows: Vec<(i64, DnsFilterItemRow)> = inner
            .rows
            .borrow()
            .iter()
            .map(|(k, r)| (*k, r.clone()))
            .collect();
        for (fid, row) in rows {
            if row.is_checked() {
                row.set_checked(false, false);
                backend::run_cmd_async(&format!("adguard-cli dns filters disable {fid}"), Box::new(|_ok, _out| ()));
            }
        }
    } else {
        // Enable standard AdGuard DNS filter (ID 1).
        let row = inner.rows.borrow().get(&1).cloned();
        if let Some(row) = row {
            row.set_checked(true, true);
            backend::run_cmd_async("adguard-cli dns filters enable 1", noop);
        }
    }

    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(is_active);
    }
}

fn open_dns_filter_editor(inner: &Rc<Inner>) {
    {
        let mut guard = inner.editor_win.borrow_mut();
        if guard.is_none() {
            let win = DnsFilterEditorWindow::new();
            let i = inner.clone();
            win.connect_closed(Box::new(move || load_filters(&i)));
            *guard = Some(win);
        }
    }
    let win = inner.editor_win.borrow();
    let w = win.as_ref().unwrap();
    // show + raise + activateWindow -> present().
    w.root().present();
}
