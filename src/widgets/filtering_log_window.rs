// Port of adguard_modern/widgets/filtering_log_window.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;
use gtk4::glib;
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use crate::constants as c;
use crate::i18n;
use crate::widgets::filtering_log_inspector::{FilteringLogInspector, LogEntry};
use crate::widgets::svg;


pub fn get_active_access_log() -> PathBuf {
    let root_log = PathBuf::from("/root/.local/share/adguard-cli/logs/access.log");
    let user_log = c::home_dir().join(".local/share/adguard-cli/logs/access.log");
    if root_log.exists() && is_readable(&root_log) {
        if !user_log.exists() {
            return root_log;
        }
        match (file_mtime(&root_log), file_mtime(&user_log)) {
            (Some(r), Some(u)) if r >= u => return root_log,
            (Some(_), Some(_)) => {}
            _ => return root_log,
        }
    }
    user_log
}

fn is_readable(path: &Path) -> bool {
    fs::File::open(path).is_ok()
}

fn file_mtime(path: &Path) -> Option<i64> {
    let m = fs::metadata(path).ok()?;
    let t = m.modified().ok()?;
    Some(t.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64)
}

struct Inner {
    root: gtk4::Window,
    lbl_title: gtk4::Label,
    btn_export: gtk4::Button,
    btn_import: gtk4::Button,
    btn_clear: gtk4::Button,
    input_search: gtk4::Entry,
    lbl_empty: gtk4::Label,
    empty_widget: gtk4::Box,
    table: gtk4::ListBox,
    table_scroll: gtk4::ScrolledWindow,
    header_labels: Vec<gtk4::Label>,
    inspector: FilteringLogInspector,
    current_tab: RefCell<String>,
    all_entries: RefCell<Vec<LogEntry>>,
    rendered: RefCell<Vec<LogEntry>>,
    row_index: RefCell<HashMap<usize, usize>>,
    tab_buttons: RefCell<HashMap<String, (gtk4::Button, String)>>,
    last_mtime: RefCell<i64>,
    last_size: RefCell<i64>,
}

#[derive(Clone)]
pub struct FilteringLogDialog {
    inner: Rc<Inner>,
}

