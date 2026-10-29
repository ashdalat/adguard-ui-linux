// Port of adguard_modern/widgets/dns_controls.py
use cairo;
use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::constants as c;

// ---- RadioButtonCircle (20x20, painted) ----
struct RadioInner {
    root: gtk4::DrawingArea,
    checked: Cell<bool>,
}

#[derive(Clone)]
pub struct RadioButtonCircle {
    inner: Rc<RadioInner>,
}

impl RadioButtonCircle {
    pub fn new(checked: bool) -> RadioButtonCircle {
        let root = gtk4::DrawingArea::new();
        root.set_size_request(20, 20);
        let inner = Rc::new(RadioInner {
            root,
            checked: Cell::new(checked),
        });
        let r = RadioButtonCircle { inner: inner.clone() };
        let r2 = inner.clone();
        inner.root.set_draw_func(move |_a, cr, w, h| {
            cr.set_antialias(cairo::Antialias::Default);
            // Qt rect().adjusted(1,1,-1,-1)
            let inset = 1.0;
            let rw = (w as f64) - 2.0 * inset;
            let rh = (h as f64) - 2.0 * inset;
            if r2.checked.get() {
                cr.set_line_width(2.0);
                cr.set_source_rgb(0x5E as f64 / 255.0, 0xBB as f64 / 255.0, 0x6A as f64 / 255.0);
                ellipse_path(cr, inset, inset, rw, rh);
                cr.stroke();
                // dot: rect adjusted(5,5,-5,-5)
                cr.arc((w as f64) / 2.0, (h as f64) / 2.0, (rw / 2.0) - 4.0, 0.0, std::f64::consts::PI * 2.0);
                cr.fill();
            } else {
                cr.set_line_width(1.5);
                cr.set_source_rgb(0x66 as f64 / 255.0, 0x66 as f64 / 255.0, 0x6A as f64 / 255.0);
                ellipse_path(cr, inset, inset, rw, rh);
                cr.stroke();
            }
        });
        r
    }

    pub fn root(&self) -> &gtk4::DrawingArea {
        &self.inner.root
    }

    pub fn is_checked(&self) -> bool {
        self.inner.checked.get()
    }

    pub fn set_checked(&self, checked: bool) {
        if self.inner.checked.get() != checked {
            self.inner.checked.set(checked);
            self.inner.root.queue_draw();
        }
    }
}

// ---- DnsServerRow ----
struct RowInner {
    root: gtk4::Box,
    server_id: String,
    upstream: String,
    has_drilldown: bool,
    radio: RadioButtonCircle,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    selected_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    drilldown_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct DnsServerRow {
    inner: Rc<RowInner>,
}

impl DnsServerRow {
    pub fn new(
        server_id: &str,
        title: &str,
        description: &str,
        upstream: &str,
        has_drilldown: bool,
    ) -> DnsServerRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("dns-server-row");
        root.set_margin_start(12);
        root.set_margin_end(16);
        root.set_margin_top(10);
        root.set_margin_bottom(10);

        let radio = RadioButtonCircle::new(false);
        root.append(radio.root());

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        text.set_hexpand(true);
        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text.append(&lbl_title);
        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_halign(gtk4::Align::Start);
        text.append(&lbl_desc);
        root.append(&text);

        let inner = Rc::new(RowInner {
            root,
            server_id: server_id.to_string(),
            upstream: upstream.to_string(),
            has_drilldown,
            radio,
            lbl_title: lbl_title.clone(),
            lbl_desc: lbl_desc.clone(),
            selected_cb: RefCell::new(None),
            drilldown_cb: RefCell::new(None),
        });
        let row = DnsServerRow { inner: inner.clone() };

        if has_drilldown {
            let divider = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            divider.set_size_request(1, 30);
            divider.add_css_class("divider-v");
            inner.root.append(&divider);

            let chevron = gtk4::Button::new();
            chevron.add_css_class("chevron-btn");
            chevron.set_size_request(24, 24);
            chevron.set_label("›");
            let i2 = inner.clone();
            let sid = inner.server_id.clone();
            chevron.connect_clicked(move |_| {
                if let Some(cb) = i2.drilldown_cb.borrow().as_ref() {
                    cb(&sid);
                }
            });
            inner.root.append(&chevron);
        }

