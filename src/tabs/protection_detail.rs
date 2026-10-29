// Port of adguard_modern/tabs/protection_detail.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::tabs::protection_data::get_module_translated_data;
use crate::tabs::protection_rows::SubFilterRow;
use crate::widgets::filter_editor_window::FilterEditorWindow;
use crate::widgets::filtering_log_window::FilteringLogDialog;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

type SubItemData = (String, String, String, bool, String, bool);

struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    lbl_warning: gtk4::Label,
    items_container: gtk4::Box,
    sub_rows: RefCell<Vec<(String, SubFilterRow)>>,
    saved_states: RefCell<Vec<(String, bool)>>,
    current_key: RefCell<String>,
    syncing: RefCell<bool>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
    sub_cb: RefCell<Option<Box<dyn Fn(&str, &str, bool)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct ProtectionDetailView {
    inner: Rc<Inner>,
}

impl ProtectionDetailView {
    pub fn new() -> ProtectionDetailView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.add_css_class("transparent-container");
        root.set_margin_top(20);
        root.set_margin_bottom(20);
        root.set_margin_start(36);
        root.set_margin_end(36);

        // Top bar: '< Back' left, '⋮' right.
        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_bar.set_margin_start(8);
        top_bar.set_margin_end(8);

        let btn_back = gtk4::Button::new();
        btn_back.set_label("<  Back");
        btn_back.add_css_class("detail-back-btn");
        top_bar.append(&btn_back);

        // Python: top_bar.addStretch() — keeps ⋮ pinned to the right edge
        let top_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        top_spacer.set_hexpand(true);
        top_bar.append(&top_spacer);

        let btn_more = gtk4::Button::new();
        btn_more.set_size_request(28, 28);
        btn_more.add_css_class("detail-more-btn");
        btn_more.set_child(Some(&svg::svg_image("stat_more.svg", 18)));
        top_bar.append(&btn_more);
        root.append(&top_bar);

        // Title + master toggle header row.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        title_row.set_margin_top(6);
        title_row.set_margin_start(12);
        title_row.set_margin_end(12);

        let lbl_title = gtk4::Label::new(Some("Ad blocking"));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle / description.
        let lbl_desc = gtk4::Label::new(Some(
            "Blocks unwanted ads like popups, banners, and video ads",
        ));
        lbl_desc.add_css_class("lbl-10-white");
        lbl_desc.set_wrap(true);
        lbl_desc.set_xalign(0.0);
        lbl_desc.set_margin_start(12);
        lbl_desc.set_margin_end(12);
        root.append(&lbl_desc);

        // Amber warning when all sub-settings or master is disabled.
        let lbl_warning = gtk4::Label::new(Some(
            "This feature isn’t applied because all of its sub-settings are disabled",
        ));
        lbl_warning.add_css_class("detail-warning");
        lbl_warning.set_wrap(true);
        lbl_warning.set_xalign(0.0);
        lbl_warning.set_margin_top(2);
        lbl_warning.set_margin_start(12);
        lbl_warning.set_margin_end(12);
        lbl_warning.set_visible(false);
        root.append(&lbl_warning);

        root.append(&spacer(6));

