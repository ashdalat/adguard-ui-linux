// Port of adguard_modern/tabs/settings_support_view.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::widgets::common_controls::BackButton;
use crate::widgets::svg;

struct RowInner {
    root: gtk4::Box,
    key: String,
    url: String,
    click_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct SupportItemRow {
    inner: Rc<RowInner>,
}

impl SupportItemRow {
    pub fn new(key: &str, icon_name: &str, title: &str, description: &str, url: &str) -> SupportItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("support-row");
        root.set_margin_top(14);
        root.set_margin_bottom(14);
        root.set_margin_start(12);
        root.set_margin_end(16);

        let icon = svg::svg_image(icon_name, 22);
        icon.set_valign(gtk4::Align::Center);
        root.append(&icon);

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("support-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_desc);

        root.append(&text_box);

        let chevron = svg::svg_image("chevron_right.svg", 14);
        chevron.set_valign(gtk4::Align::Center);
        root.append(&chevron);

        let inner = Rc::new(RowInner {
            root,
            key: key.to_string(),
            url: url.to_string(),
            click_cb: RefCell::new(None),
        });

        // mousePressEvent: left click opens the URL then emits clicked(key).
        let i1 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            if n != 1 {
                return;
            }
            if !i1.url.is_empty() {
                let _ = std::process::Command::new("xdg-open").arg(&i1.url).spawn();
            }
            if let Some(cb) = i1.click_cb.borrow().as_ref() {
                cb(&i1.key);
            }
        });
        inner.root.add_controller(gesture);

        SupportItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.click_cb.borrow_mut() = Some(cb);
    }
}

struct Inner {
    root: gtk4::Box,
    btn_back: BackButton,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
}

pub struct SettingsSupportView {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsSupportView {
    pub fn new() -> SettingsSupportView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");
        root.set_margin_top(24);
        root.set_margin_bottom(24);
        root.set_margin_start(36);
        root.set_margin_end(36);

        let btn_back = BackButton::new("Back");
        root.append(&btn_back.root);
        root.append(&spacer(18));

        let lbl_title = gtk4::Label::new(Some("Support"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);
        root.append(&spacer(26));

        let items = [
            ("faq", "supp_faq.svg", "FAQ", "Most questions are already answered here",
             "https://adguard.com/kb/general/faq/"),
            ("contact", "supp_contact.svg", "Contact support",
             "Send bug reports, feature requests, and suggestions", "https://adguard.com/support.html"),
            ("report", "supp_report.svg", "Report incorrect blocking",
             "Report missed ads or “broken” websites", "https://adguard.com/reports/"),
            ("discuss", "supp_discuss.svg", "Discuss",
             "Check out AdGuard’s social networks or GitHub", "https://forum.adguard.com/"),
            ("rate", "supp_rate.svg", "Rate app", "Tell us what you think about AdGuard",
             "https://adguard.com/feedback.html"),
        ];
        for (key, icon_fn, title, desc, url) in items {
            let row = SupportItemRow::new(key, icon_fn, title, desc, url);
            root.append(row.root());
            root.append(&spacer(4));
        }

        let bb_root = btn_back.root.clone();
        let inner = Rc::new(Inner {
            root,
            btn_back,
            back_cb: RefCell::new(None),
        });

        let i1 = inner.clone();
        bb_root.connect_clicked(move |_| {
            if let Some(cb) = i1.back_cb.borrow().as_ref() {
                cb();
            }
        });

        SettingsSupportView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_back_requested(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }
}