impl FilteringLogDialog {
    pub fn new() -> FilteringLogDialog {
        let root = crate::util::sub_window("Filtering log");
        root.set_default_size(1180, 700);
        root.set_title(Some("Nhật ký lọc"));
        root.add_css_class("flog-window");

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_child(Some(&col));

        // Header title bar
        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        header.add_css_class("flog-header");
        header.set_size_request(-1, 40);
        header.set_margin_start(16);
        header.set_margin_end(16);
        let lbl_title = gtk4::Label::new(Some("Nhật ký lọc"));
        lbl_title.add_css_class("flog-title");
        lbl_title.set_halign(gtk4::Align::Start);
        header.append(&lbl_title);
        let hstretch = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        hstretch.set_hexpand(true);
        header.append(&hstretch);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(26, 26);
        btn_close.set_label("✕");
        let r2 = root.clone();
        btn_close.connect_clicked(move |_| r2.set_visible(false));
        header.append(&btn_close);
        col.append(&header);

        // Tabs bar + top right actions
        let sub_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        sub_bar.add_css_class("flog-subbar");
        sub_bar.set_size_request(-1, 38);
        sub_bar.set_margin_start(16);
        sub_bar.set_margin_end(16);
        col.append(&sub_bar);

        let tab_names: [(&str, &str); 10] = [
            ("ALL", "flog_tab_all"),
            ("HTML", "HTML"),
            ("CSS", "CSS"),
            ("Javascript", "Javascript"),
            ("Ajax", "Ajax"),
            ("Image", "Image"),
            ("Media", "Media"),
            ("DNS", "DNS"),
            ("Connection", "Connection"),
            ("Other", "flog_tab_other"),
        ];

        let inner: Rc<Inner> = Rc::new(Inner {
            root: root.clone(),
            lbl_title: lbl_title.clone(),
            btn_export: gtk4::Button::new(),
            btn_import: gtk4::Button::new(),
            btn_clear: gtk4::Button::new(),
            input_search: gtk4::Entry::new(),
            lbl_empty: gtk4::Label::new(Some("Nhật ký lọc đang trống")),
            empty_widget: gtk4::Box::new(gtk4::Orientation::Vertical, 14),
            table: gtk4::ListBox::new(),
            table_scroll: gtk4::ScrolledWindow::new(),
            header_labels: Vec::new(),
            inspector: FilteringLogInspector::new(),
            current_tab: RefCell::new("ALL".to_string()),
            all_entries: RefCell::new(Vec::new()),
            rendered: RefCell::new(Vec::new()),
            row_index: RefCell::new(HashMap::new()),
            tab_buttons: RefCell::new(HashMap::new()),
            last_mtime: RefCell::new(0),
            last_size: RefCell::new(0),
        });

        for (key, i18n_k) in tab_names {
            let txt = if i18n_k.contains('_') { i18n::tr(i18n_k) } else { i18n_k.to_string() };
            let btn = gtk4::Button::new();
            btn.add_css_class("flog-tab");
            if key == "ALL" {
                btn.add_css_class("active");
            }
            btn.set_label(&txt);
            let i2 = inner.clone();
            let k = key.to_string();
            btn.connect_clicked(move |_| on_tab_clicked(&i2, &k));
            sub_bar.append(&btn);
            inner.tab_buttons.borrow_mut().insert(key.to_string(), (btn, i18n_k.to_string()));
        }

        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        sub_bar.append(&spacer);

        let btn_export = inner.btn_export.clone();
        btn_export.add_css_class("flog-action-green");
        btn_export.set_label("Xuất tất cả...");
        let i2 = inner.clone();
        btn_export.connect_clicked(move |_| export_logs(&i2));
        sub_bar.append(&btn_export);

        let btn_import = inner.btn_import.clone();
        btn_import.add_css_class("flog-action-green");
        btn_import.set_label("Nhập tất cả...");
        sub_bar.append(&btn_import);

        let btn_clear = inner.btn_clear.clone();
        btn_clear.add_css_class("flog-action-red");
        btn_clear.set_label("Xoá nhật ký");
        let i2 = inner.clone();
        btn_clear.connect_clicked(move |_| clear_logs(&i2));
        sub_bar.append(&btn_clear);

        // Search row
        let search_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_row.add_css_class("flog-searchrow");
        search_row.set_size_request(-1, 42);
        search_row.set_margin_start(16);
        search_row.set_margin_end(16);
        search_row.set_margin_top(4);
        search_row.set_margin_bottom(4);
        col.append(&search_row);

        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        search_box.add_css_class("flog-searchbox");
        search_box.set_size_request(300, -1);
        search_box.set_margin_start(8);
        search_box.set_margin_end(8);
        search_box.set_margin_top(2);
        search_box.set_margin_bottom(2);
        let emoji = gtk4::Label::new(Some("🔍"));
        emoji.add_css_class("flog-emoji");
        search_box.append(&emoji);
        let input_search = inner.input_search.clone();
        input_search.add_css_class("search-entry");
        input_search.set_hexpand(true);
        input_search.set_placeholder_text(Some("Tìm kiếm nhật ký"));
        let i2 = inner.clone();
        input_search.connect_changed(move |_| render_table(&i2));
        search_box.append(&input_search);
        search_row.append(&search_box);

        let row_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        row_spacer.set_hexpand(true);
        search_row.append(&row_spacer);

        let btn_filter_cfg = gtk4::Button::new();
        btn_filter_cfg.add_css_class("flog-gear");
        btn_filter_cfg.set_size_request(28, 28);
        btn_filter_cfg.set_label("⚙");
        search_row.append(&btn_filter_cfg);

        // Center area: table on the left, inspector on the right
        let center = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        center.add_css_class("flog-center");
        center.set_vexpand(true);
        col.append(&center);

        let left_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        left_container.set_hexpand(true);
        center.append(&left_container);

        let empty_widget = inner.empty_widget.clone();
        empty_widget.set_halign(gtk4::Align::Center);
        empty_widget.set_valign(gtk4::Align::Center);
        empty_widget.set_vexpand(true);
        let mascot = svg::svg_image("adguard_mascot.svg", 110);
        mascot.set_halign(gtk4::Align::Center);
        empty_widget.append(&mascot);
        let lbl_empty = inner.lbl_empty.clone();
        lbl_empty.add_css_class("flog-empty");
        lbl_empty.set_halign(gtk4::Align::Center);
        empty_widget.append(&lbl_empty);
        left_container.append(&empty_widget);

        let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        header_row.add_css_class("flog-headrow");
        header_row.set_margin_start(10);
        header_row.set_margin_end(10);
        header_row.set_margin_top(6);
        header_row.set_margin_bottom(6);
        left_container.append(&header_row);
        let mut header_labels = Vec::new();
        for (i, key) in [
            "flog_col_name",
            "flog_col_status",
            "flog_col_rule",
            "flog_col_filter",
            "flog_col_type",
            "flog_col_source",
            "flog_col_process",
        ]
        .iter()
        .enumerate()
        {
            let l = gtk4::Label::new(Some(&i18n::tr(key)));
            l.add_css_class("flog-head");
            l.set_halign(gtk4::Align::Start);
            if i == 0 {
                l.set_width_chars(40);
            }
            if i == 5 {
                l.set_width_chars(20);
            }
            header_row.append(&l);
            header_labels.push(l);
        }

        let table = inner.table.clone();
        table.add_css_class("flog-table");
        let table_scroll = inner.table_scroll.clone();
        table_scroll.set_child(Some(&table));
        table_scroll.set_vexpand(true);
        left_container.append(&table_scroll);

        let i2 = inner.clone();
        table.connect_row_selected(move |listbox, row| {
            if let Some(row) = row {
                let idx = i2.row_index.borrow().get(&(row.as_ptr() as usize)).copied();
                if let Some(idx) = idx {
                    let entry = i2.rendered.borrow()[idx].clone();
                    i2.inspector.set_entry(entry);
                    i2.inspector.root().set_visible(true);
                }
            }
            let _ = listbox;
        });

        let inspector = inner.inspector.clone();
        inspector.root().set_visible(false);
        center.append(inspector.root());
        let i2 = inner.clone();
        inspector.connect_closed(Box::new(move || i2.inspector.root().set_visible(false)));
        let i2 = inner.clone();
        inspector.connect_block_rule_requested(Box::new(move |target| block_rule(&i2, target)));

        load_live_requests(&inner);

        i18n::connect_language_changed(Box::new({
            let i3 = inner.clone();
            move |_| retranslate_ui(&i3)
        }));
        retranslate_ui(&inner);

        // Real-time auto-refresh timer (every 2 seconds)
        let i2 = inner.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(2000), move || {
            load_live_requests(&i2);
            glib::ControlFlow::Continue
        });