        // Scroll area for sub-filters.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let items_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        items_container.add_css_class("transparent-container");
        items_container.set_margin_end(12);
        items_container.set_margin_bottom(16);
        scroll.set_child(Some(&items_container));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            master_switch,
            lbl_desc,
            lbl_warning,
            items_container,
            sub_rows: RefCell::new(Vec::new()),
            saved_states: RefCell::new(Vec::new()),
            current_key: RefCell::new("ad_blocking".to_string()),
            syncing: RefCell::new(false),
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
            sub_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });

        let bb = inner.btn_back.clone();
        let i1 = inner.clone();
        bb.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let bm = inner.btn_more.clone();
        let i2 = inner.clone();
        bm.connect_clicked(move |_| show_options_menu(&i2));

        let i3 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| on_master_toggle(&i3, val)));

        let i4 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i4)));

        retranslate_ui(&inner);

        ProtectionDetailView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_master_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.master_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_sub_item_toggled(&self, cb: Box<dyn Fn(&str, &str, bool)>) {
        *self.inner.sub_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_drilldown_requested(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn set_module(
        &self,
        key: &str,
        title: &str,
        description: &str,
        is_active: bool,
        sub_items: &[SubItemData],
    ) {
        let inner = &self.inner;
        *inner.syncing.borrow_mut() = true;
        *inner.current_key.borrow_mut() = key.to_string();

        // Dynamically fetch translated texts for active language.
        let (t, d, trans_sub_items) = get_module_translated_data(key);
        inner.lbl_title.set_text(if t.is_empty() { title } else { &t });
        inner.lbl_desc.set_text(if d.is_empty() { description } else { &d });
        inner.master_switch.set_checked(is_active, false);

        // Clear previous sub-items.
        while let Some(child) = inner.items_container.first_child() {
            inner.items_container.remove(&child);
        }
        inner.sub_rows.borrow_mut().clear();
        inner.saved_states.borrow_mut().clear();

        // Build translation map for sub items.
        let mut trans_map: Vec<(String, String, String, String)> = Vec::new();
        for item in &trans_sub_items {
            trans_map.push((item.0.clone(), item.1.clone(), item.2.clone(), item.4.clone()));
        }

        // Add new sub-items.
        for item_data in sub_items {
            let sub_key = &item_data.0;
            let (sub_title, sub_desc, warning_text) = match trans_map
                .iter()
                .find(|(k, _, _, _)| k == sub_key)
            {
                Some((_, st, sd, sw)) => (st.clone(), sd.clone(), sw.clone()),
                None => (item_data.1.clone(), item_data.2.clone(), item_data.4.clone()),
            };
            let sub_initial = item_data.3;
            let has_subpage = item_data.5;

            // If master is inactive, sub-items are initially off.
            let init_val = if is_active { sub_initial } else { false };
            let row = SubFilterRow::new(sub_key, &sub_title, &sub_desc, init_val, &warning_text, has_subpage);

            let i = inner.clone();
            let sk = sub_key.clone();
            row.connect_toggled(Box::new(move |sub_key2, is_checked| {
                on_sub_row_toggled(&i, sub_key2, is_checked);
            }));

            if has_subpage {
                let i = inner.clone();
                row.connect_clicked(Box::new(move |k| {
                    if let Some(cb) = i.drilldown_cb.borrow().as_ref() {
                        cb(k);
                    }
                }));
            }

            inner.items_container.append(row.root());
            inner.sub_rows.borrow_mut().push((sub_key.clone(), row));
            inner.saved_states.borrow_mut().push((sub_key.clone(), sub_initial));
        }

        // Set warning state.
        let any_on = inner
            .sub_rows
            .borrow()
            .iter()
            .any(|(_, row)| row.is_checked());
        inner.lbl_warning.set_visible(!any_on || !is_active);

        *inner.syncing.borrow_mut() = false;
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn sub_row(&self, key: &str) -> Option<SubFilterRow> {
        self.inner.sub_rows.borrow().iter().find(|(k, _)| k == key).map(|(_, r)| r.clone())
    }

    pub fn master_switch(&self) -> &ToggleSwitch {
        &self.inner.master_switch
    }

    pub fn current_key(&self) -> String {
        self.inner.current_key.borrow().clone()
    }

    pub fn sub_rows_checked(&self) -> bool {
        self.inner.sub_rows.borrow().iter().any(|(_, r)| r.is_checked())
    }
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_arrow_back_spaced"));
    inner.lbl_warning.set_text(&i18n::tr("warning_sub_disabled"));

    let key = inner.current_key.borrow().clone();
    if !key.is_empty() {
        let (t, d, sub_items) = get_module_translated_data(&key);
        inner.lbl_title.set_text(&t);
        inner.lbl_desc.set_text(&d);
        for (sk, st, sd, _, sw, _) in sub_items {
            if let Some((_, row)) = inner
                .sub_rows
                .borrow()
                .iter()
                .find(|(k, _)| k == &sk)
            {
                row.update_text(&st, &sd, &sw);
            }
        }
    }
}

fn on_master_toggle(inner: &Rc<Inner>, val: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    *inner.syncing.borrow_mut() = true;

    if !val {
        // Save states and turn off all sub-filters.
        let rows: Vec<(String, SubFilterRow)> = inner
            .sub_rows
            .borrow()
            .iter()
            .map(|(k, r)| (k.clone(), (*r).clone()))
            .collect();
        let mut saved = inner.saved_states.borrow_mut();
        saved.clear();
        for (k, row) in &rows {
            saved.push((k.clone(), row.is_checked()));
            row.set_checked(false, true);
        }
        inner.lbl_warning.set_visible(true);
    } else {
        // Restore saved states or turn on defaults.
        let rows: Vec<(String, SubFilterRow)> = inner
            .sub_rows
            .borrow()
            .iter()
            .map(|(k, r)| (k.clone(), r.clone()))
            .collect();
        let saved: Vec<(String, bool)> = inner
            .saved_states
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        let mut has_any = false;
        for (k, row) in &rows {
            let prev = saved
                .iter()
                .find(|(sk, _)| sk == k)
                .map(|(_, v)| *v)
                .unwrap_or(true);
            row.set_checked(prev, true);
            if prev {
                has_any = true;
            }
        }
        if !has_any && !rows.is_empty() {
            // Turn on first item if none was active.
            rows[0].1.set_checked(true, true);
        }
        inner.lbl_warning.set_visible(false);
    }

    *inner.syncing.borrow_mut() = false;
    let key = inner.current_key.borrow().clone();
    if let Some(cb) = inner.master_cb.borrow().as_ref() {
        cb(&key, val);
    }
}

fn on_sub_row_toggled(inner: &Rc<Inner>, sub_key: &str, is_checked: bool) {
    if *inner.syncing.borrow() {
        return;
    }
    *inner.syncing.borrow_mut() = true;

    let any_on = inner
        .sub_rows
        .borrow()
        .iter()
        .any(|(_, row)| row.is_checked());
    let key = inner.current_key.borrow().clone();
    if !any_on {
        inner.master_switch.set_checked(false, true);
        inner.lbl_warning.set_visible(true);
        if let Some(cb) = inner.master_cb.borrow().as_ref() {
            cb(&key, false);
        }
    } else {
        inner.master_switch.set_checked(true, true);
        inner.lbl_warning.set_visible(false);
        if let Some(cb) = inner.master_cb.borrow().as_ref() {
            cb(&key, true);
        }
    }

    *inner.syncing.borrow_mut() = false;
    if let Some(cb) = inner.sub_cb.borrow().as_ref() {
        cb(&key, sub_key, is_checked);
    }
}

fn show_options_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("detail-menu-popover");
    popover.set_parent(&inner.btn_more);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_editor = gtk4::Button::new();
    item_editor.set_label(&i18n::tr("setting_filter_editor"));
    item_editor.add_css_class("detail-menu-item");
    let p1 = popover.clone();
    item_editor.connect_clicked(move |_| {
        p1.hide();
        // Deviation: QDialog.exec() -> non-modal GTK window (no parent).
        FilterEditorWindow::new().root().present();
    });
    menu_box.append(&item_editor);

    let item_log = gtk4::Button::new();
    item_log.set_label(&i18n::tr("home_filtering_log"));
    item_log.add_css_class("detail-menu-item");
    let p2 = popover.clone();
    item_log.connect_clicked(move |_| {
        p2.hide();
        FilteringLogDialog::new().root().present();
    });
    menu_box.append(&item_log);

    popover.set_child(Some(&menu_box));
    popover.show();
}
