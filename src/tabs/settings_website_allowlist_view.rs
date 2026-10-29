// Port of adguard_modern/tabs/settings_website_allowlist_view.py — the HTTPS
// website allowlist subview (real exclusions file, search, pagination, add/remove).
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

const ITEMS_PER_PAGE: usize = 8;

fn exclusions_file() -> PathBuf {
    c::home_dir().join(".local/share/adguard-cli/https_exclusions.txt")
}

fn root_exclusions_file() -> PathBuf {
    PathBuf::from("/root/.local/share/adguard-cli/https_exclusions.txt")
}

fn exclusions_bak() -> PathBuf {
    c::home_dir().join(".local/share/adguard-cli/https_exclusions.txt.bak")
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    lbl_sub: gtk4::Label,
    switch_master: ToggleSwitch,
    input_search: gtk4::Entry,
    add_btn: gtk4::Button,
    list_container: gtk4::Box,
    pag_container: gtk4::Box,
    all_domains: RefCell<Vec<String>>,
    custom_domains: RefCell<HashSet<String>>,
    search_query: RefCell<String>,
    current_page: RefCell<usize>,
    filtered_domains: RefCell<Vec<String>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct SettingsWebsiteAllowlistView {
    inner: Rc<Inner>,
}

impl SettingsWebsiteAllowlistView {
    pub fn new() -> SettingsWebsiteAllowlistView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(20);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top_bar.append(&spacer);

        let btn_more = gtk4::Button::new();
        btn_more.add_css_class("flat-icon-btn");
        btn_more.set_size_request(30, 30);
        if c::icons_dir().join("stat_more.svg").exists() {
            btn_more.set_child(Some(&svg::svg_image("stat_more.svg", 16)));
        }
        top_bar.append(&btn_more);
        root.append(&top_bar);
        root.append(&spacer_px(20));

        let head_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        title_box.set_hexpand(true);
        let lbl_title = gtk4::Label::new(Some("Website allowlist"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        title_box.append(&lbl_title);
        let lbl_sub = gtk4::Label::new(Some("HTTPS traffic isn’t filtered for 0 websites"));
        lbl_sub.add_css_class("lbl-sub-11");
        lbl_sub.set_halign(gtk4::Align::Start);
        title_box.append(&lbl_sub);
        head_row.append(&title_box);
        let head_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        head_spacer.set_hexpand(true);
        head_row.append(&head_spacer);

        let switch_master = ToggleSwitch::new(46, 26);
        switch_master.set_checked(true, false);
        head_row.append(switch_master.root());
        root.append(&head_row);
        root.append(&spacer_px(20));

        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_box.add_css_class("search-box");
        search_box.set_size_request(-1, 42);
        if c::icons_dir().join("search.svg").exists() {
            search_box.append(&svg::svg_image("search.svg", 16));
        }
        let input_search = gtk4::Entry::new();
        input_search.set_placeholder_text(Some("Search"));
        input_search.add_css_class("search-input");
        input_search.set_hexpand(true);
        search_box.append(&input_search);
        root.append(&search_box);
        root.append(&spacer_px(16));

        let add_btn = gtk4::Button::new();
        add_btn.set_label("+  Add website");
        add_btn.add_css_class("ev-add-btn");
        root.append(&add_btn);
        root.append(&spacer_px(10));

        let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        root.append(&list_container);

        let pag_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        pag_container.set_margin_top(12);
        root.append(&pag_container);

        let bb_root = btn_back.root.clone();
        let bm = btn_more.clone();
        let is = input_search.clone();
        let ab = add_btn.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            lbl_sub,
            switch_master,
            input_search,
            add_btn,
            list_container,
            pag_container,
            all_domains: RefCell::new(Vec::new()),
            custom_domains: RefCell::new(HashSet::new()),
            search_query: RefCell::new(String::new()),
            current_page: RefCell::new(1),
            filtered_domains: RefCell::new(Vec::new()),
            back_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i3 = inner.clone();
        let bm2 = bm.clone();
        bm.connect_clicked(move |_| show_more_menu(&i3, &bm2));

        let i4 = inner.clone();
        is.connect_changed(move |e| {
            *i4.search_query.borrow_mut() = e.text().trim().to_string();
            apply_filter(&i4);
            render_current_page(&i4);
        });

        let i5 = inner.clone();
        ab.connect_clicked(move |_| open_add_dialog(&i5));

        let i6 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i6)));

        load_data(&inner);
        // Python calls _render_current_page() at the end of _init_ui(), after
        // _load_data() has filled all_domains — without this the list stays empty.
        render_current_page(&inner);
        retranslate_ui(&inner);

        SettingsWebsiteAllowlistView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }
}

