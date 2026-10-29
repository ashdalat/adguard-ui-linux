// Port of adguard_modern/tabs/extensions_view.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use regex::Regex;

use crate::i18n;
use crate::widgets::filtering_log_window::show_message;
use crate::widgets::toggle_switch::ToggleSwitch;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn scripts_dir() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/userscripts")
}

fn cli_bin() -> String {
    let home_bin = home_dir().join(".local/bin/adguard-cli");
    if home_bin.exists() {
        home_bin.to_string_lossy().to_string()
    } else {
        "adguard-cli".to_string()
    }
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

#[derive(Clone)]
pub struct ExtensionData {
    pub id: String,
    pub name: String,
    pub version: String,
    pub time_str: String,
    pub homepage: String,
    pub description: String,
    pub meta_path: PathBuf,
    pub js_path: PathBuf,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// ExtensionCard
// ---------------------------------------------------------------------------
struct CardInner {
    root: gtk4::Box,
    check_btn: gtk4::Button,
    title_lbl: gtk4::Label,
    type_lbl: gtk4::Label,
    detail_lbl: gtk4::Label,
    menu_btn: gtk4::Button,
    data: RefCell<ExtensionData>,
    toggled_cb: RefCell<Option<Box<dyn Fn(String, bool)>>>,
    menu_cb: RefCell<Option<Box<dyn Fn(&str, &ExtensionData)>>>,
}

#[derive(Clone)]
pub struct ExtensionCard {
    inner: Rc<CardInner>,
}

impl ExtensionCard {
    pub fn new(data: ExtensionData) -> ExtensionCard {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("ext-card");

        // 1. Green square checkbox button.
        let check_btn = gtk4::Button::new();
        check_btn.set_size_request(22, 22);
        root.append(&check_btn);

        // 2. Text layout (Title, Subtitle, Version & timestamp).
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text_box.set_hexpand(true);

        let title_lbl = gtk4::Label::new(Some(if data.name.is_empty() {
            "Unnamed Extension"
        } else {
            &data.name
        }));
        title_lbl.add_css_class("ext-title-14");
        title_lbl.set_halign(gtk4::Align::Start);
        text_box.append(&title_lbl);

        let type_lbl = gtk4::Label::new(Some("Userscript"));
        type_lbl.add_css_class("ext-type-12");
        type_lbl.set_halign(gtk4::Align::Start);
        text_box.append(&type_lbl);

        let detail_text = format!(
            "{}, last updated on {}",
            if data.version.is_empty() { "1.0.0" } else { &data.version },
            if data.time_str.is_empty() { "Recently" } else { &data.time_str }
        );
        let detail_lbl = gtk4::Label::new(Some(&detail_text));
        detail_lbl.add_css_class("ext-detail-12");
        detail_lbl.set_halign(gtk4::Align::Start);
        text_box.append(&detail_lbl);

        root.append(&text_box);

        // 3. 3-dots context menu button.
        let menu_btn = gtk4::Button::new();
        menu_btn.set_label("⋮");
        menu_btn.set_size_request(28, 28);
        menu_btn.add_css_class("ext-header-menu-btn");
        root.append(&menu_btn);

        let inner = Rc::new(CardInner {
            root,
            check_btn,
            title_lbl,
            type_lbl,
            detail_lbl,
            menu_btn,
            data: RefCell::new(data),
            toggled_cb: RefCell::new(None),
            menu_cb: RefCell::new(None),
        });

        update_checkbox_style(&inner);

        let i1 = inner.clone();
        inner.check_btn.connect_clicked(move |_| {
            let enabled = !i1.data.borrow().enabled;
            i1.data.borrow_mut().enabled = enabled;
            update_checkbox_style(&i1);
            let id = i1.data.borrow().id.clone();
            if let Some(cb) = i1.toggled_cb.borrow().as_ref() {
                cb(id, enabled);
            }
        });

        let i2 = inner.clone();
        inner.menu_btn.connect_clicked(move |_| show_card_menu(&i2));

        retranslate_card(&inner);

        ExtensionCard { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn retranslate_ui(&self) {
        retranslate_card(&self.inner);
    }

    pub fn connect_toggled(&self, cb: Box<dyn Fn(String, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_menu_requested(&self, cb: Box<dyn Fn(&str, &ExtensionData)>) {
        *self.inner.menu_cb.borrow_mut() = Some(cb);
    }
}

fn retranslate_card(inner: &Rc<CardInner>) {
    inner.type_lbl.set_text(&i18n::tr("ext_type_userscript"));
    let detail = i18n::tr("ext_last_updated")
        .replace("{version}", &inner.data.borrow().version)
        .replace("{time_str}", &inner.data.borrow().time_str);
    inner.detail_lbl.set_text(&detail);
}

fn update_checkbox_style(inner: &Rc<CardInner>) {
    if inner.data.borrow().enabled {
        inner.check_btn.set_label("✓");
        inner.check_btn.remove_css_class("ext-check-btn-off");
        inner.check_btn.add_css_class("ext-check-btn");
    } else {
        inner.check_btn.set_label("");
        inner.check_btn.remove_css_class("ext-check-btn");
        inner.check_btn.add_css_class("ext-check-btn-off");
    }
}

fn show_card_menu(inner: &Rc<CardInner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("detail-menu-popover");
    popover.set_parent(&inner.menu_btn);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let act_home = gtk4::Button::new();
    act_home.set_label(&i18n::tr("menu_homepage"));
    act_home.add_css_class("detail-menu-item");
    let p1 = popover.clone();
    let i1 = inner.clone();
    act_home.connect_clicked(move |_| {
        p1.hide();
        let data = i1.data.borrow();
        if let Some(cb) = i1.menu_cb.borrow().as_ref() {
            cb("Homepage", &data);
        }
    });
    menu_box.append(&act_home);

    let act_reinstall = gtk4::Button::new();
    act_reinstall.set_label(&i18n::tr("menu_reinstall"));
    act_reinstall.add_css_class("detail-menu-item");
    let p2 = popover.clone();
    let i2 = inner.clone();
    act_reinstall.connect_clicked(move |_| {
        p2.hide();
        let data = i2.data.borrow();
        if let Some(cb) = i2.menu_cb.borrow().as_ref() {
            cb("Reinstall", &data);
        }
    });
    menu_box.append(&act_reinstall);

    let act_edit = gtk4::Button::new();
    act_edit.set_label(&i18n::tr("menu_edit"));
    act_edit.add_css_class("detail-menu-item");
    let p3 = popover.clone();
    let i3 = inner.clone();
    act_edit.connect_clicked(move |_| {
        p3.hide();
        let data = i3.data.borrow();
        if let Some(cb) = i3.menu_cb.borrow().as_ref() {
            cb("Edit", &data);
        }
    });
    menu_box.append(&act_edit);

    let act_storage = gtk4::Button::new();
    act_storage.set_label(&i18n::tr("menu_storage"));
    act_storage.add_css_class("detail-menu-item");
    let p4 = popover.clone();
    let i4 = inner.clone();
    act_storage.connect_clicked(move |_| {
        p4.hide();
        let data = i4.data.borrow();
        if let Some(cb) = i4.menu_cb.borrow().as_ref() {
            cb("Storage", &data);
        }
    });
    menu_box.append(&act_storage);

    let sep = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep.add_css_class("menu-separator");
    menu_box.append(&sep);

    let act_remove = gtk4::Button::new();
    act_remove.set_label(&i18n::tr("menu_remove"));
    act_remove.add_css_class("detail-menu-item");
    let p5 = popover.clone();
    let i5 = inner.clone();
    act_remove.connect_clicked(move |_| {
        p5.hide();
        let data = i5.data.borrow();
        if let Some(cb) = i5.menu_cb.borrow().as_ref() {
            cb("Remove", &data);
        }
    });
    menu_box.append(&act_remove);

    popover.set_child(Some(&menu_box));
    popover.show();
}

// ---------------------------------------------------------------------------
// ExtensionsView
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    list_container: gtk4::Box,
    btn_back: gtk4::Button,
    header_menu_btn: gtk4::Button,
    title_lbl: gtk4::Label,
    desc_lbl: gtk4::Label,
    link_btn: gtk4::Button,
    auto_title: gtk4::Label,
    auto_sub: gtk4::Label,
    section_title: gtk4::Label,
    add_ext_btn: gtk4::Button,
    master_switch: ToggleSwitch,
    auto_switch: ToggleSwitch,
    is_master_active: std::cell::Cell<bool>,
    auto_detect: std::cell::Cell<bool>,
    editor_windows: RefCell<HashMap<String, gtk4::Window>>,
    cards: RefCell<Vec<ExtensionCard>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    master_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct ExtensionsView {
    inner: Rc<Inner>,
}

impl ExtensionsView {
    pub fn new() -> ExtensionsView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");

        // Scroll area.
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        content.add_css_class("transparent-container");
        content.set_margin_top(20);
        content.set_margin_bottom(28);
        content.set_margin_start(28);
        content.set_margin_end(28);

        // 1. Top bar: '< Back' left, '⋮' right.
        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = gtk4::Button::new();
        btn_back.set_label("‹ Back");
        btn_back.add_css_class("ext-back-btn");
        btn_back.set_halign(gtk4::Align::Start);
        top_bar.append(&btn_back);

        let header_menu_btn = gtk4::Button::new();
        header_menu_btn.set_label("⋮");
        header_menu_btn.set_size_request(28, 28);
        header_menu_btn.add_css_class("ext-header-menu-btn");
        header_menu_btn.set_halign(gtk4::Align::End);
        top_bar.append(&header_menu_btn);
        content.append(&top_bar);

        // 2. Title row + master switch.
        let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let title_lbl = gtk4::Label::new(Some("Extensions"));
        title_lbl.add_css_class("page-title");
        title_row.append(&title_lbl);

        let master_switch = ToggleSwitch::new(46, 26);
        master_switch.set_checked(true, false);
        master_switch.root().set_valign(gtk4::Align::Center);
        title_row.append(master_switch.root());
        content.append(&title_row);

        // 3. Description & Learn more link.
        let desc_lbl = gtk4::Label::new(Some(
            "Allows adding userscripts and userstyles to extend AdGuard Ad Blocker's functionality and improve your web experience",
        ));
        desc_lbl.add_css_class("lbl-13-muted");
        desc_lbl.set_wrap(true);
        desc_lbl.set_xalign(0.0);
        content.append(&desc_lbl);

        let link_btn = gtk4::Button::new();
        link_btn.set_label("Learn more about extensions  ↗");
        link_btn.add_css_class("ext-link");
        link_btn.set_halign(gtk4::Align::Start);
        link_btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/extensions/")
                .spawn();
        });
        content.append(&link_btn);

        content.append(&spacer(6));

        // 4. Card: Automatically detect userscripts.
        let auto_card = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        auto_card.add_css_class("ext-card");

        let auto_text = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        auto_text.set_hexpand(true);
        let auto_title = gtk4::Label::new(Some("Automatically detect userscripts"));
        auto_title.add_css_class("ext-title-13");
        auto_title.set_halign(gtk4::Align::Start);
        auto_text.append(&auto_title);
        let auto_sub = gtk4::Label::new(Some(
            "Detects userscripts you're trying to install and offers to add them directly to the AdGuard app",
        ));
        auto_sub.add_css_class("lbl-12-muted");
        auto_sub.set_wrap(true);
        auto_sub.set_xalign(0.0);
        auto_text.append(&auto_sub);
        auto_card.append(&auto_text);

        let auto_switch = ToggleSwitch::new(46, 26);
        auto_switch.set_checked(true, false);
        auto_switch.root().set_valign(gtk4::Align::Center);
        auto_card.append(auto_switch.root());
        content.append(&auto_card);

        content.append(&spacer(10));

        // 5. Section header + "+ Add extension".
        let ext_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let section_title = gtk4::Label::new(Some("Installed extensions"));
        section_title.add_css_class("ext-section-16");
        ext_header.append(&section_title);
        content.append(&ext_header);

        let add_ext_btn = gtk4::Button::new();
        add_ext_btn.set_label("+  Add extension");
        add_ext_btn.add_css_class("ext-add-btn");
        add_ext_btn.set_halign(gtk4::Align::Start);
        content.append(&add_ext_btn);

        // 6. List container for installed extensions.
        let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content.append(&list_container);

        scroll.set_child(Some(&content));
        root.append(&scroll);

        let inner = Rc::new(Inner {
            root,
            list_container,
            btn_back,
            header_menu_btn,
            title_lbl,
            desc_lbl,
            link_btn,
            auto_title,
            auto_sub,
            section_title,
            add_ext_btn,
            master_switch,
            auto_switch,
            is_master_active: std::cell::Cell::new(true),
            auto_detect: std::cell::Cell::new(true),
            editor_windows: RefCell::new(HashMap::new()),
            cards: RefCell::new(Vec::new()),
            back_cb: RefCell::new(None),
            master_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        inner.btn_back.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        let i2 = inner.clone();
        inner.header_menu_btn.connect_clicked(move |_| show_header_menu(&i2));

        let i3 = inner.clone();
        inner.master_switch.connect_toggled(Box::new(move |val| {
            i3.is_master_active.set(val);
            if let Some(cb) = i3.master_cb.borrow().as_ref() {
                cb(val);
            }
        }));

        let i4 = inner.clone();
        inner.auto_switch.connect_toggled(Box::new(move |val| {
            i4.auto_detect.set(val);
        }));

        let i5 = inner.clone();
        inner.add_ext_btn.connect_clicked(move |_| {
            let dlg = AddExtensionDialog::new();
            let i6 = i5.clone();
            dlg.connect_accepted(Box::new(move || load_extensions(&i6)));
            dlg.root.present();
        });

        let i7 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i7)));

        load_extensions(&inner);
        retranslate_ui(&inner);

        ExtensionsView { inner }
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

    pub fn set_active(&self, active: bool) {
        self.inner.is_master_active.set(active);
        self.inner.master_switch.set_checked(active, false);
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }


    pub fn load_extensions(&self) {
        load_extensions(&self.inner);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.btn_back.set_label(&i18n::tr("btn_back"));
    inner.title_lbl.set_text(&i18n::tr("ext_title"));
    inner.desc_lbl.set_text(&i18n::tr("ext_desc"));
    inner.link_btn.set_label(&i18n::tr("ext_link_learn"));
    inner.auto_title.set_text(&i18n::tr("ext_auto_detect_t"));
    inner.auto_sub.set_text(&i18n::tr("ext_auto_detect_d"));
    inner.section_title.set_text(&i18n::tr("ext_installed_head"));
    inner.add_ext_btn.set_label(&i18n::tr("ext_btn_add"));
    let cards = inner.cards.borrow().clone();
    for card in cards {
        card.retranslate_ui();
    }
}

fn show_header_menu(inner: &Rc<Inner>) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("detail-menu-popover");
    popover.set_parent(&inner.header_menu_btn);

    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    let act_update = gtk4::Button::new();
    act_update.set_label(&i18n::tr("nav_updates"));
    act_update.add_css_class("detail-menu-item");
    let p1 = popover.clone();
    let i1 = inner.clone();
    act_update.connect_clicked(move |_| {
        p1.hide();
        check_for_updates(&i1);
    });
    menu_box.append(&act_update);

    let act_reset = gtk4::Button::new();
    act_reset.set_label(&i18n::tr("menu_reset_default"));
    act_reset.add_css_class("detail-menu-item");
    let p2 = popover.clone();
    let i2 = inner.clone();
    act_reset.connect_clicked(move |_| {
        p2.hide();
        reset_to_default(&i2);
    });
    menu_box.append(&act_reset);

    popover.set_child(Some(&menu_box));
    popover.show();
}

fn check_for_updates(inner: &Rc<Inner>) {
    let bin = cli_bin();
    match std::process::Command::new(&bin).arg("check-update").spawn() {
        Ok(_) => {
            let is_en = i18n::get_current_language() == i18n::LANG_EN;
            let title = if is_en { "Extensions Update" } else { "Cập nhật tiện ích" };
            let msg = if is_en {
                "Checking for userscript updates in background."
            } else {
                "Đang kiểm tra cập nhật tiện ích trong nền."
            };
            show_message(None, title, msg);
        }
        Err(e) => {
            show_message(None, "Error", &format!("Failed to check updates: {}", e));
        }
    }
    let _ = inner;
}

// QMessageBox.question -> GTK window with Yes/No buttons.
fn ask_yes_no(title: &str, body: &str, on_yes: Box<dyn Fn()>) {
    let dlg = crate::util::sub_window(title);
    dlg.set_default_size(380, 140);
    dlg.add_css_class("dlg-dark");
    let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    box_.set_margin_top(24);
    box_.set_margin_bottom(20);
    box_.set_margin_start(24);
    box_.set_margin_end(24);

    let lbl = gtk4::Label::new(Some(body));
    lbl.add_css_class("dlg-title");
    lbl.set_wrap(true);
    lbl.set_xalign(0.0);
    box_.append(&lbl);

    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    row.set_halign(gtk4::Align::End);

    let btn_no = gtk4::Button::new();
    btn_no.set_label("No");
    btn_no.add_css_class("outline-btn");
    let d1 = dlg.clone();
    btn_no.connect_clicked(move |_| d1.close());
    row.append(&btn_no);

    let btn_yes = gtk4::Button::new();
    btn_yes.set_label("Yes");
    btn_yes.add_css_class("green-btn");
    let d2 = dlg.clone();
    btn_yes.connect_clicked(move |_| {
        d2.close();
        on_yes();
    });
    row.append(&btn_yes);

    box_.append(&row);
    dlg.set_child(Some(&box_));
    dlg.present();
}

fn reset_to_default(inner: &Rc<Inner>) {
    let i = inner.clone();
    ask_yes_no(
        "Reset Extensions",
        "Are you sure you want to reset extensions to default settings?",
        Box::new(move || load_extensions(&i)),
    );
}

// Python strftime "%B %d, %Y at %H:%M" via localtime (English month names).
fn format_time_str(ts: i64) -> String {
    const MONTHS: [&str; 12] = [
        "January", "February", "March", "April", "May", "June", "July", "August", "September",
        "October", "November", "December",
    ];
    unsafe {
        let t = ts as libc::time_t;
        let tm = libc::localtime(&t);
        if tm.is_null() {
            return "Recently".to_string();
        }
        let tm = &*tm;
        format!(
            "{} {:02}, {} at {:02}:{:02}",
            MONTHS[tm.tm_mon as usize],
            tm.tm_mday,
            tm.tm_year + 1900,
            tm.tm_hour,
            tm.tm_min
        )
    }
}

fn load_extensions(inner: &Rc<Inner>) {
    // Clear existing cards.
    while let Some(child) = inner.list_container.first_child() {
        inner.list_container.remove(&child);
    }
    inner.cards.borrow_mut().clear();

    let dir = scripts_dir();
    let bin = cli_bin();

    // 1. Read CLI list to get enabled/disabled status.
    let mut cli_status: HashMap<String, bool> = HashMap::new();
    if let Ok(res) = std::process::Command::new(&bin)
        .args(["userscripts", "list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&res.stdout);
        let re_id = Regex::new(r"ID:\s*([a-zA-Z0-9_-]+)").unwrap();
        let mut current_enabled = false;
        for line in stdout.lines() {
            if line.contains('[') && line.contains(']') && line.contains('|') {
                current_enabled = line.to_lowercase().contains("[x]");
            }
            if let Some(caps) = re_id.captures(line) {
                cli_status.insert(caps[1].trim().to_string(), current_enabled);
            }
        }
    }

    // 2. Scan meta.json files.
    let mut installed: Vec<ExtensionData> = Vec::new();
    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if !fname.ends_with(".meta.json") {
                    continue;
                }
                let script_id = fname.trim_end_matches(".meta.json").to_string();
                let meta_path = dir.join(&fname);
                let js_path = dir.join(format!("{}.user.js", script_id));

                let content = std::fs::read_to_string(&meta_path).unwrap_or_default();
                let meta: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);

                let name = meta
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&script_id)
                    .to_string();
                let version = meta
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0")
                    .to_string();
                let homepage = meta
                    .get("homepageURL")
                    .and_then(|v| v.as_str())
                    .or_else(|| meta.get("homepage").and_then(|v| v.as_str()))
                    .or_else(|| meta.get("website").and_then(|v| v.as_str()))
                    .unwrap_or("")
                    .to_string();
                let description = meta
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let time_str = match entry.metadata() {
                    Ok(md) => match md.modified() {
                        Ok(t) => match t.duration_since(std::time::UNIX_EPOCH) {
                            Ok(d) => format_time_str(d.as_secs() as i64),
                            Err(_) => "Recently".to_string(),
                        },
                        Err(_) => "Recently".to_string(),
                    },
                    Err(_) => "Recently".to_string(),
                };

                let enabled = *cli_status.get(&script_id).unwrap_or(&true);

                installed.push(ExtensionData {
                    id: script_id,
                    name,
                    version,
                    time_str,
                    homepage,
                    description,
                    meta_path,
                    js_path,
                    enabled,
                });
            }
        }
    }

    // Known standard default extensions.
    let known_defaults: Vec<ExtensionData> = vec![
        ExtensionData {
            id: "adguard-assistant".to_string(),
            name: "AdGuard Assistant".to_string(),
            version: "4.4.15".to_string(),
            time_str: "September 5th, 2026 at 05:33".to_string(),
            homepage: "https://adguard.com/assistant.html".to_string(),
            description: String::new(),
            meta_path: dir.join("adguard-assistant.meta.json"),
            js_path: dir.join("adguard-assistant.user.js"),
            enabled: true,
        },
        ExtensionData {
            id: "adguard-extra".to_string(),
            name: "AdGuard Extra".to_string(),
            version: "1.1.38".to_string(),
            time_str: "September 5th, 2026 at 05:33".to_string(),
            homepage: "https://github.com/AdguardTeam/AdguardExtra".to_string(),
            description: String::new(),
            meta_path: dir.join("adguard-extra.meta.json"),
            js_path: dir.join("adguard-extra.user.js"),
            enabled: true,
        },
        ExtensionData {
            id: "adguard-popup-blocker".to_string(),
            name: "AdGuard Popup Blocker".to_string(),
            version: "2.5.117".to_string(),
            time_str: "August 22nd, 2026 at 06:46".to_string(),
            homepage: "https://github.com/AdguardTeam/PopupBlocker".to_string(),
            description: String::new(),
            meta_path: dir.join("adguard-popup-blocker.meta.json"),
            js_path: dir.join("adguard-popup-blocker.user.js"),
            enabled: true,
        },
        ExtensionData {
            id: "web-of-trust".to_string(),
            name: "Web of Trust".to_string(),
            version: "1.1.37".to_string(),
            time_str: "August 22nd, 2026 at 06:46".to_string(),
            homepage: "https://www.mywot.com".to_string(),
            description: String::new(),
            meta_path: dir.join("web-of-trust.meta.json"),
            js_path: dir.join("web-of-trust.user.js"),
            enabled: true,
        },
    ];

    let installed_ids: Vec<String> = installed.iter().map(|i| i.id.clone()).collect();
    for default_item in known_defaults {
        if !installed_ids.contains(&default_item.id) {
            installed.push(default_item);
        }
    }

    for ext in installed {
        let card = ExtensionCard::new(ext);

        let i1 = inner.clone();
        card.connect_toggled(Box::new(move |ext_id, is_enabled| {
            let bin = cli_bin();
            let cmd = if is_enabled { "enable" } else { "disable" };
            let _ = std::process::Command::new(&bin)
                .args(["userscripts", cmd, &ext_id])
                .output();
            let _ = i1;
        }));

        let i2 = inner.clone();
        card.connect_menu_requested(Box::new(move |action, data| {
            handle_menu_action(&i2, action, data);
        }));

        inner.cards.borrow_mut().push(card.clone());
        inner.list_container.append(card.root());
    }
}

