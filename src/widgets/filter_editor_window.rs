// Port of adguard_modern/widgets/filter_editor_window.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use crate::constants as c;
use crate::filter_db;
use crate::i18n;
use crate::widgets::filter_editor_item::FilterListItem;

struct Inner {
    root: gtk4::Window,
    lbl_win_title: gtk4::Label,
    input_search_filters: gtk4::Entry,
    input_search_rules: gtk4::Entry,
    btn_syntax: gtk4::Button,
    lbl_rules_header: gtk4::Label,
    lbl_empty_msg: gtk4::Label,
    btn_add_r: gtk4::Button,
    btn_imp_r: gtk4::Button,
    tb_add: gtk4::Button,
    tb_del: gtk4::Button,
    empty_widget: gtk4::Box,
    rules_scroll: gtk4::ScrolledWindow,
    left_container: gtk4::Box,
    rules_container: gtk4::Box,
    filter_items: RefCell<Vec<FilterListItem>>,
    user_rules: RefCell<Vec<String>>,
    selected_item: RefCell<Option<FilterListItem>>,
}

#[derive(Clone)]
pub struct FilterEditorWindow {
    inner: Rc<Inner>,
}

impl FilterEditorWindow {
    pub fn new() -> FilterEditorWindow {
        let root = crate::util::sub_window("Filter editor");
        root.set_default_size(1000, 680);
        root.add_css_class("fe-window");

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_child(Some(&col));

        // Header
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header.add_css_class("fe-header");
        header.set_size_request(-1, 44);
        header.set_margin_start(16);
        header.set_margin_end(16);
        let lbl_win_title = gtk4::Label::new(Some("Chỉnh sửa quy tắc"));
        lbl_win_title.add_css_class("lbl-title-10");
        header.append(&lbl_win_title);
        let hs = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        hs.set_hexpand(true);
        header.append(&hs);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(28, 28);
        btn_close.set_label("✕");
        let r2 = root.clone();
        btn_close.connect_clicked(move |_| r2.set_visible(false));
        header.append(&btn_close);
        col.append(&header);

        // Body: left / right panes (QSplitter equivalent = two panes with a divider)
        let body = gtk4::Box::new(gtk4::Orientation::Horizontal, 1);
        body.set_vexpand(true);
        col.append(&body);

        // LEFT PANE
        let left = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        left.set_size_request(320, -1);
        left.set_margin_start(12);
        left.set_margin_end(12);
        left.set_margin_top(12);
        left.set_margin_bottom(10);
        body.append(&left);

        let search_filters = search_box("Tìm bộ lọc");
        left.append(&search_filters.0);
        let input_search_filters = search_filters.1;

        let left_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        left_container.set_margin_end(4);
        let scroll_left = gtk4::ScrolledWindow::new();
        scroll_left.set_child(Some(&left_container));
        scroll_left.set_vexpand(true);
        left.append(&scroll_left);

        let left_bottom = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        left_bottom.set_margin_start(4);
        left_bottom.set_margin_end(4);
        let btn_add_f = tool_button("+");
        btn_add_f.set_tooltip_text(Some("Add custom filter"));
        left_bottom.append(&btn_add_f);
        let btn_opt_f = tool_button("⚙");
        left_bottom.append(&btn_opt_f);
        left.append(&left_bottom);

        // RIGHT PANE
        let right = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        right.set_hexpand(true);
        right.set_margin_start(16);
        right.set_margin_end(16);
        right.set_margin_top(12);
        right.set_margin_bottom(10);
        body.append(&right);

        let r_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let search_rules = search_box("Tìm quy tắc");
        search_rules.0.set_size_request(280, -1);
        r_header.append(&search_rules.0);
        let input_search_rules = search_rules.1;
        let rh_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        rh_spacer.set_hexpand(true);
        r_header.append(&rh_spacer);
        let btn_syntax = gtk4::Button::new();
        btn_syntax.add_css_class("green-link");
        btn_syntax.set_label("Cú pháp quy tắc lọc");
        btn_syntax.connect_clicked(|_| {
            let _ = Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/ad-filtering/create-own-filters/")
                .spawn();
        });
        r_header.append(&btn_syntax);
        right.append(&r_header);

        let lbl_rules_header = gtk4::Label::new(Some("Quy tắc lọc"));
        lbl_rules_header.add_css_class("lbl-header-10");
        lbl_rules_header.set_halign(gtk4::Align::Start);
        right.append(&lbl_rules_header);

        let center_frame = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        center_frame.add_css_class("fe-center");
        center_frame.set_vexpand(true);

        // Mascot empty state
        let empty_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        empty_widget.set_vexpand(true);
        if let Some(tex) = crate::widgets::svg::svg_texture(&c::icons_dir().join("adguard_mascot.svg"), 110) {
            let img = gtk4::Image::new();
            img.set_paintable(Some(&tex));
            img.set_size_request(110, 110);
            img.set_halign(gtk4::Align::Center);
            empty_widget.append(&img);
        }
        let lbl_empty_msg = gtk4::Label::new(Some("Bộ lọc của người dùng đang trống"));
        lbl_empty_msg.add_css_class("lbl-empty-12");
        lbl_empty_msg.set_halign(gtk4::Align::Center);
        empty_widget.append(&lbl_empty_msg);

        let empty_btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        empty_btn_row.set_halign(gtk4::Align::Center);
        let btn_add_r = gtk4::Button::new();
        btn_add_r.add_css_class("fe-green-btn");
        btn_add_r.set_size_request(-1, 36);
        btn_add_r.set_label("Thêm quy tắc");
        empty_btn_row.append(&btn_add_r);
        let btn_imp_r = gtk4::Button::new();
        btn_imp_r.add_css_class("fe-import-btn");
        btn_imp_r.set_size_request(-1, 36);
        btn_imp_r.set_label("Nhập quy tắc");
        empty_btn_row.append(&btn_imp_r);
        empty_widget.append(&empty_btn_row);
        center_frame.append(&empty_widget);

        let rules_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        rules_container.set_margin_start(8);
        rules_container.set_margin_end(8);
        rules_container.set_margin_top(8);
        rules_container.set_margin_bottom(8);
        let rules_scroll = gtk4::ScrolledWindow::new();
        rules_scroll.set_child(Some(&rules_container));
        rules_scroll.set_vexpand(true);
        rules_scroll.set_visible(false);
        center_frame.append(&rules_scroll);
        right.append(&center_frame);

        // Right toolbar: [+] [−] [⚙] [✎]
        let right_bottom = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        right_bottom.set_margin_start(4);
        right_bottom.set_margin_end(4);
        let tb_add = tool_button("+");
        right_bottom.append(&tb_add);
        let tb_del = tool_button("−");
        right_bottom.append(&tb_del);
        let tb_opt = tool_button("⚙");
        tb_opt.set_tooltip_text(Some("Options"));
        right_bottom.append(&tb_opt);
        let tb_raw = tool_button("✎");
        tb_raw.set_tooltip_text(Some("Raw rules"));
        right_bottom.append(&tb_raw);
        right.append(&right_bottom);

        let inner = Rc::new(Inner {
            root,
            lbl_win_title,
            input_search_filters: input_search_filters.clone(),
            input_search_rules: input_search_rules.clone(),
            btn_syntax,
            lbl_rules_header,
            lbl_empty_msg,
            btn_add_r: btn_add_r.clone(),
            btn_imp_r: btn_imp_r.clone(),
            tb_add: tb_add.clone(),
            tb_del: tb_del.clone(),
            empty_widget,
            rules_scroll,
            left_container,
            rules_container,
            filter_items: RefCell::new(Vec::new()),
            user_rules: RefCell::new(Vec::new()),
            selected_item: RefCell::new(None),
        });
        let w = FilterEditorWindow { inner: inner.clone() };

        // Wire actions
        let w2 = w.clone();
        input_search_filters.connect_changed(move |e| w2.filter_left_list(&e.text()));
        let w3 = w.clone();
        input_search_rules.connect_changed(move |_e| {
            // Port of _filter_rules_list: currently a no-op in Python.
            let _ = &w3;
        });

        let w4 = w.clone();
        btn_add_r.connect_clicked(move |_| w4.add_rule());
        let w5 = w.clone();
        tb_add.connect_clicked(move |_| w5.add_rule());
        let w6 = w.clone();
        tb_del.connect_clicked(move |_| w6.delete_selected_rule());
        let w7 = w.clone();
        tb_raw.connect_clicked(move |_| w7.edit_raw_rules());
        let w8 = w.clone();
        btn_imp_r.connect_clicked(move |_| w8.edit_raw_rules());
        let w9 = w.clone();
        btn_add_f.connect_clicked(move |_| w9.add_custom_filter());

        w.load_user_rules();

        i18n::connect_language_changed(Box::new({
            let w10 = w.clone();
            move |_| w10.retranslate_ui()
        }));
        w.retranslate_ui();

        w
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        let i = &self.inner;
        let title = i18n::tr("fe_win_title");
        i.root.set_title(Some(&title));
        i.lbl_win_title.set_text(&title);
        i.input_search_filters
            .set_placeholder_text(Some(&i18n::tr("fe_search_filters")));
        i.input_search_rules
            .set_placeholder_text(Some(&i18n::tr("fe_search_rules")));
        i.btn_syntax.set_label(&i18n::tr("fe_syntax_btn"));
        i.lbl_rules_header.set_text(&i18n::tr("fe_filtering_rules"));
        i.lbl_empty_msg.set_text(&i18n::tr("fe_empty_rules"));
        i.btn_add_r.set_label(&i18n::tr("fe_add_rule_btn"));
        i.btn_imp_r.set_label(&i18n::tr("fe_import_rule_btn"));
        i.tb_add.set_tooltip_text(Some(&i18n::tr("fe_add_rule_btn")));
        i.tb_del.set_tooltip_text(Some(&i18n::tr("btn_delete")));
        self.populate_filters_from_db();
    }