fn load_data(inner: &Rc<Inner>) {
    inner.all_domains.borrow_mut().clear();
    inner.custom_domains.borrow_mut().clear();

    let source = if exclusions_file().exists() {
        exclusions_file()
    } else {
        exclusions_bak()
    };
    if source.exists() {
        if let Ok(content) = std::fs::read_to_string(&source) {
            let mut domains = inner.all_domains.borrow_mut();
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') && !domains.contains(&line.to_string()) {
                    domains.push(line.to_string());
                }
            }
        }
    }
    apply_filter(inner);
}

fn apply_filter(inner: &Rc<Inner>) {
    let query = inner.search_query.borrow().clone();
    let filtered: Vec<String> = if query.is_empty() {
        inner.all_domains.borrow().clone()
    } else {
        let q = query.to_lowercase();
        inner
            .all_domains
            .borrow()
            .iter()
            .filter(|d| d.to_lowercase().contains(&q))
            .cloned()
            .collect()
    };
    *inner.filtered_domains.borrow_mut() = filtered;
    *inner.current_page.borrow_mut() = 1;
}

fn render_current_page(inner: &Rc<Inner>) {
    while let Some(child) = inner.list_container.first_child() {
        inner.list_container.remove(&child);
    }

    let filtered = inner.filtered_domains.borrow().clone();
    let total = filtered.len();
    let total_pages = ((total + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE).max(1);
    if *inner.current_page.borrow() > total_pages {
        *inner.current_page.borrow_mut() = total_pages;
    }
    let page = *inner.current_page.borrow();
    let start = (page - 1) * ITEMS_PER_PAGE;
    let end = (start + ITEMS_PER_PAGE).min(total);
    let page_items = filtered[start..end].to_vec();

    if page_items.is_empty() {
        let empty_lbl = gtk4::Label::new(Some(&i18n::tr("allowlist_empty")));
        empty_lbl.add_css_class("allowlist-empty");
        inner.list_container.append(&empty_lbl);
    } else {
        for domain in &page_items {
            let is_custom = inner.custom_domains.borrow().contains(domain);
            let row = allowlist_row(inner, domain, is_custom);
            inner.list_container.append(&row);
        }
    }

    render_pagination_bar(inner, total_pages);
}

fn allowlist_row(inner: &Rc<Inner>, domain: &str, is_custom: bool) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    row.add_css_class("allowlist-row");
    row.set_size_request(-1, 44);
    row.set_margin_start(10);
    row.set_margin_end(10);

    if c::icons_dir().join("domain_web.svg").exists() {
        row.append(&svg::svg_image("domain_web.svg", 18));
    }

    let lbl = gtk4::Label::new(Some(domain));
    lbl.add_css_class("allowlist-domain");
    row.append(&lbl);

    let row_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    row_spacer.set_hexpand(true);
    row.append(&row_spacer);

    if is_custom {
        let btn_del = gtk4::Button::new();
        btn_del.set_label("✕");
        btn_del.add_css_class("pag-del");
        btn_del.set_size_request(24, 24);
        let target = inner.clone();
        let d = domain.to_string();
        btn_del.connect_clicked(move |_| remove_domain(&target, &d));
        row.append(&btn_del);
    }
    row
}

