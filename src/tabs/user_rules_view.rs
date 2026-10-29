// Port of adguard_modern/tabs/user_rules_view.py
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::tabs::dns_filter_editor::DnsFilterEditorWindow;
use crate::widgets::assistant_modal::AdGuardAssistantModal;
use crate::widgets::svg;
use crate::widgets::toggle_switch::ToggleSwitch;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

pub fn user_rules_file() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/user.txt")
}

// ---------------------------------------------------------------------------
// UserRuleItemRow
// ---------------------------------------------------------------------------
struct RuleRowInner {
    rule_text: String,
    root: gtk4::Box,
    delete_cb: RefCell<Option<Box<dyn Fn(String)>>>,
}

#[derive(Clone)]
pub struct UserRuleItemRow {
    inner: Rc<RuleRowInner>,
}

impl UserRuleItemRow {
    pub fn new(rule_text: &str) -> UserRuleItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        root.add_css_class("user-rule-row");
        root.set_margin_top(10);
        root.set_margin_bottom(10);
        root.set_margin_start(14);
        root.set_margin_end(14);

        let lbl = gtk4::Label::new(Some(rule_text));
        lbl.add_css_class("lbl-rule-10");
        lbl.set_selectable(true);
        lbl.set_halign(gtk4::Align::Start);
        lbl.set_hexpand(true);
        root.append(&lbl);

        let btn_delete = gtk4::Button::new();
        btn_delete.set_label("✕");
        btn_delete.set_size_request(24, 24);
        btn_delete.add_css_class("user-rule-delete-btn");
        root.append(&btn_delete);

        let inner = Rc::new(RuleRowInner {
            rule_text: rule_text.to_string(),
            root,
            delete_cb: RefCell::new(None),
        });

        let i = inner.clone();
        btn_delete.connect_clicked(move |_| {
            if let Some(cb) = i.delete_cb.borrow().as_ref() {
                cb(i.rule_text.clone());
            }
        });

        UserRuleItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn rule_text(&self) -> &str {
        &self.inner.rule_text
    }

    pub fn connect_delete_requested(&self, cb: Box<dyn Fn(String)>) {
        *self.inner.delete_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// UserRulesView
// ---------------------------------------------------------------------------
struct Inner {
    rules_file: PathBuf,
    custom_title: String,
    custom_desc: String,
    root: gtk4::Box,
    btn_back: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    master_switch: ToggleSwitch,
    lbl_desc: gtk4::Label,
    search_input: gtk4::Entry,
    lbl_card_title: gtk4::Label,
    lbl_card_desc: gtk4::Label,
    btn_go_editor: gtk4::Button,
    scroll_rules: gtk4::ScrolledWindow,
    rules_container: gtk4::Box,
    rule_rows: RefCell<Vec<UserRuleItemRow>>,
    editor_window: RefCell<Option<DnsFilterEditorWindow>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    create_rule_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct UserRulesView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl UserRulesView {
    pub fn new() -> UserRulesView {
        Self::new_with(
            user_rules_file(),
            "User rules",
            "Contains your manually added rules and rules created via the filtering log",
        )
    }

    pub fn new_with(rules_file: PathBuf, title: &str, desc: &str) -> UserRulesView {
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

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("page-title");
        title_row.append(&lbl_title);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        root.append(&title_row);

        // Subtitle.
        let lbl_desc = gtk4::Label::new(Some(desc));
        lbl_desc.add_css_class("lbl-10-white");
        lbl_desc.set_xalign(0.0);
        lbl_desc.set_margin_start(12);
        lbl_desc.set_margin_end(12);
        root.append(&lbl_desc);

        root.append(&spacer(6));

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

        // Editor card.
        let editor_card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        editor_card.add_css_class("ur-editor-card");
        editor_card.set_margin_top(40);
        editor_card.set_margin_bottom(40);
        editor_card.set_margin_start(30);
        editor_card.set_margin_end(30);
        editor_card.set_vexpand(true);

        let lbl_card_title = gtk4::Label::new(Some("Editor opened in another window"));
        lbl_card_title.add_css_class("ur-card-title");
        editor_card.append(&lbl_card_title);

        let lbl_card_desc = gtk4::Label::new(Some("Your DNS rules will appear here after you close it"));
        lbl_card_desc.add_css_class("ur-card-desc");
        editor_card.append(&lbl_card_desc);

        let btn_go_editor = gtk4::Button::new();
        btn_go_editor.set_label("Go to editor");
        btn_go_editor.set_size_request(160, 38);
        btn_go_editor.add_css_class("ur-go-btn");
        editor_card.append(&btn_go_editor);
        root.append(&editor_card);

        // Scroll area for rules list (hidden until editor closed with rules).
        let scroll_rules = gtk4::ScrolledWindow::new();
        scroll_rules.set_vexpand(true);
        scroll_rules.add_css_class("home-scroll");

        let rules_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        rules_container.add_css_class("transparent-container");
        rules_container.set_margin_top(4);
        rules_container.set_margin_end(12);
        rules_container.set_margin_bottom(16);
        scroll_rules.set_child(Some(&rules_container));
        scroll_rules.set_visible(false);
        root.append(&scroll_rules);

        let inner = Rc::new(Inner {
            rules_file,
            custom_title: title.to_string(),
            custom_desc: desc.to_string(),
            root,
            btn_back,
            btn_more,
            lbl_title,
            master_switch,
            lbl_desc,
            search_input,
            lbl_card_title,
            lbl_card_desc,
            btn_go_editor,
            scroll_rules,
            rules_container,
            rule_rows: RefCell::new(Vec::new()),
            editor_window: RefCell::new(None),
            back_cb: RefCell::new(None),
            create_rule_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
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
        bm.connect_clicked(move |_| show_more_menu(&i2));

        let i3 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| {
            if let Some(cb) = i3.master_cb.borrow().as_ref() {
                cb(val);
            }
        }));

        let i4 = inner.clone();
        inner.search_input.connect_changed(move |e| filter_rules(&i4, e.text().as_str()));

        let i5 = inner.clone();
        inner.btn_go_editor.connect_clicked(move |_| open_editor_window(&i5));

        let i6 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i6)));

        retranslate_ui(&inner);

        UserRulesView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_create_rule_clicked(&self, cb: Box<dyn Fn()>) {
        // Python declares create_rule_clicked but never emits it in this file;
        // protection_tab connects it, so the registry is kept for interface parity.
        *self.inner.create_rule_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_master_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.master_cb.borrow_mut() = Some(cb);
    }

    pub fn set_master_checked(&self, checked: bool, animate: bool) {
        self.inner.master_switch.set_checked(checked, animate);
    }

    pub fn refresh_rules(&self) {
        refresh_rules(&self.inner);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    let is_dns = inner.rules_file.to_string_lossy().to_lowercase().contains("dns");
    if is_dns {
        inner.lbl_title.set_text(&i18n::tr("dns_card_user_rules_t"));
        inner.lbl_desc.set_text(&i18n::tr("dns_card_user_rules_d"));
    } else {
        inner.lbl_title.set_text(&i18n::tr("ur_title"));
        inner.lbl_desc.set_text(&i18n::tr("ur_desc"));
    }
    inner.search_input.set_placeholder_text(Some(&i18n::tr("ur_search_ph")));
    inner.lbl_card_title.set_text(&i18n::tr("ur_card_title"));
    inner.lbl_card_desc.set_text(&i18n::tr("ur_card_desc"));
    inner.btn_go_editor.set_label(&i18n::tr("ur_btn_go_editor"));
}

fn open_editor_window(inner: &Rc<Inner>) {
    {
        let mut guard = inner.editor_window.borrow_mut();
        if guard.is_none() {
            let win = DnsFilterEditorWindow::new();
            let i = inner.clone();
            win.connect_closed(Box::new(move || refresh_rules(&i)));
            *guard = Some(win);
        }
    }
    let win = inner.editor_window.borrow();
    let w = win.as_ref().unwrap();
    // show + raise + activateWindow -> present().
    w.root().present();
}

// Read live rules from the configured rules file.
fn refresh_rules(inner: &Rc<Inner>) {
    let mut rules: Vec<String> = Vec::new();
    if inner.rules_file.exists() {
        match std::fs::read_to_string(&inner.rules_file) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('!') && !line.starts_with('#') {
                        rules.push(line.to_string());
                    }
                }
            }
            Err(e) => println!("Error reading user rules: {e}"),
        }
    }

    // Clear existing rows.
    while let Some(child) = inner.rules_container.first_child() {
        inner.rules_container.remove(&child);
    }
    inner.rule_rows.borrow_mut().clear();

    for r in rules {
        let row = UserRuleItemRow::new(&r);

        let i = inner.clone();
        row.connect_delete_requested(Box::new(move |rule_text| delete_rule(&i, &rule_text)));

        inner.rules_container.append(row.root());
        inner.rule_rows.borrow_mut().push(row);
    }
}