    fn populate_filters_from_db(&self) {
        let i = &self.inner;
        let is_en = i18n::get_current_language() == i18n::LANG_EN;

        while let Some(child) = i.left_container.first_child() {
            i.left_container.remove(&child);
        }
        i.filter_items.borrow_mut().clear();

        let db_filters = filter_db::get_real_installed_filters();

        // Group by category name, preserving first-seen order
        let mut groups: Vec<(String, Vec<&filter_db::FilterInfo>)> = Vec::new();
        for f in &db_filters {
            match groups.iter_mut().find(|g| g.0 == f.group) {
                Some(g) => g.1.push(f),
                None => groups.push((f.group.clone(), vec![f])),
            }
        }

        let custom_header = i18n::tr("fe_custom_filter");
        i.left_container.append(&group_header(&custom_header));

        let count_u = filter_db::count_user_rules();
        let user_title = if is_en { "User filter" } else { "Bộ lọc của Người dùng" };
        let trusted_tag = i18n::tr("fe_trusted_tag");
        let user_item = FilterListItem::new(0, user_title, &format!("{} rules", count_u), &[trusted_tag], true);
        let w2 = self.clone();
        user_item.connect_selected(Box::new(move |fid| {
            w2.on_filter_selected(fid);
        }));
        i.left_container.append(user_item.root());
        i.filter_items.borrow_mut().push(user_item.clone());
        user_item.set_active(true);
        *i.selected_item.borrow_mut() = Some(user_item);

        for (grp_name, f_list) in &groups {
            if grp_name == "Bộ lọc tùy chỉnh" || grp_name == "Custom filter" {
                continue;
            }
            i.left_container.append(&group_header(grp_name));
            for f in f_list {
                if f.id == -2147483648 {
                    continue;
                }
                let mut sub_txt = format!("{} · {} rules", f.version, f.rules_count);
                if !f.updated.is_empty() {
                    let updated_label = if is_en { "updated" } else { "đã cập nhật" };
                    sub_txt = format!(
                        "{} {} {}\n{} rules",
                        f.version, updated_label, f.updated, f.rules_count
                    );
                }
                let item = FilterListItem::new(f.id, &f.title, &sub_txt, &f.tags, f.is_enabled);
                let w3 = self.clone();
                item.connect_selected(Box::new(move |fid| w3.on_filter_selected(fid)));
                let w4 = self.clone();
                item.connect_toggled(Box::new(move |fid, enabled| w4.on_filter_toggled(fid, enabled)));
                i.left_container.append(item.root());
                i.filter_items.borrow_mut().push(item);
            }
        }
    }