fn render_pagination_bar(inner: &Rc<Inner>, total_pages: usize) {
    while let Some(child) = inner.pag_container.first_child() {
        inner.pag_container.remove(&child);
    }
    if total_pages <= 1 {
        return;
    }

    let page = *inner.current_page.borrow();

    let btn_prev = gtk4::Button::new();
    btn_prev.set_label("<");
    btn_prev.add_css_class("pag-btn");
    btn_prev.set_size_request(32, 32);
    if page > 1 {
        btn_prev.add_css_class("pag-btn-normal");
    } else {
        btn_prev.add_css_class("pag-btn-disabled");
    }
    btn_prev.set_sensitive(page > 1);
    let t1 = inner.clone();
    btn_prev.connect_clicked(move |_| {
        let p = *t1.current_page.borrow();
        if p > 1 {
            *t1.current_page.borrow_mut() = p - 1;
            render_current_page(&t1);
        }
    });
    inner.pag_container.append(&btn_prev);

    let pages_to_show: Vec<String> = if total_pages <= 7 {
        (1..=total_pages).map(|p| p.to_string()).collect()
    } else if page <= 4 {
        vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into(), "6".into(), "...".into(), total_pages.to_string()]
    } else if page >= total_pages - 3 {
        vec![
            "1".into(),
            "...".into(),
            (total_pages - 4).to_string(),
            (total_pages - 3).to_string(),
            (total_pages - 2).to_string(),
            (total_pages - 1).to_string(),
            total_pages.to_string(),
        ]
    } else {
        vec![
            "1".into(),
            "...".into(),
            (page - 1).to_string(),
            page.to_string(),
            (page + 1).to_string(),
            "...".into(),
            total_pages.to_string(),
        ]
    };

    for p in pages_to_show {
        if p == "..." {
            let lbl = gtk4::Label::new(Some("..."));
            lbl.add_css_class("pag-ellipsis");
            lbl.set_size_request(24, 32);
            inner.pag_container.append(&lbl);
        } else {
            let page_num: usize = p.parse().unwrap_or(1);
            let is_active = page_num == page;
            let b = gtk4::Button::new();
            b.set_label(&p);
            b.set_size_request(32, 32);
            b.add_css_class("pag-btn");
            if is_active {
                b.add_css_class("pag-btn-active");
            } else {
                b.add_css_class("pag-btn-normal");
            }
            let t2 = inner.clone();
            b.connect_clicked(move |_| {
                *t2.current_page.borrow_mut() = page_num;
                render_current_page(&t2);
            });
            inner.pag_container.append(&b);
        }
    }

    let btn_next = gtk4::Button::new();
    btn_next.set_label(">");
    btn_next.add_css_class("pag-btn");
    btn_next.set_size_request(32, 32);
    if page < total_pages {
        btn_next.add_css_class("pag-btn-normal");
    } else {
        btn_next.add_css_class("pag-btn-disabled");
    }
    btn_next.set_sensitive(page < total_pages);
    let t3 = inner.clone();
    let tp = total_pages;
    btn_next.connect_clicked(move |_| {
        let p = *t3.current_page.borrow();
        if p < tp {
            *t3.current_page.borrow_mut() = p + 1;
            render_current_page(&t3);
        }
    });
    inner.pag_container.append(&btn_next);
}

fn add_domain(inner: &Rc<Inner>, dom: &str) {
    let dom = dom.trim().to_string();
    if dom.is_empty() {
        return;
    }
    if !inner.all_domains.borrow().contains(&dom) {
        inner.all_domains.borrow_mut().insert(0, dom.clone());
        inner.custom_domains.borrow_mut().insert(dom.clone());
        for path in [exclusions_file(), root_exclusions_file()] {
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&path) {
                use std::io::Write;
                let _ = f.write(format!("\n{}", dom).as_bytes());
            }
        }
        set_sub_label(inner);
        apply_filter(inner);
        render_current_page(inner);
    }
}

fn remove_domain(inner: &Rc<Inner>, domain: &str) {
    inner.custom_domains.borrow_mut().remove(domain);
    inner.all_domains.borrow_mut().retain(|d| d != domain);
    let domains = inner.all_domains.borrow().clone();
    for path in [exclusions_file(), root_exclusions_file()] {
        if let Ok(mut f) = std::fs::OpenOptions::new().write(true).truncate(true).open(&path) {
            use std::io::Write;
            for d in &domains {
                let _ = f.write(format!("{}\n", d).as_bytes());
            }
        }
    }
    set_sub_label(inner);
    apply_filter(inner);
    render_current_page(inner);
}

fn set_sub_label(inner: &Rc<Inner>) {
    inner.lbl_sub.set_text(&i18n::tr("https_allowlist_d").replace("{count}", &inner.all_domains.borrow().len().to_string()));
}