fn handle_menu_action(inner: &Rc<Inner>, action: &str, data: &ExtensionData) {
    if action == "Homepage" {
        let url = if !data.homepage.is_empty() {
            data.homepage.clone()
        } else {
            "https://adguard.com/kb/general/extensions/".to_string()
        };
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    } else if action == "Reinstall" {
        show_message(
            None,
            "Reinstall",
            &format!("Reinstalling '{}'...", data.name),
        );
        load_extensions(inner);
    } else if action == "Edit" {
        open_script_editor(inner, data);
    } else if action == "Storage" {
        open_script_storage(inner, data);
    } else if action == "Remove" {
        let i = inner.clone();
        let d = data.clone();
        ask_yes_no(
            "Remove Extension",
            &format!("Are you sure you want to remove '{}'?", d.name),
            Box::new(move || {
                let bin = cli_bin();
                let _ = std::process::Command::new(&bin)
                    .args(["userscripts", "remove", &d.id])
                    .output();
                if d.js_path.exists() {
                    let _ = std::fs::remove_file(&d.js_path);
                }
                if d.meta_path.exists() {
                    let _ = std::fs::remove_file(&d.meta_path);
                }
                load_extensions(&i);
            }),
        );
    }
}

fn open_script_editor(inner: &Rc<Inner>, data: &ExtensionData) {
    let existing = inner.editor_windows.borrow().get(&data.id).cloned();
    if let Some(win) = existing {
        if win.get_visible() {
            win.present();
            return;
        }
    }

    let win = build_editor_window(data);
    inner
        .editor_windows
        .borrow_mut()
        .insert(data.id.clone(), win.clone());
    win.present();
}