        FilteringLogDialog { inner }
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    /// Python: `dlg.input_search.setText(name)` before `dlg.exec()`.
    pub fn set_search(&self, text: &str) {
        self.inner.input_search.set_text(text);
        render_table(&self.inner);
    }
}

fn retranslate_ui(inner: &Inner) {
    inner.root.set_title(Some(&i18n::tr("flog_win_title")));
    inner.lbl_title.set_text(&i18n::tr("flog_win_title"));
    inner.btn_export.set_label(&i18n::tr("flog_export_all"));
    inner.btn_import.set_label(&i18n::tr("flog_import_all"));
    inner.btn_clear.set_label(&i18n::tr("flog_clear_btn"));
    inner.input_search.set_placeholder_text(Some(&i18n::tr("flog_search_placeholder")));
    inner.lbl_empty.set_text(&i18n::tr("flog_empty"));
    for (_k, (btn, i18n_k)) in inner.tab_buttons.borrow().iter() {
        let txt = if i18n_k.contains('_') { i18n::tr(i18n_k) } else { i18n_k.clone() };
        btn.set_label(&txt);
    }
    for (i, l) in inner.header_labels.iter().enumerate() {
        let key = [
            "flog_col_name",
            "flog_col_status",
            "flog_col_rule",
            "flog_col_filter",
            "flog_col_type",
            "flog_col_source",
            "flog_col_process",
        ][i];
        l.set_text(&i18n::tr(key));
    }
    inner.inspector.retranslate_ui();
    render_table(inner);
}