fn open_add_dialog(inner: &Rc<Inner>) {
    let dlg = crate::util::sub_window("");
    dlg.set_title(Some(&i18n::tr("allowlist_dlg_title")));
    dlg.set_default_size(400, 210);
    dlg.add_css_class("dlg-dark");

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    col.set_margin_start(24);
    col.set_margin_end(24);
    col.set_margin_top(24);
    col.set_margin_bottom(20);
    dlg.set_child(Some(&col));

    let title = gtk4::Label::new(Some(&i18n::tr("allowlist_dlg_head")));
    title.add_css_class("dlg-title");
    title.set_halign(gtk4::Align::Start);
    col.append(&title);

    let desc = gtk4::Label::new(Some(&i18n::tr("allowlist_dlg_desc")));
    desc.add_css_class("lbl-10");
    desc.set_wrap(true);
    desc.set_halign(gtk4::Align::Start);
    col.append(&desc);

    let input_domain = gtk4::Entry::new();
    input_domain.set_placeholder_text(Some("example.com"));
    input_domain.set_size_request(-1, 40);
    input_domain.add_css_class("dlg-input");
    col.append(&input_domain);

    let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let row_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    row_spacer.set_hexpand(true);
    btn_row.append(&row_spacer);

    let btn_cancel = gtk4::Button::new();
    btn_cancel.set_label(&i18n::tr("btn_cancel"));
    btn_cancel.add_css_class("outline-btn");
    btn_cancel.set_size_request(-1, 34);
    let d1 = dlg.clone();
    btn_cancel.connect_clicked(move |_| d1.set_visible(false));

    let btn_add = gtk4::Button::new();
    btn_add.set_label(&i18n::tr("btn_add"));
    btn_add.add_css_class("green-btn");
    btn_add.set_size_request(-1, 34);

    let target = inner.clone();
    let d2 = dlg.clone();
    let input = input_domain.clone();
    btn_add.connect_clicked(move |_| {
        let raw = input.text().trim().to_lowercase();
        let val = if raw.starts_with("http://") {
            raw[7..].to_string()
        } else if raw.starts_with("https://") {
            raw[8..].to_string()
        } else {
            raw.clone()
        };
        let val = val.split('/').next().unwrap_or("").to_string();
        if val.is_empty() || !val.contains('.') {
            show_message(None, "Invalid Domain", "Please enter a valid domain (e.g. example.com).");
            return;
        }
        add_domain(&target, &val);
        d2.set_visible(false);
    });
    btn_row.append(&btn_cancel);
    btn_row.append(&btn_add);
    col.append(&btn_row);

    dlg.present();
}

fn show_more_menu(inner: &Rc<Inner>, anchor: &gtk4::Button) {
    let popover = gtk4::Popover::new();
    popover.set_parent(anchor);
    popover.add_css_class("menu-popover");
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_add = gtk4::Button::new();
    item_add.set_label(&i18n::tr("menu_add_website"));
    item_add.add_css_class("menu-item");
    let pop1 = popover.clone();
    let t1 = inner.clone();
    item_add.connect_clicked(move |_| {
        open_add_dialog(&t1);
        pop1.hide();
    });
    menu_box.append(&item_add);

    let sep = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep.add_css_class("menu-separator");
    sep.set_size_request(-1, 1);
    menu_box.append(&sep);

    let item_open = gtk4::Button::new();
    item_open.set_label(&i18n::tr("menu_open_exclusions_file"));
    item_open.add_css_class("menu-item");
    let pop2 = popover.clone();
    item_open.connect_clicked(move |_| {
        open_file();
        pop2.hide();
    });
    menu_box.append(&item_open);

    popover.set_child(Some(&menu_box));
    popover.present();
}

fn open_file() {
    let target = if exclusions_file().exists() { exclusions_file() } else { exclusions_bak() };
    match std::process::Command::new("xdg-open").arg(&target).spawn() {
        Ok(_) => {}
        Err(e) => show_message(None, "Error", &e.to_string()),
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("https_allowlist_t"));
    set_sub_label(inner);
    inner.input_search.set_placeholder_text(Some(&i18n::tr("search_placeholder")));
    inner.add_btn.set_label(&i18n::tr("allowlist_add_btn"));
    render_current_page(inner);
}

fn spacer_px(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}