// Delete a rule from the configured rules file and refresh.
fn delete_rule(inner: &Rc<Inner>, rule_text: &str) {
    if !inner.rules_file.exists() {
        return;
    }
    match std::fs::read_to_string(&inner.rules_file) {
        Ok(content) => {
            let kept: String = content
                .lines()
                .filter(|line| line.trim() != rule_text)
                .map(|line| format!("{line}\n"))
                .collect();
            if let Err(e) = std::fs::write(&inner.rules_file, kept) {
                println!("Error deleting rule: {e}");
                return;
            }
            refresh_rules(inner);
        }
        Err(e) => println!("Error deleting rule: {e}"),
    }
}

fn filter_rules(inner: &Rc<Inner>, text: &str) {
    let query = text.trim().to_lowercase();
    for row in inner.rule_rows.borrow().iter() {
        if query.is_empty() || row.rule_text().to_lowercase().contains(&query) {
            row.root().set_visible(true);
        } else {
            row.root().set_visible(false);
        }
    }
}

fn show_more_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("detail-menu-popover");
    popover.set_parent(&inner.btn_more);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let item_syntax = gtk4::Button::new();
    item_syntax.set_label(&i18n::tr("menu_rule_syntax"));
    item_syntax.add_css_class("detail-menu-item");
    let p1 = popover.clone();
    item_syntax.connect_clicked(move |_| {
        p1.hide();
        let _ = std::process::Command::new("xdg-open")
            .arg("https://adguard.com/kb/general/ad-filtering/create-own-filters/")
            .spawn();
    });
    menu_box.append(&item_syntax);

    let item_assistant = gtk4::Button::new();
    item_assistant.set_label(&i18n::tr("menu_adguard_assistant"));
    item_assistant.add_css_class("detail-menu-item");
    let p2 = popover.clone();
    item_assistant.connect_clicked(move |_| {
        p2.hide();
        // Deviation: modal.exec() -> non-modal window (no parent).
        AdGuardAssistantModal::new().root.present();
    });
    menu_box.append(&item_assistant);

    popover.set_child(Some(&menu_box));
    popover.show();
}