fn build_editor_window(data: &ExtensionData) -> gtk4::Window {
    let win = crate::util::sub_window("");
    win.set_title(Some(&format!("Edit - {}", data.name)));
    win.set_default_size(750, 550);

    let layout = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    layout.set_margin_top(16);
    layout.set_margin_bottom(16);
    layout.set_margin_start(16);
    layout.set_margin_end(16);
    win.set_child(Some(&layout));

    // Header.
    let head = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    let title_lbl = gtk4::Label::new(Some(&format!("Editing: {}", data.name)));
    title_lbl.add_css_class("lbl-16-bold-white");
    head.append(&title_lbl);

    let save_btn = gtk4::Button::new();
    save_btn.set_label("Save changes");
    save_btn.add_css_class("green-btn");
    head.append(&save_btn);
    layout.append(&head);

    // Code text area.
    let code_edit = gtk4::TextView::new();
    code_edit.add_css_class("ext-code-view");
    code_edit.set_vexpand(true);
    layout.append(&code_edit);

    // Load code.
    if data.js_path.exists() {
        match std::fs::read_to_string(&data.js_path) {
            Ok(content) => code_edit.buffer().set_text(&content),
            Err(e) => code_edit
                .buffer()
                .set_text(&format!("// Error reading file: {}", e)),
        }
    } else {
        let default_template = format!(
            "// ==UserScript==\n// @name {}\n// @version {}\n// ==/UserScript==\n\n(function() {{\n    'use strict';\n    // Code here...\n}});",
            data.name, data.version
        );
        code_edit.buffer().set_text(&default_template);
    }

    let js_path = data.js_path.clone();
    let w1 = win.clone();
    let tv = code_edit.clone();
    save_btn.connect_clicked(move |_| {
        if js_path.as_os_str().is_empty() {
            show_message(Some(&w1), "Error", "No valid file path to save code.");
            return;
        }
        if let Some(parent) = js_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let start = tv.buffer().start_iter();
        let end = tv.buffer().end_iter();
        let text = tv.buffer().text(&start, &end, false).to_string();
        match std::fs::write(&js_path, text) {
            Ok(_) => show_message(Some(&w1), "Saved", "Userscript saved successfully."),
            Err(e) => show_message(Some(&w1), "Error", &format!("Failed to save script: {}", e)),
        }
    });

    win
}

