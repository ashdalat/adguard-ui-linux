// Port of adguard_modern/tabs/settings_ev_certificates_view.py — the EV
// certificates subview (master toggle, search, add website, more menu).
use gtk4::prelude::*;
use regex::RegexBuilder;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use crate::widgets::common_controls::BackButton;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

fn proxy_yaml() -> PathBuf {
    c::home_dir().join(".local/share/adguard-cli/proxy.yaml")
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
    lbl_no_web: gtk4::Label,
    ev_domains: RefCell<Vec<String>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct SettingsEvCertificatesView {
    inner: Rc<Inner>,
}

impl SettingsEvCertificatesView {
    pub fn new() -> SettingsEvCertificatesView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(20);

        // Load config before building the UI (Python calls _load_config first).
        let ev_enabled = read_ev_enabled();

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = BackButton::new("Back");
        top_bar.append(&btn_back.root);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top_bar.append(&spacer);

        let btn_more = gtk4::Button::new();
        btn_more.add_css_class("flat-icon-btn");
        btn_more.set_size_request(30, 30);
        let more_ic = c::icons_dir().join("stat_more.svg");
        if more_ic.exists() {
            btn_more.set_child(Some(&svg::svg_image("stat_more.svg", 16)));
        }
        top_bar.append(&btn_more);
        root.append(&top_bar);
        root.append(&spacer_px(20));

        let head_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        title_box.set_hexpand(true);
        let lbl_title = gtk4::Label::new(Some("Filter websites with EV certificates"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        title_box.append(&lbl_title);
        let lbl_sub = gtk4::Label::new(Some("Filters websites with Extended Validation security certificates"));
        lbl_sub.add_css_class("lbl-sub-11");
        lbl_sub.set_halign(gtk4::Align::Start);
        title_box.append(&lbl_sub);
        head_row.append(&title_box);
        let head_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        head_spacer.set_hexpand(true);
        head_row.append(&head_spacer);

        let switch_master = ToggleSwitch::new(46, 26);
        switch_master.set_checked(ev_enabled, false);
        head_row.append(switch_master.root());
        root.append(&head_row);
        root.append(&spacer_px(20));

        // Search bar
        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_box.add_css_class("search-box");
        search_box.set_size_request(-1, 42);
        let search_svg = c::icons_dir().join("search.svg");
        if search_svg.exists() {
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
        root.append(&spacer_px(40));

        // Empty state
        let empty_container = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        empty_container.set_halign(gtk4::Align::Center);
        let icon_empty = c::icons_dir().join("empty_rules.svg");
        if icon_empty.exists() {
            let img = svg::svg_image("empty_rules.svg", 64);
            img.set_halign(gtk4::Align::Center);
            empty_container.append(&img);
        }
        let lbl_no_web = gtk4::Label::new(Some("No websites"));
        lbl_no_web.add_css_class("lbl-sub-11");
        lbl_no_web.set_halign(gtk4::Align::Center);
        empty_container.append(&lbl_no_web);
        root.append(&empty_container);

        let bb_root = btn_back.root.clone();
        let sm = switch_master.clone();
        let ab = add_btn.clone();
        let bm = btn_more.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            lbl_sub,
            switch_master,
            input_search,
            add_btn,
            lbl_no_web,
            ev_domains: RefCell::new(Vec::new()),
            back_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });

        sm.connect_toggled(Box::new(move |checked| on_ev_toggled(checked)));

        let i4 = inner.clone();
        let bm2 = bm.clone();
        bm.connect_clicked(move |_| show_more_menu(&i4, &bm2));

        let i5 = inner.clone();
        ab.connect_clicked(move |_| open_add_dialog(&i5));

        let i6 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i6)));
        retranslate_ui(&inner);

        SettingsEvCertificatesView { inner }
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