    fn load_user_rules(&self) {
        let i = &self.inner;
        *i.user_rules.borrow_mut() = filter_db::load_user_rules();

        while let Some(child) = i.rules_container.first_child() {
            i.rules_container.remove(&child);
        }

        let rules = i.user_rules.borrow().clone();
        if rules.is_empty() {
            i.empty_widget.set_visible(true);
            i.rules_scroll.set_visible(false);
        } else {
            i.empty_widget.set_visible(false);
            i.rules_scroll.set_visible(true);
            for (idx, r_text) in rules.iter().enumerate() {
                let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
                row.add_css_class("fe-rule-row");
                row.set_margin_start(6);
                row.set_margin_end(6);
                row.set_margin_top(4);
                row.set_margin_bottom(4);

                let cb = gtk4::CheckButton::new();
                cb.set_active(true);
                row.append(&cb);

                let lbl_num = gtk4::Label::new(Some(&(idx + 1).to_string()));
                lbl_num.add_css_class("lbl-num-9");
                row.append(&lbl_num);

                let lbl_r = gtk4::Label::new(Some(r_text));
                lbl_r.add_css_class("lbl-rule-10");
                lbl_r.set_hexpand(true);
                lbl_r.set_selectable(true);
                row.append(&lbl_r);

                i.rules_container.append(&row);
            }
        }
    }