        // mousePressEvent: right part of the row = drilldown, otherwise select
        let gesture = gtk4::GestureClick::new();
        let r2 = row.clone();
        gesture.connect_pressed(move |_g, n, x, _y| {
            if n != 1 {
                return;
            }
            let sid = r2.inner.server_id.clone();
            let width = r2.inner.root.width();
            if r2.inner.has_drilldown && x > (width - 50) as f64 {
                if let Some(cb) = r2.inner.drilldown_cb.borrow().as_ref() {
                    cb(&sid);
                }
            } else if let Some(cb) = r2.inner.selected_cb.borrow().as_ref() {
                cb(&sid);
            }
        });
        inner.root.add_controller(gesture);

        row
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_checked(&self, checked: bool) {
        self.inner.radio.set_checked(checked);
    }

    pub fn is_checked(&self) -> bool {
        self.inner.radio.is_checked()
    }

    pub fn connect_selected(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.selected_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_drilldown(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.drilldown_cb.borrow_mut() = Some(cb);
    }

    pub fn upstream(&self) -> &str {
        &self.inner.upstream
    }

    pub fn lbl_title(&self) -> &gtk4::Label {
        &self.inner.lbl_title
    }

    pub fn lbl_desc(&self) -> &gtk4::Label {
        &self.inner.lbl_desc
    }
}

// ---- DnsFeatureIcon (22x22, painted) ----
pub fn dns_feature_icon(icon_type: &str) -> gtk4::DrawingArea {
    let root = gtk4::DrawingArea::new();
    root.set_size_request(22, 22);
    let t = icon_type.to_string();
    root.set_draw_func(move |_a, cr, _w, _h| {
        cr.set_antialias(cairo::Antialias::Default);
        cr.set_line_width(1.8);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);
        cr.set_source_rgb(0x5E as f64 / 255.0, 0xBB as f64 / 255.0, 0x6A as f64 / 255.0);

        match t.as_str() {
            "ad_blocking" => {
                rounded_rect(cr, 3.0, 4.0, 16.0, 12.0, 2.0);
                cr.stroke();
                cr.move_to(2.0, 2.0);
                cr.line_to(20.0, 18.0);
                cr.stroke();
            }
            "secure" => {
                cr.arc(11.0, 11.0, 8.0, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.move_to(3.0, 11.0);
                cr.line_to(19.0, 11.0);
                cr.stroke();
                ellipse_path(cr, 7.0, 3.0, 8.0, 16.0);
                cr.stroke();
            }
            "no_logs" => {
                cr.arc(11.0, 11.0, 8.0, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.arc(11.0, 11.0, 2.0, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.move_to(2.0, 3.0);
                cr.line_to(20.0, 19.0);
                cr.stroke();
            }
            "encryption" => {
                rounded_rect(cr, 4.0, 9.0, 14.0, 10.0, 2.0);
                cr.stroke();
                cr.arc(11.0, 8.0, 4.0, std::f64::consts::PI, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.set_line_width(1.0);
                cr.move_to(11.0, 13.0);
                cr.line_to(11.0, 15.0);
                cr.stroke();
            }
            "adult_blocking" => {
                cr.arc(6.5, 14.5, 3.5, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.move_to(6.0, 8.0);
                cr.line_to(13.0, 15.0);
                cr.stroke();
                cr.arc(14.0, 8.0, 5.0, -std::f64::consts::PI / 6.0, std::f64::consts::PI + 5.0 * std::f64::consts::PI / 6.0);
                cr.stroke();
            }
            "safe_search" => {
                cr.arc(9.0, 9.0, 6.0, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.move_to(12.0, 12.0);
                cr.line_to(19.0, 19.0);
                cr.stroke();
                cr.move_to(6.0, 9.0);
                cr.line_to(8.0, 11.0);
                cr.line_to(12.0, 6.0);
                cr.stroke();
            }
            _ => {
                cr.arc(11.0, 11.0, 8.0, 0.0, std::f64::consts::PI * 2.0);
                cr.stroke();
                cr.move_to(7.0, 11.0);
                cr.line_to(10.0, 14.0);
                cr.line_to(15.0, 8.0);
                cr.stroke();
            }
        }
    });
    root
}

pub fn dns_feature_row(icon_type: &str, title: &str, description: &str) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    row.set_margin_top(6);
    row.set_margin_bottom(6);
    row.append(&dns_feature_icon(icon_type));

    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    text.set_hexpand(true);
    let lbl_title = gtk4::Label::new(Some(title));
    lbl_title.add_css_class("lbl-title");
    lbl_title.set_halign(gtk4::Align::Start);
    text.append(&lbl_title);
    let lbl_desc = gtk4::Label::new(Some(description));
    lbl_desc.add_css_class("lbl-desc-9");
    lbl_desc.set_wrap(true);
    lbl_desc.set_halign(gtk4::Align::Start);
    text.append(&lbl_desc);
    row.append(&text);
    row
}

// Port of ProtocolCheckmarkDelegate. GTK4's DropDown has no checkmark delegate,
// so the plain dropdown is the closest equivalent (recorded as a deviation).
pub fn combo_with_checkmark() -> gtk4::DropDown {
    let combo = gtk4::DropDown::new(None::<gtk4::StringList>, None::<gtk4::Expression>);
    combo.add_css_class("combo-no-wheel");
    combo
}

// cairo-rs has no ellipse(); the outline is approximated with four bezier arcs.
fn ellipse_path(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64) {
    const K: f64 = 0.5522847498;
    let rx = w / 2.0;
    let ry = h / 2.0;
    let cx = x + rx;
    let cy = y + ry;
    cr.move_to(cx, cy - ry);
    cr.curve_to(cx + K * rx, cy - ry, cx + rx, cy - K * ry, cx + rx, cy);
    cr.curve_to(cx + rx, cy + K * ry, cx + K * rx, cy + ry, cx, cy + ry);
    cr.curve_to(cx - K * rx, cy + K * ry, cx - rx, cy + K * ry, cx - rx, cy);
    cr.curve_to(cx - K * rx, cy - K * ry, cx - K * rx, cy - ry, cx, cy - ry);
    cr.close_path();
}

// ---- AddCustomDnsDialog ----
pub struct AddCustomDnsDialog {
    pub root: gtk4::Window,
    pub input_name: gtk4::Entry,
    pub input_url: gtk4::Entry,
    accepted_cb: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl AddCustomDnsDialog {
    pub fn new() -> AddCustomDnsDialog {
        let root = crate::util::sub_window("");
        root.set_default_size(500, 310);
        root.add_css_class("dlg-dark");

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        col.set_margin_start(28);
        col.set_margin_end(28);
        col.set_margin_top(24);
        col.set_margin_bottom(24);
        root.set_child(Some(&col));

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Add custom server"));
        lbl_title.add_css_class("dlg-title-18");
        lbl_title.set_halign(gtk4::Align::Start);
        header.append(&lbl_title);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(28, 28);
        btn_close.set_label("✕");
        let r2 = root.clone();
        btn_close.connect_clicked(move |_| r2.set_visible(false));
        header.append(&btn_close);
        col.append(&header);

        let lbl_name = muted_label("Server name");
        col.append(&lbl_name);
        let input_name = entry("My DNS server");
        col.append(&input_name);

        let lbl_addr = muted_label("Server address");
        col.append(&lbl_addr);
        let input_url = entry("IP address, sdns://, https://, or tls://");
        col.append(&input_url);

        let inner_acc: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        let btn_add = gtk4::Button::new();
        btn_add.add_css_class("green-btn");
        btn_add.set_size_request(-1, 42);
        btn_add.set_label("Add");
        let r3 = root.clone();
        let acc = inner_acc.clone();
        btn_add.connect_clicked(move |_| {
            r3.set_visible(false);
            if let Some(cb) = acc.borrow().as_ref() {
                cb();
            }
        });
        btn_row.append(&btn_add);

        let btn_cancel = gtk4::Button::new();
        btn_cancel.add_css_class("outline-btn");
        btn_cancel.set_size_request(-1, 42);
        btn_cancel.set_label("Cancel");
        let r4 = root.clone();
        btn_cancel.connect_clicked(move |_| r4.set_visible(false));
        btn_row.append(&btn_cancel);
        col.append(&btn_row);

        AddCustomDnsDialog {
            root,
            input_name,
            input_url,
            accepted_cb: inner_acc,
        }
    }

    /// Port of the QDialog.exec() == Accepted path: the Add button resolves the
    /// dialog (non-modal window deviation) and fires this callback.
    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.accepted_cb.borrow_mut() = Some(cb);
    }

    /// Port of get_server_info()
    pub fn get_server_info(&self) -> (String, String) {
        let name = self.input_name.text().trim().to_string();
        let name = if name.is_empty() { "My DNS server".to_string() } else { name };
        (name, self.input_url.text().trim().to_string())
    }
}

fn entry(placeholder: &str) -> gtk4::Entry {
    let e = gtk4::Entry::new();
    e.set_placeholder_text(Some(placeholder));
    e.set_size_request(-1, 42);
    e.add_css_class("input-dark");
    e
}

fn muted_label(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("lbl-desc-9");
    l.set_halign(gtk4::Align::Start);
    l
}

// ---- SelectKnownDnsDialog ----
pub struct SelectKnownDnsDialog {
    pub root: gtk4::Window,
    selected: Rc<RefCell<Option<(String, String)>>>,
}

impl SelectKnownDnsDialog {
    pub fn new() -> SelectKnownDnsDialog {
        let root = crate::util::sub_window("");
        root.set_default_size(540, 480);
        root.add_css_class("dlg-dark");

        let col = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        col.set_margin_start(24);
        col.set_margin_end(24);
        col.set_margin_top(20);
        col.set_margin_bottom(20);
        root.set_child(Some(&col));

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_title = gtk4::Label::new(Some("Known DNS providers"));
        lbl_title.add_css_class("dlg-title-16");
        lbl_title.set_halign(gtk4::Align::Start);
        header.append(&lbl_title);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        let btn_close = gtk4::Button::new();
        btn_close.add_css_class("dlg-close");
        btn_close.set_size_request(28, 28);
        btn_close.set_label("✕");
        let r2 = root.clone();
        btn_close.connect_clicked(move |_| r2.set_visible(false));
        header.append(&btn_close);
        col.append(&header);

        let input_search = entry("Search providers (e.g. NextDNS, Mullvad, Control D)...");
        input_search.set_size_request(-1, 38);
        col.append(&input_search);

        let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        list_container.set_margin_top(4);
        list_container.set_margin_bottom(4);
        list_container.set_margin_end(8);
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&list_container));
        scroll.set_vexpand(true);
        col.append(&scroll);

        let btn_cancel = gtk4::Button::new();
        btn_cancel.add_css_class("outline-btn");
        btn_cancel.set_size_request(-1, 38);
        btn_cancel.set_label("Cancel");
        let r3 = root.clone();
        btn_cancel.connect_clicked(move |_| r3.set_visible(false));
        col.append(&btn_cancel);

        let dlg = SelectKnownDnsDialog {
            root,
            selected: Rc::new(RefCell::new(None)),
        };

        // Dependency: tabs/dns_data.rs (KNOWN_DNS_PROVIDERS_LIBRARY) — ported separately.
        let library = crate::tabs::dns_data::known_dns_providers();
        populate_list(&dlg, &list_container, &library);

        let lc = list_container.clone();
        let d2 = {
            let sel = dlg.selected.clone();
            let r = dlg.root.clone();
            let lib = library.clone();
            (lc, sel, r, lib)
        };
        input_search.connect_changed(move |e| {
            let q = e.text().trim().to_lowercase();
            let filtered: Vec<_> = if q.is_empty() {
                d2.3.clone()
            } else {
                d2.3
                    .iter()
                    .filter(|it| it.name.to_lowercase().contains(&q) || it.desc.to_lowercase().contains(&q))
                    .cloned()
                    .collect()
            };
            populate_list_records(&d2.1, &d2.0, &filtered);
        });

        dlg
    }

    pub fn get_selected_server(&self) -> Option<(String, String)> {
        self.selected.borrow().clone()
    }
}

#[derive(Clone)]
pub struct DnsProvider {
    pub name: String,
    pub desc: String,
}

fn populate_list(dlg: &SelectKnownDnsDialog, container: &gtk4::Box, items: &[DnsProvider]) {
    populate_list_records(&dlg.selected, container, items);
}

fn populate_list_records(
    selected: &Rc<RefCell<Option<(String, String)>>>,
    container: &gtk4::Box,
    items: &[DnsProvider],
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    for item in items {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        row.add_css_class("provider-row");
        row.set_margin_start(12);
        row.set_margin_end(12);
        row.set_margin_top(10);
        row.set_margin_bottom(10);

        let text = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        text.set_hexpand(true);
        let lbl_name = gtk4::Label::new(Some(&item.name));
        lbl_name.add_css_class("lbl-title");
        lbl_name.set_halign(gtk4::Align::Start);
        text.append(&lbl_name);
        let lbl_desc = gtk4::Label::new(Some(&item.desc));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_wrap(true);
        lbl_desc.set_halign(gtk4::Align::Start);
        text.append(&lbl_desc);
        row.append(&text);

        let btn_add = gtk4::Button::new();
        btn_add.add_css_class("green-btn");
        btn_add.set_size_request(60, 30);
        btn_add.set_label("Add");
        let name = item.name.clone();
        let desc = item.desc.clone();
        let sel = selected.clone();
        btn_add.connect_clicked(move |_| {
            *sel.borrow_mut() = Some((name.clone(), desc.clone()));
        });
        row.append(&btn_add);

        container.append(&row);
    }
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.move_to(x + r, y);
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.arc(x + r, y + r, r, std::f64::consts::PI, std::f64::consts::FRAC_PI_2 * 3.0);
    cr.close_path();
}