// Deviation: QDialog.exec() -> non-modal window.
fn open_script_storage(inner: &Rc<Inner>, data: &ExtensionData) {
    let content = match std::fs::read_to_string(&data.meta_path) {
        Ok(c) => c,
        Err(e) => e.to_string(),
    };

    let dlg = crate::util::sub_window("");
    dlg.set_title(Some(&format!("Storage - {}", data.name)));
    dlg.set_default_size(500, 400);
    dlg.add_css_class("dlg-dark");

    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    dlg.set_child(Some(&vbox));

    let lbl = gtk4::Label::new(Some(&format!("Metadata for {}:", data.name)));
    lbl.add_css_class("lbl-13-bold-white");
    lbl.set_halign(gtk4::Align::Start);
    vbox.append(&lbl);

    let txt = gtk4::TextView::new();
    txt.set_editable(false);
    txt.add_css_class("ext-storage-view");
    txt.set_vexpand(true);
    txt.buffer().set_text(&content);
    vbox.append(&txt);

    let btn_close = gtk4::Button::new();
    btn_close.set_label("Close");
    btn_close.add_css_class("green-btn");
    let d1 = dlg.clone();
    btn_close.connect_clicked(move |_| d1.close());
    vbox.append(&btn_close);

    dlg.present();
    let _ = inner;
}