    fn on_filter_selected(&self, filter_id: i64) {
        let i = &self.inner;
        for item in i.filter_items.borrow().iter() {
            item.set_active(item.filter_id() == filter_id);
        }
        let sel = i.filter_items.borrow().iter().find(|x| x.filter_id() == filter_id).cloned();
        *i.selected_item.borrow_mut() = sel;

        if filter_id == 0 {
            self.load_user_rules();
        } else {
            let is_en = i18n::get_current_language() == i18n::LANG_EN;
            i.empty_widget.set_visible(true);
            i.rules_scroll.set_visible(false);
            let active_label = if is_en { "is active" } else { "đang hoạt động" };
            let title = i
                .filter_items
                .borrow()
                .iter()
                .find(|x| x.filter_id() == filter_id)
                .map(|x| x.title())
                .unwrap_or_default();
            i.lbl_empty_msg.set_text(&format!("{} {}", title, active_label));
        }
    }

    fn on_filter_toggled(&self, filter_id: i64, is_enabled: bool) {
        crate::backend::toggle_filter(filter_id, is_enabled);
    }

    fn add_rule(&self) {
        let is_en = i18n::get_current_language() == i18n::LANG_EN;
        let title = if is_en { "Add rule" } else { "Thêm quy tắc" };
        let prompt = if is_en {
            "Enter filter rule (e.g. ||example.com^):"
        } else {
            "Nhập quy tắc lọc (ví dụ: ||example.com^):"
        };
        let w = self.clone();
        prompt_text(title, prompt, Box::new(move |rule: Option<String>| {
            if let Some(r) = rule {
                let r = r.trim().to_string();
                if !r.is_empty() {
                    w.inner.user_rules.borrow_mut().push(r);
                    let snapshot = w.inner.user_rules.borrow().clone();
                    filter_db::save_user_rules(&snapshot);
                    w.load_user_rules();
                }
            }
        }));
    }