fn on_tab_clicked(inner: &Inner, key: &str) {
    *inner.current_tab.borrow_mut() = key.to_string();
    for (k, (btn, _)) in inner.tab_buttons.borrow().iter() {
        if k == key {
            btn.add_css_class("active");
        } else {
            btn.remove_css_class("active");
        }
    }
    render_table(inner);
}

fn filtered_entries(inner: &Inner) -> Vec<LogEntry> {
    let query = inner.input_search.text().trim().to_lowercase();
    let tab = inner.current_tab.borrow().clone();
    let mut out = Vec::new();
    for e in inner.all_entries.borrow().iter() {
        if tab != "ALL" && e.r#type != tab {
            continue;
        }
        if !query.is_empty()
            && !e.name.to_lowercase().contains(&query)
            && !e.source.to_lowercase().contains(&query)
            && !e.status.to_lowercase().contains(&query)
        {
            continue;
        }
        out.push(e.clone());
    }
    out
}

fn render_table(inner: &Inner) {
    let filtered = filtered_entries(inner);
    if filtered.is_empty() {
        inner.empty_widget.set_visible(true);
        inner.table_scroll.set_visible(false);
        return;
    }
    inner.empty_widget.set_visible(false);
    inner.table_scroll.set_visible(true);

    let vadj = inner.table_scroll.vadjustment();
    let old_scroll = vadj.value();

    while let Some(row) = inner.table.first_child() {
        inner.table.remove(&row);
    }
    inner.row_index.borrow_mut().clear();

    let rev: Vec<LogEntry> = filtered.iter().rev().cloned().collect();
    inner.rendered.replace(rev.clone());

    for (row, item) in rev.iter().enumerate() {
        let lrow = gtk4::ListBoxRow::new();
        lrow.set_selectable(true);
        let cells = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        cells.set_margin_start(10);
        cells.set_margin_end(10);
        cells.set_margin_top(5);
        cells.set_margin_bottom(5);

        cells.append(&cell(&item.name, "flog-cell-name", 40));

        let status_class = if item.status == "Đã chặn" { "c-red" } else { "c-green" };
        cells.append(&cell(&item.status, &format!("flog-cell-status {}", status_class), -1));

        let rule_class = if item.rule == "Stealth mode" {
            "c-muted"
        } else if item.rule.contains("áp dụng") || item.rule.contains("applied") {
            "c-green"
        } else if item.rule == "AdGuard Base" || item.rule.contains("Chặn") {
            "c-red"
        } else {
            ""
        };
        cells.append(&cell(&item.rule, &format!("flog-cell {}", rule_class), -1));

        cells.append(&cell(&item.filter, "flog-cell", -1));

        let type_class = match item.r#type.as_str() {
            "Image" => "c-orange",
            "Ajax" => "c-blue",
            "Media" => "c-purple",
            "HTML" => "c-sky",
            _ => "",
        };
        cells.append(&cell(&item.r#type, &format!("flog-cell {}", type_class), -1));

        cells.append(&cell(&item.source, "flog-cell", 20));
        cells.append(&cell(&item.proc, "flog-cell-proc", -1));

        lrow.set_child(Some(&cells));
        inner.row_index.borrow_mut().insert(lrow.as_ptr() as usize, row);
        inner.table.append(&lrow);
    }

    if old_scroll > 0.0 && old_scroll < vadj.upper() {
        vadj.set_value(old_scroll);
    }
}

fn cell(text: &str, classes: &str, width_chars: i32) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    for class in classes.split(' ') {
        if !class.is_empty() {
            l.add_css_class(class);
        }
    }
    l.set_halign(gtk4::Align::Start);
    if width_chars > 0 {
        l.set_width_chars(width_chars);
    }
    l
}