fn read_ev_enabled() -> bool {
    let p = proxy_yaml();
    if p.exists() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Some(m) = RegexBuilder::new(r"filter_ev_certificates:\s*(true|false)")
                .case_insensitive(true)
                .build()
                .ok()
                .and_then(|re| re.captures(&content))
            {
                return m.get(1).unwrap().as_str().to_lowercase() == "true";
            }
        }
    }
    false
}

fn on_ev_toggled(checked: bool) {
    let val_str = if checked { "true" } else { "false" };
    let paths = vec![
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
        c::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
    ];
    let re = match RegexBuilder::new(r"filter_ev_certificates:\s*(true|false)").case_insensitive(true).build() {
        Ok(r) => r,
        Err(_) => return,
    };
    for p in paths {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                let updated = if content.contains("filter_ev_certificates:") {
                    re.replace_all(&content, format!("filter_ev_certificates: {}", val_str)).as_ref().to_string()
                } else {
                    format!("{}\n  filter_ev_certificates: {}\n", content, val_str)
                };
                let _ = std::fs::write(&p, updated);
            }
        }
    }
}

fn open_add_dialog(inner: &Rc<Inner>) {
    let dlg = crate::util::sub_window("");
    dlg.set_title(Some(&i18n::tr("ev_dlg_title")));
    dlg.set_default_size(400, 210);
    dlg.add_css_class("dlg-dark");

    let col = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    col.set_margin_start(24);
    col.set_margin_end(24);
    col.set_margin_top(24);
    col.set_margin_bottom(20);
    dlg.set_child(Some(&col));

    let title = gtk4::Label::new(Some(&i18n::tr("ev_dlg_head")));
    title.add_css_class("dlg-title");
    title.set_halign(gtk4::Align::Start);
    col.append(&title);

    let desc = gtk4::Label::new(Some(&i18n::tr("ev_dlg_desc")));
    desc.add_css_class("lbl-10");
    desc.set_wrap(true);
    desc.set_halign(gtk4::Align::Start);
    col.append(&desc);

    let input_domain = gtk4::Entry::new();
    input_domain.set_placeholder_text(Some("bank.com"));
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
        let val = strip_scheme(&raw);
        let val = val.split('/').next().unwrap_or("").to_string();
        if val.is_empty() || !val.contains('.') {
            show_message(None, "Invalid Domain", "Please enter a valid domain.");
            return;
        }
        {
            let domains = target.ev_domains.borrow();
            if domains.contains(&val) {
                d2.set_visible(false);
                return;
            }
        }
        target.ev_domains.borrow_mut().push(val.clone());
        show_message(None, "Added", &format!("Website {} added to EV filtering.", val));
        d2.set_visible(false);
    });
    btn_row.append(&btn_cancel);
    btn_row.append(&btn_add);
    col.append(&btn_row);

    dlg.present();
}

fn strip_scheme(val: &str) -> String {
    if val.starts_with("http://") {
        val[7..].to_string()
    } else if val.starts_with("https://") {
        val[8..].to_string()
    } else {
        val.to_string()
    }
}

fn show_more_menu(inner: &Rc<Inner>, anchor: &gtk4::Button) {
    let popover = gtk4::Popover::new();
    popover.set_parent(anchor);
    popover.add_css_class("menu-popover");
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let item = gtk4::Button::new();
    item.set_label(&i18n::tr("menu_add_website"));
    item.add_css_class("menu-item");
    let pop = popover.clone();
    let target = inner.clone();
    item.connect_clicked(move |_| {
        open_add_dialog(&target);
        pop.hide();
    });
    menu_box.append(&item);
    popover.set_child(Some(&menu_box));
    popover.present();
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_text(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("https_ev_t"));
    inner.lbl_sub.set_text(&i18n::tr("https_ev_d"));
    inner.input_search.set_placeholder_text(Some(&i18n::tr("search_placeholder")));
    inner.add_btn.set_label(&i18n::tr("ev_add_btn"));
    inner.lbl_no_web.set_text(&i18n::tr("ur_empty"));
}

fn spacer_px(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}