// ---------------------------------------------------------------------------
// AddExtensionDialog
// ---------------------------------------------------------------------------
pub struct AddExtensionDialog {
    pub root: gtk4::Window,
    pub url_input: gtk4::Entry,
    accepted_cb: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl AddExtensionDialog {
    pub fn new() -> AddExtensionDialog {
        let root = crate::util::sub_window("Add extension");
        root.set_default_size(460, 240);
        root.add_css_class("dlg-dark");

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(20);
        vbox.set_margin_start(24);
        vbox.set_margin_end(24);
        root.set_child(Some(&vbox));

        let title = gtk4::Label::new(Some("Add custom extension"));
        title.add_css_class("dlg-title-18");
        title.set_halign(gtk4::Align::Start);
        vbox.append(&title);

        let desc = gtk4::Label::new(Some(
            "Enter the URL of the userscript (.user.js) or select a file from disk:",
        ));
        desc.add_css_class("lbl-12-muted");
        desc.set_wrap(true);
        desc.set_xalign(0.0);
        vbox.append(&desc);

        // URL input row.
        let url_input = gtk4::Entry::new();
        url_input.set_placeholder_text(Some("https://example.com/script.user.js"));
        url_input.add_css_class("dlg-input");
        vbox.append(&url_input);

        // File browse row.
        let file_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let file_lbl = gtk4::Label::new(Some("Or choose local file:"));
        file_lbl.add_css_class("lbl-12-muted");
        file_row.append(&file_lbl);

        let browse_btn = gtk4::Button::new();
        browse_btn.set_label("Browse .user.js...");
        browse_btn.add_css_class("ext-browse-btn");
        let entry = url_input.clone();
        browse_btn.connect_clicked(move |_| {
            let dlg = gtk4::FileChooserDialog::new(
                Some("Open Userscript"),
                None::<&gtk4::Window>,
                gtk4::FileChooserAction::Open,
                &[
                    ("Cancel", gtk4::ResponseType::Cancel),
                    ("Open", gtk4::ResponseType::Accept),
                ],
            );
            let filter = gtk4::FileFilter::new();
            filter.add_pattern("*.user.js");
            filter.add_pattern("*.js");
            dlg.add_filter(&filter);
            dlg.set_default_size(500, 350);
            let e1 = entry.clone();
            dlg.connect_response(move |d, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(f) = d.file() {
                        if let Some(p) = f.path() {
                            e1.set_text(&p.to_string_lossy());
                        }
                    }
                }
                d.close();
            });
            dlg.present();
        });
        file_row.append(&browse_btn);
        vbox.append(&file_row);