    fn delete_selected_rule(&self) {
        let rules = self.inner.user_rules.borrow().clone();
        if rules.is_empty() {
            return;
        }
        let is_en = i18n::get_current_language() == i18n::LANG_EN;
        let title = if is_en { "Delete rule" } else { "Xóa quy tắc" };
        let prompt = if is_en { "Select rule to delete:" } else { "Chọn quy tắc cần xóa:" };
        let w = self.clone();
        prompt_choice(title, prompt, &rules, Box::new(move |picked: Option<String>| {
            if let Some(p) = picked {
                let mut cur = w.inner.user_rules.borrow_mut();
                if let Some(pos) = cur.iter().position(|r| *r == p) {
                    cur.remove(pos);
                }
                let snapshot = cur.clone();
                filter_db::save_user_rules(&snapshot);
                drop(cur);
                w.load_user_rules();
            }
        }));
    }

    fn edit_raw_rules(&self) {
        let is_en = i18n::get_current_language() == i18n::LANG_EN;
        let content = std::fs::read_to_string(filter_db::user_rules_path()).unwrap_or_default();

        let dlg = crate::util::sub_window("");
        dlg.set_default_size(600, 450);
        dlg.set_title(Some(if is_en {
            "Edit raw rules (user.txt)"
        } else {
            "Chỉnh sửa quy tắc lọc thô (user.txt)"
        }));
        dlg.add_css_class("dlg-dark");

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        dlg.set_child(Some(&col));
        col.append(&label_left(if is_en { "One rule per line:" } else { "Mỗi quy tắc trên một dòng:" }));

        let txt = gtk4::TextView::new();
        txt.add_css_class("fe-raw-view");
        txt.set_vexpand(true);
        txt.buffer().set_text(&content);
        col.append(&txt);

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let sp = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        sp.set_hexpand(true);
        btn_row.append(&sp);
        let btn_save = gtk4::Button::new();
        btn_save.add_css_class("green-btn");
        btn_save.set_label(&i18n::tr("btn_save"));
        let btn_cancel = gtk4::Button::new();
        btn_cancel.add_css_class("outline-btn");
        btn_cancel.set_label(&i18n::tr("btn_cancel"));

        let w = self.clone();
        let buf = txt.buffer().clone();
        let d1 = dlg.clone();
        btn_save.connect_clicked(move |_| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
            let _ = std::fs::write(filter_db::user_rules_path(), text.as_str());
            w.load_user_rules();
            d1.set_visible(false);
        });
        btn_cancel.connect_clicked({
            let d2 = dlg.clone();
            move |_| d2.set_visible(false)
        });
        btn_row.append(&btn_save);
        btn_row.append(&btn_cancel);
        col.append(&btn_row);

        dlg.present();
    }

    fn add_custom_filter(&self) {
        let is_en = i18n::get_current_language() == i18n::LANG_EN;
        let title = if is_en { "Add custom filter" } else { "Thêm bộ lọc tùy chỉnh" };
        let prompt = if is_en {
            "Enter filter URL or file path:"
        } else {
            "Nhập URL hoặc đường dẫn file bộ lọc:"
        };
        let w = self.clone();
        prompt_text(title, prompt, Box::new(move |url: Option<String>| {
            if let Some(u) = url {
                let u = u.trim().to_string();
                if !u.is_empty() {
                    let _ = Command::new("adguard-cli")
                        .args(["filters", "install", &u])
                        .output();
                    let info_title = if is_en { "AdGuard" } else { "Thông báo" };
                    let info_msg = if is_en {
                        "Custom filter installation request sent!"
                    } else {
                        "Đã gửi lệnh cài đặt bộ lọc tùy chỉnh!"
                    };
                    show_message(&w.inner.root, &info_title, &info_msg);
                }
            }
        }));
    }

    fn filter_left_list(&self, text: &str) {
        let q = text.trim().to_lowercase();
        for item in self.inner.filter_items.borrow().iter() {
            let show = q.is_empty() || item.title().to_lowercase().contains(&q);
            item.root().set_visible(show);
        }
    }
}