fn load_live_requests(inner: &Inner) {
    let log_path = get_active_access_log();
    if !log_path.exists() {
        return;
    }

    let mtime = match file_mtime(&log_path) {
        Some(m) => m,
        None => return,
    };
    let size = match fs::metadata(&log_path) {
        Ok(m) => m.len() as i64,
        Err(_) => return,
    };
    if *inner.last_mtime.borrow() == mtime && *inner.last_size.borrow() == size {
        return;
    }
    *inner.last_mtime.borrow_mut() = mtime;
    *inner.last_size.borrow_mut() = size;

    let mut f = match fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    if size > 40000 {
        let _ = f.seek(SeekFrom::Start((size - 40000) as u64));
    }
    let mut raw = String::new();
    if f.read_to_string(&mut raw).is_err() {
        return;
    }
    let lines: Vec<String> = raw.lines().map(|l| l.to_string()).collect();
    let lines: Vec<String> = if size > 40000 { if lines.is_empty() { Vec::new() } else { lines[1..].to_vec() } } else { lines };

    let window = &lines[lines.len().saturating_sub(100)..];
    let mut new_entries: Vec<LogEntry> = Vec::new();
    for line in window {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }
        // Skip HTTP1 CONNECT proxy tunnel establishment lines
        if parts.len() > 4 && parts[3] == "HTTP1" && parts[4] == "CONNECT" {
            continue;
        }
        let protocol = if parts.len() > 3 { parts[3] } else { "" };

        let mut raw_target: Option<&str> = None;
        for p in &parts[3..] {
            if matches!(
                *p,
                "TLS" | "HTTP1" | "TCP" | "CONNECT" | "GET" | "POST" | "any" | "NONE" | "--" | "-"
            ) {
                continue;
            }
            if p.ends_with("ms") || p.ends_with('b') || !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if (p.contains('.') || p.contains("http")) && !p.starts_with('[') {
                raw_target = Some(*p);
                break;
            }
        }
        let raw_target = match raw_target {
            Some(t) if t != "-" && !t.starts_with("internal") => t,
            _ => continue,
        };

        let url_low = raw_target.to_lowercase();
        let line_low = line.to_lowercase();
        let mut req_type = "Connection".to_string();
        let display_url;

        if url_low.contains("googlevideo.com") {
            display_url = format!("https://{}/videoplayback?expire=...&ei=...", raw_target);
            req_type = "Media".to_string();
        } else if url_low.contains("ytimg.com") {
            display_url = format!("https://{}/vi/hqdefault.jpg?sqp=...", raw_target);
            req_type = "Image".to_string();
        } else if url_low.contains("ggpht.com") {
            display_url = format!("https://{}/ytc/avatar=s48-c-k...", raw_target);
            req_type = "Image".to_string();
        } else if url_low.contains("youtube.com") {
            display_url = format!("https://{}/watch?v=...", raw_target);
            req_type = "HTML".to_string();
        } else if url_low.contains("sync-v2") || url_low.contains("brave.com") {
            display_url = if raw_target.contains("instances") {
                raw_target.to_string()
            } else {
                format!("https://{}/instances/typical/randomness", raw_target)
            };
            req_type = "Ajax".to_string();
        } else if line_low.contains("dns") || protocol == "DNS" {
            display_url = format!("dns://{}", raw_target);
            req_type = "DNS".to_string();
        } else if protocol == "TCP" || line_low.contains("connect") || raw_target.contains(':') {
            display_url = if raw_target.starts_with("tcp://") {
                raw_target.to_string()
            } else {
                format!("tcp://{}", raw_target)
            };
            req_type = "Connection".to_string();
        } else if raw_target.starts_with("http://") || raw_target.starts_with("https://") {
            display_url = raw_target.to_string();
            if [".png", ".jpg", ".jpeg", ".svg", ".webp", ".gif", ".ico"]
                .iter()
                .any(|e| url_low.contains(e))
            {
                req_type = "Image".to_string();
            } else if [".mp4", ".mp3", ".webm", ".ogg"].iter().any(|e| url_low.contains(e)) {
                req_type = "Media".to_string();
            } else if url_low.contains(".js") {
                req_type = "Javascript".to_string();
            } else if url_low.contains(".css") {
                req_type = "CSS".to_string();
            } else if line_low.contains("xhr") || line_low.contains("ajax") {
                req_type = "Ajax".to_string();
            } else {
                req_type = "HTML".to_string();
            }
        } else {
            display_url = format!("https://{}/", raw_target);
            req_type = "Connection".to_string();
        }

        let is_blocked = ["502", "BLOCK", "REJECT", "403"].iter().any(|kw| line.contains(kw));
        let status = if is_blocked { "Đã chặn".to_string() } else { "Được phép".to_string() };
        let mut rule = "-".to_string();
        let mut filter_name = String::new();
        if is_blocked {
            rule = "Chặn quảng cáo".to_string();
            filter_name = "AdGuard Base".to_string();
        } else if line.contains("MODIFIED_CONTENT") {
            rule = "7 rules applied".to_string();
            filter_name = "Một số bộ lọc".to_string();
        } else if line.contains("MODIFIED_META")
            || url_low.contains("youtube")
            || url_low.contains("googlevideo")
            || url_low.contains("ytimg")
            || url_low.contains("ggpht")
        {
            rule = "Stealth mode".to_string();
        }

        let mut source = "-".to_string();
        if ["youtube", "googlevideo", "ytimg", "ggpht"].iter().any(|k| display_url.contains(k)) {
            source = "https://www.youtube.com/".to_string();
        } else if display_url.starts_with("http") {
            source = display_url.split('?').next().unwrap_or("").to_string();
            if source.len() > 40 {
                source = format!("{}...", &source[..37]);
            }
        }

        let mut proc = "brave".to_string();
        if line_low.contains("firefox") {
            proc = "firefox".to_string();
        } else if line_low.contains("mullvad") {
            proc = "mullvad-browser".to_string();
        }

        new_entries.push(LogEntry {
            name: display_url,
            status,
            rule,
            filter: filter_name,
            r#type: req_type,
            source,
            proc,
        });
    }

    let take = new_entries.len().min(100);
    inner.all_entries.replace(new_entries[new_entries.len() - take..].to_vec());
    render_table(inner);
}