        // Action buttons.
        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);

        let cancel_btn = gtk4::Button::new();
        cancel_btn.set_label("Cancel");
        cancel_btn.add_css_class("cr-cancel-btn");
        let d1 = root.clone();
        cancel_btn.connect_clicked(move |_| d1.close());
        btn_box.append(&cancel_btn);

        let install_btn = gtk4::Button::new();
        install_btn.set_label("Install");
        install_btn.add_css_class("green-btn");
        btn_box.append(&install_btn);
        vbox.append(&btn_box);

        let dialog = AddExtensionDialog {
            root,
            url_input,
            accepted_cb: Rc::new(RefCell::new(None)),
        };

        let entry = dialog.url_input.clone();
        let w = dialog.root.clone();
        let acc = dialog.accepted_cb.clone();
        install_btn.connect_clicked(move |_| {
            let target = entry.text().trim().to_string();
            if target.is_empty() {
                show_message(Some(&w), "Warning", "Please enter a userscript URL or select a file.");
                return;
            }

            let bin = cli_bin();

            if target.starts_with("http://") || target.starts_with("https://") {
                match std::process::Command::new(&bin)
                    .args(["userscripts", "install", &target])
                    .output()
                {
                    Ok(res) => {
                        if res.status.code() == Some(0) {
                            show_message(Some(&w), "Success", "Userscript installed successfully!");
                            if let Some(cb) = acc.borrow().as_ref() {
                                cb();
                            }
                            w.close();
                        } else {
                            let err = String::from_utf8_lossy(&res.stderr).trim().to_string();
                            let err = if err.is_empty() {
                                String::from_utf8_lossy(&res.stdout).trim().to_string()
                            } else {
                                err
                            };
                            show_message(Some(&w), "Error", &format!("Failed to install userscript:\n{}", err));
                        }
                    }
                    Err(e) => {
                        show_message(Some(&w), "Error", &format!("Execution error: {}", e));
                    }
                }
            } else if std::path::Path::new(&target).exists() {
                let dir = scripts_dir();
                let _ = std::fs::create_dir_all(&dir);
                let basename = std::path::Path::new(&target)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let dest = dir.join(&basename);
                match std::fs::copy(&target, &dest) {
                    Ok(_) => {
                        let script_id = basename
                            .replace(".user.js", "")
                            .replace(".js", "");
                        let meta_dest = dir.join(format!("{}.meta.json", script_id));
                        if !meta_dest.exists() {
                            let capitalized = capitalize(&script_id);
                            let _ = std::fs::write(
                                &meta_dest,
                                format!("{{\n    \"name\": \"{}\",\n    \"version\": \"1.0.0\"\n}}", capitalized),
                            );
                        }
                        show_message(
                            Some(&w),
                            "Success",
                            &format!("Userscript '{}' installed successfully!", basename),
                        );
                        if let Some(cb) = acc.borrow().as_ref() {
                            cb();
                        }
                        w.close();
                    }
                    Err(e) => {
                        show_message(Some(&w), "Error", &format!("Failed to copy file: {}", e));
                    }
                }
            } else {
                show_message(Some(&w), "Warning", "Invalid URL or file path.");
            }
        });

        dialog
    }

    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.accepted_cb.borrow_mut() = Some(cb);
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