fn search_box(placeholder: &str) -> (gtk4::Box, gtk4::Entry) {
    let frame = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    frame.add_css_class("fe-search");
    frame.set_margin_start(8);
    frame.set_margin_end(8);
    frame.set_margin_top(4);
    frame.set_margin_bottom(4);
    frame.append(&gtk4::Label::new(Some("🔍")));
    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some(placeholder));
    entry.add_css_class("search-entry");
    frame.append(&entry);
    (frame, entry)
}

fn tool_button(label: &str) -> gtk4::Button {
    let b = gtk4::Button::new();
    b.add_css_class("fe-tool-btn");
    b.set_size_request(28, 28);
    b.set_label(label);
    b
}

fn group_header(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("lbl-group-9");
    l.set_halign(gtk4::Align::Start);
    l.set_margin_top(8);
    l
}

fn label_left(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("lbl-white-10");
    l.set_halign(gtk4::Align::Start);
    l
}

/// QInputDialog.getText equivalent: a small entry dialog with a callback.
fn prompt_text(title: &str, prompt: &str, on_ok: Box<dyn Fn(Option<String>)>) {
    let dlg = crate::util::sub_window("");
    dlg.set_default_size(420, 140);
    dlg.set_title(Some(title));
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    dlg.set_child(Some(&col));
    col.append(&label_left(prompt));
    let entry = gtk4::Entry::new();
    entry.add_css_class("input-dark");
    col.append(&entry);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let ok = gtk4::Button::new();
    ok.add_css_class("green-btn");
    ok.set_label("OK");
    let cancel = gtk4::Button::new();
    cancel.add_css_class("outline-btn");
    cancel.set_label(&i18n::tr("btn_cancel"));
    let e2 = entry.clone();
    let d2 = dlg.clone();
    ok.connect_clicked(move |_| {
        on_ok(Some(e2.text().to_string()));
        d2.set_visible(false);
    });
    let d3 = dlg.clone();
    cancel.connect_clicked(move |_| {
        let _ = &d3;
        d3.set_visible(false);
    });
    row.append(&ok);
    row.append(&cancel);
    col.append(&row);
    dlg.present();
}

/// QInputDialog.getItem equivalent: pick one of the given items.
fn prompt_choice(title: &str, prompt: &str, items: &[String], on_ok: Box<dyn Fn(Option<String>)>) {
    let dlg = crate::util::sub_window("");
    dlg.set_default_size(420, 300);
    dlg.set_title(Some(title));
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    dlg.set_child(Some(&col));
    col.append(&label_left(prompt));

    let list = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let chosen: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    for it in items {
        let b = gtk4::Button::new();
        b.add_css_class("opt-row");
        b.set_label(it);
        let c2 = chosen.clone();
        let v = it.clone();
        b.connect_clicked(move |_| *c2.borrow_mut() = Some(v.clone()));
        list.append(&b);
    }
    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_child(Some(&list));
    scroll.set_vexpand(true);
    col.append(&scroll);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let ok = gtk4::Button::new();
    ok.add_css_class("green-btn");
    ok.set_label("OK");
    let c3 = chosen.clone();
    let d2 = dlg.clone();
    ok.connect_clicked(move |_| {
        on_ok(c3.borrow().clone());
        d2.set_visible(false);
    });
    let d3 = dlg.clone();
    row.append(&ok);
    col.append(&row);
    dlg.present();
}

fn show_message(parent: &gtk4::Window, title: &str, body: &str) {
    let dlg = crate::util::sub_window(title);
    dlg.set_default_size(360, 120);
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    dlg.set_child(Some(&col));
    let l = gtk4::Label::new(Some(body));
    l.set_wrap(true);
    col.append(&l);
    let ok = gtk4::Button::new();
    ok.add_css_class("green-btn");
    ok.set_label("OK");
    let d2 = dlg.clone();
    ok.connect_clicked(move |_| d2.set_visible(false));
    col.append(&ok);
    let _ = parent;
    dlg.present();
}