fn clear_logs(inner: &Rc<Inner>) {
    let title = i18n::tr("flog_clear_confirm_title");
    let msg = i18n::tr("flog_clear_confirm_msg");
    let parent = inner.root.clone();
    let i2 = inner.clone();
    ask_yes_no(&parent, &title, &msg, move |yes: bool| {
        if !yes {
            return;
        }
        let log_path = get_active_access_log();
        if log_path.exists() {
            let _ = fs::write(&log_path, "");
        }
        i2.all_entries.borrow_mut().clear();
        *i2.last_size.borrow_mut() = 0;
        *i2.last_mtime.borrow_mut() = 0;
        render_table(&i2);
    });
}

fn export_logs(inner: &Inner) {
    let is_en = i18n::get_current_language() == i18n::LANG_EN;
    let parent = inner.root.clone();
    let dlg = gtk4::FileChooserDialog::new(
        Some(&i18n::tr("flog_export_all")),
        Some(&parent),
        gtk4::FileChooserAction::Save,
        &[
            (&i18n::tr("btn_cancel"), gtk4::ResponseType::Cancel),
            ("Save", gtk4::ResponseType::Accept),
        ],
    );
    dlg.set_current_name("adguard_access_log.txt");
    let entries = inner.all_entries.borrow().clone();
    let p2 = parent.clone();
    dlg.connect_response(move |d, response| {
        if response != gtk4::ResponseType::Accept {
            return;
        }
        let path = match d.file().and_then(|f| f.path()) {
            Some(p) => p,
            None => return,
        };
        let mut body = String::new();
        for e in &entries {
            body.push_str(&format!(
                "{} | {} | {} | {} | {} | {}\n",
                e.name, e.rule, e.filter, e.r#type, e.source, e.proc
            ));
        }
        match fs::write(&path, body) {
            Ok(()) => {
                let title = if is_en { "AdGuard" } else { "Thông báo" };
                let msg = if is_en {
                    "Logs exported successfully!"
                } else {
                    "Đã xuất nhật ký thành công!"
                };
                show_message(Some(&p2), title, msg);
            }
            Err(ex) => {
                let title = if is_en { "Error" } else { "Lỗi" };
                show_message(Some(&p2), title, &ex.to_string());
            }
        }
    });
    dlg.present();
}

