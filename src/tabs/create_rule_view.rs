// Port of adguard_modern/tabs/create_rule_view.py
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::assistant_modal::AdGuardAssistantModal;
use crate::widgets::common_controls::no_wheel_combo;
use crate::widgets::svg;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn user_rules_file() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/user.txt")
}

fn string_list(items: &[&str]) -> gtk4::StringList {
    let sl = gtk4::StringList::new(&[]);
    for it in items {
        sl.append(it);
    }
    sl
}

struct Inner {
    root: gtk4::Box,
    btn_back: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_title: gtk4::Label,
    combo_type: gtk4::DropDown,
    lbl_domain: gtk4::Label,
    input_domain: gtk4::Entry,
    lbl_content: gtk4::Label,
    combo_content: gtk4::DropDown,
    lbl_scope: gtk4::Label,
    combo_scope: gtk4::DropDown,
    chk_priority: gtk4::CheckButton,
    btn_create: gtk4::Button,
    btn_cancel: gtk4::Button,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    rule_created_cb: RefCell<Option<Box<dyn Fn(String)>>>,
    // Python blockSignals guard: programmatic model/index changes must not
    // re-trigger on_type_changed.
    syncing: RefCell<bool>,
}

#[derive(Clone)]
pub struct CreateRuleView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl CreateRuleView {
    pub fn new() -> CreateRuleView {
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

        // Title: Create rule.
        let lbl_title = gtk4::Label::new(Some("Create rule"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        lbl_title.set_margin_start(12);
        root.append(&lbl_title);

        root.append(&spacer(6));

        // Form container.
        let form = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        form.set_margin_start(12);
        form.set_margin_end(12);

        // 1. Rule type dropdown.
        let combo_type = no_wheel_combo();
        combo_type.set_model(Some(&string_list(&[
            "🚫   Block requests",
            "🔗   Unblock requests",
            "🛡️   Disable filtering for a website",
            "⚡   Custom",
            "💬   Comment",
        ])));
        form.append(&combo_type);

        // 2. Domain input.
        let lbl_domain = gtk4::Label::new(Some("Block requests to this domain"));
        lbl_domain.add_css_class("lbl-12-bold-white");
        lbl_domain.set_halign(gtk4::Align::Start);
        form.append(&lbl_domain);

        let input_domain = gtk4::Entry::new();
        input_domain.set_placeholder_text(Some("example.com"));
        input_domain.add_css_class("cr-entry");
        form.append(&input_domain);

        // 3. Block this content dropdown.
        let lbl_content = gtk4::Label::new(Some("Block this content"));
        lbl_content.add_css_class("lbl-12-bold-white");
        lbl_content.set_halign(gtk4::Align::Start);
        form.append(&lbl_content);

        let combo_content = no_wheel_combo();
        combo_content.set_model(Some(&string_list(&[
            "None selected",
            "All content",
            "Images",
            "CSS Stylesheets",
            "Scripts",
            "Web pages",
        ])));
        form.append(&combo_content);

        // 4. Apply the rule to these websites.
        let lbl_scope = gtk4::Label::new(Some("Apply the rule to these websites"));
        lbl_scope.add_css_class("lbl-12-bold-white");
        lbl_scope.set_halign(gtk4::Align::Start);
        form.append(&lbl_scope);

        let combo_scope = no_wheel_combo();
        combo_scope.set_model(Some(&string_list(&[
            "All",
            "Only this",
            "All other",
            "Only listed",
            "All except listed",
        ])));
        form.append(&combo_scope);

        // 5. Set to high priority checkbox.
        let chk_priority = gtk4::CheckButton::new();
        chk_priority.set_label(Some("Set to high priority"));
        chk_priority.add_css_class("lbl-12-bold-white");
        form.append(&chk_priority);

        root.append(&form);
        root.append(&spacer(0)); // stretch placeholder handled by vexpand below

        // Bottom buttons: Create / Cancel.
        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        btn_row.set_margin_top(10);
        btn_row.set_margin_bottom(10);
        btn_row.set_margin_start(12);
        btn_row.set_margin_end(12);

        let btn_create = gtk4::Button::new();
        btn_create.set_label("Create");
        btn_create.set_size_request(130, 40);
        btn_create.add_css_class("cr-create-btn");
        btn_row.append(&btn_create);

        let btn_cancel = gtk4::Button::new();
        btn_cancel.set_label("Cancel");
        btn_cancel.set_size_request(130, 40);
        btn_cancel.add_css_class("cr-cancel-btn");
        btn_row.append(&btn_cancel);
        root.append(&btn_row);

        let inner = Rc::new(Inner {
            root,
            btn_back,
            btn_more,
            lbl_title,
            combo_type,
            lbl_domain,
            input_domain,
            lbl_content,
            combo_content,
            lbl_scope,
            combo_scope,
            chk_priority,
            btn_create,
            btn_cancel,
            back_cb: RefCell::new(None),
            rule_created_cb: RefCell::new(None),
            syncing: RefCell::new(false),
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
        inner.combo_type.connect_selected_notify(move |c| {
            if *i3.syncing.borrow() {
                return;
            }
            on_type_changed(&i3, c.selected() as i32);
        });

        let i4 = inner.clone();
        inner.btn_create.connect_clicked(move |_| create_rule(&i4));

        let i5 = inner.clone();
        inner.btn_cancel.connect_clicked(move |_| {
            if let Some(cb) = i5.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i6 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i6)));

        retranslate_ui(&inner);

        CreateRuleView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_rule_created(&self, cb: Box<dyn Fn(String)>) {
        *self.inner.rule_created_cb.borrow_mut() = Some(cb);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.lbl_title.set_text(&i18n::tr("cr_title"));
    inner.lbl_content.set_text(&i18n::tr("cr_content_label"));
    inner.lbl_scope.set_text(&i18n::tr("cr_scope_label"));
    inner.chk_priority.set_label(Some(&i18n::tr("cr_priority")));
    inner.btn_create.set_label(&i18n::tr("cr_btn_create"));
    inner.btn_cancel.set_label(&i18n::tr("cr_btn_cancel"));

    // Python rebuilds the combo model with translations, keeping the current
    // index, with signals blocked during the rebuild.
    let curr = inner.combo_type.selected();
    *inner.syncing.borrow_mut() = true;
    inner.combo_type.set_model(Some(&string_list(&[
        &i18n::tr("cr_type_block"),
        &i18n::tr("cr_type_unblock"),
        &i18n::tr("cr_type_disable"),
        &i18n::tr("cr_type_custom"),
        &i18n::tr("cr_type_comment"),
    ])));
    if curr < u32::MAX {
        inner.combo_type.set_selected(curr);
    } else {
        inner.combo_type.set_selected(0);
    }
    *inner.syncing.borrow_mut() = false;

    on_type_changed(inner, inner.combo_type.selected() as i32);
}

fn on_type_changed(inner: &Rc<Inner>, idx: i32) {
    if idx == 0 {
        inner.lbl_domain.set_text(&i18n::tr("cr_domain_label"));
        inner.input_domain.set_placeholder_text(Some("example.com"));
    } else if idx == 1 {
        inner.lbl_domain.set_text(&i18n::tr("cr_type_unblock"));
        inner.input_domain.set_placeholder_text(Some("example.com"));
    } else if idx == 2 {
        inner.lbl_domain.set_text(&i18n::tr("cr_type_disable"));
        inner.input_domain.set_placeholder_text(Some("example.com"));
    } else if idx == 3 {
        inner.lbl_domain.set_text(&i18n::tr("cr_type_custom"));
        inner.input_domain.set_placeholder_text(Some("||example.com^ or ##.banner"));
    } else {
        inner.lbl_domain.set_text(&i18n::tr("cr_type_comment"));
        inner.input_domain.set_placeholder_text(Some("My custom rules note"));
    }
}

fn create_rule(inner: &Rc<Inner>) {
    let domain = inner.input_domain.text().trim().to_string();
    if domain.is_empty() {
        return;
    }

    let rule_type_idx = inner.combo_type.selected() as i32;
    let content_choice = inner.combo_content.selected() as i32;
    let is_priority = inner.chk_priority.is_active();

    // Python compares combo currentText() against English literals; the
    // content combo is never retranslated, so index mapping is equivalent.
    let mut modifiers: Vec<String> = Vec::new();
    match content_choice {
        2 => modifiers.push("image".to_string()),
        3 => modifiers.push("stylesheet".to_string()),
        4 => modifiers.push("script".to_string()),
        5 => modifiers.push("document".to_string()),
        _ => {}
    }
    if is_priority {
        modifiers.push("important".to_string());
    }

    let mod_str = if modifiers.is_empty() {
        String::new()
    } else {
        format!("${}", modifiers.join(","))
    };

    let final_rule = match rule_type_idx {
        0 => format!("||{domain}^{mod_str}"),
        1 => format!("@@||{domain}^{mod_str}"),
        2 => format!("@@||{domain}^$document"),
        3 => domain.clone(),
        _ => format!("! {domain}"),
    };

    // Write to user.txt.
    let file = user_rules_file();
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match std::fs::OpenOptions::new().append(true).create(true).open(&file) {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = writeln!(f, "{final_rule}") {
                println!("Error saving rule: {e}");
            }
        }
        Err(e) => println!("Error saving rule: {e}"),
    }

    // Reset fields (signals guarded so reset does not re-run on_type_changed).
    *inner.syncing.borrow_mut() = true;
    inner.input_domain.set_text("");
    inner.combo_type.set_selected(0);
    inner.combo_content.set_selected(0);
    inner.combo_scope.set_selected(0);
    inner.chk_priority.set_active(false);
    *inner.syncing.borrow_mut() = false;

    if let Some(cb) = inner.rule_created_cb.borrow().as_ref() {
        cb(final_rule);
    }
    if let Some(cb) = inner.back_cb.borrow().as_ref() {
        cb();
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