fn block_rule(inner: &Inner, target_url: &str) {
    let is_en = i18n::get_current_language() == i18n::LANG_EN;
    let clean = if target_url.contains("://") {
        target_url.split("://").last().unwrap_or("").split('/').next().unwrap_or("")
    } else {
        target_url
    };
    let clean = clean.split(':').next().unwrap_or(clean);
    let rule = format!("||{}^", clean);

    let user_txt = c::home_dir().join(".local/share/adguard-cli/user.txt");
    match fs::OpenOptions::new().append(true).create(true).open(&user_txt) {
        Ok(mut f) => {
            use std::io::Write;
            let _ = writeln!(f, "\n{}\n", rule);
            let title = if is_en { "Rule added successfully" } else { "Thêm quy tắc thành công" };
            let msg = if is_en {
                format!("Rule added to User filter:\n{}", rule)
            } else {
                format!("Đã thêm quy tắc vào Danh sách người dùng:\n{}", rule)
            };
            show_message(Some(&inner.root), title, &msg);
        }
        Err(ex) => {
            let title = if is_en { "Error" } else { "Lỗi" };
            show_message(Some(&inner.root), title, &ex.to_string());
        }
    }
}

pub fn show_message(parent: Option<&gtk4::Window>, title: &str, body: &str) {
    let dlg = crate::util::sub_window(title);
    if let Some(p) = parent {
        dlg.set_transient_for(Some(p));
    }
    dlg.set_default_size(360, 120);
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    dlg.set_child(Some(&col));
    let l = gtk4::Label::new(Some(body));
    l.set_wrap(true);
    l.set_halign(gtk4::Align::Start);
    col.append(&l);
    let ok = gtk4::Button::new();
    ok.add_css_class("green-btn");
    ok.set_label("OK");
    let d2 = dlg.clone();
    ok.connect_clicked(move |_| d2.set_visible(false));
    col.append(&ok);
    dlg.present();
}

fn ask_yes_no(parent: &gtk4::Window, title: &str, body: &str, on_answer: impl Fn(bool) + 'static) {
    let dlg = crate::util::sub_window(title);
    dlg.set_transient_for(Some(parent));
    dlg.set_default_size(380, 140);
    dlg.add_css_class("dlg-dark");
    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    dlg.set_child(Some(&col));
    let l = gtk4::Label::new(Some(body));
    l.set_wrap(true);
    l.set_halign(gtk4::Align::Start);
    col.append(&l);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let yes = gtk4::Button::new();
    yes.add_css_class("green-btn");
    yes.set_label("Yes");
    let no = gtk4::Button::new();
    no.add_css_class("outline-btn");
    no.set_label("No");

    let answer: Rc<dyn Fn(bool)> = Rc::new(on_answer);
    let d2 = dlg.clone();
    let a2 = answer.clone();
    yes.connect_clicked(move |_| {
        a2(true);
        d2.set_visible(false);
    });
    let d3 = dlg.clone();
    let a3 = answer.clone();
    no.connect_clicked(move |_| {
        a3(false);
        d3.set_visible(false);
    });

    row.append(&yes);
    row.append(&no);
    col.append(&row);
    dlg.present();
}
