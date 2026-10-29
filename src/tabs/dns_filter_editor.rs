// Port of adguard_modern/tabs/dns_filter_editor.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use rusqlite::Connection;

use crate::i18n;
use crate::widgets::filtering_log_window::show_message;

fn home_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn dns_user_rules_file() -> std::path::PathBuf {
    home_dir().join(".local/share/adguard-cli/dns_user.txt")
}

fn dns_db_path() -> std::path::PathBuf {
    home_dir().join(".local/share/adguard-cli/agflm_dns.db")
}

// ---------------------------------------------------------------------------
// MascotWidget (cairo drawing, 140x140)
// ---------------------------------------------------------------------------
fn hex(c: &str) -> (f64, f64, f64) {
    let h = c.trim_start_matches('#');
    (
        u32::from_str_radix(&h[0..2], 16).unwrap_or(0) as f64 / 255.0,
        u32::from_str_radix(&h[2..4], 16).unwrap_or(0) as f64 / 255.0,
        u32::from_str_radix(&h[4..6], 16).unwrap_or(0) as f64 / 255.0,
    )
}

fn rounded_rect(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.move_to(x + r, y);
    cr.line_to(x + w - r, y);
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.line_to(x + w, y + h - r);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.line_to(x + r, y + h);
    cr.arc(x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
    cr.line_to(x, y + r);
    cr.arc(x + r, y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2);
    cr.close_path();
}

fn ellipse(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64) {
    // cairo has no ellipse(): approximate with bezier curves.
    let k = 0.5522847498;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    cr.move_to(cx - rx, cy);
    cr.curve_to(cx - rx, cy - ry * k, cx - rx * k, cy - ry, cx, cy - ry);
    cr.curve_to(cx + rx * k, cy - ry, cx + rx, cy - ry * k, cx + rx, cy);
    cr.curve_to(cx + rx, cy + ry * k, cx + rx * k, cy + ry, cx, cy + ry);
    cr.curve_to(cx - rx * k, cy + ry, cx - rx, cy + ry * k, cx - rx, cy);
    cr.close_path();
}

pub fn mascot_widget() -> gtk4::DrawingArea {
    let area = gtk4::DrawingArea::new();
    area.set_size_request(140, 140);
    area.set_draw_func(|_area, cr, _w, _h| {
        cr.set_antialias(cairo::Antialias::Best);

        // Cap
        cr.set_fill_rule(cairo::FillRule::Winding);
        let (r, g, b) = hex("#5EBB6A");
        cr.set_source_rgb(r, g, b);
        rounded_rect(cr, 35.0, 20.0, 50.0, 24.0, 12.0);
        cr.fill();

        // Face
        let (r, g, b) = hex("#FFE0BD");
        cr.set_source_rgb(r, g, b);
        rounded_rect(cr, 40.0, 32.0, 40.0, 36.0, 10.0);
        cr.fill();

        // Beard
        let (r, g, b) = hex("#D08D50");
        cr.set_source_rgb(r, g, b);
        rounded_rect(cr, 38.0, 52.0, 44.0, 22.0, 11.0);
        cr.fill();

        // Eyes / magnifying glass
        let (r, g, b) = hex("#FFFFFF");
        cr.set_source_rgb(r, g, b);
        let (pr, pg, pb) = hex("#333336");
        cr.set_source_rgb(pr, pg, pb);
        cr.set_line_width(3.0);
        ellipse(cr, 60.0, 30.0, 32.0, 32.0);
        cr.fill_preserve();
        cr.stroke();

        // Pupil inside glass
        let (r, g, b) = hex("#242427");
        cr.set_source_rgb(r, g, b);
        cr.set_line_width(1.0);
        ellipse(cr, 72.0, 42.0, 8.0, 8.0);
        cr.fill();

        // Handle
        let (pr, pg, pb) = hex("#333336");
        cr.set_source_rgb(pr, pg, pb);
        cr.set_line_width(4.0);
        cr.move_to(86.0, 56.0);
        cr.line_to(98.0, 70.0);
        cr.stroke();

        // Clothes
        let (r, g, b) = hex("#D9D9DC");
        cr.set_source_rgb(r, g, b);
        rounded_rect(cr, 30.0, 72.0, 60.0, 40.0, 8.0);
        cr.fill();

        // Shield
        let (r, g, b) = hex("#5EBB6A");
        cr.set_source_rgb(r, g, b);
        cr.move_to(20.0, 72.0);
        cr.line_to(52.0, 72.0);
        cr.line_to(52.0, 104.0);
        cr.line_to(36.0, 122.0);
        cr.line_to(20.0, 104.0);
        cr.close_path();
        cr.fill_preserve();
        let (pr, pg, pb) = hex("#FFFFFF");
        cr.set_source_rgb(pr, pg, pb);
        cr.set_line_width(2.5);
        cr.stroke();

        // White checkmark in shield
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(2.5);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);
        cr.move_to(27.0, 94.0);
        cr.line_to(34.0, 102.0);
        cr.stroke();
        cr.move_to(34.0, 102.0);
        cr.line_to(45.0, 86.0);
        cr.stroke();
    });
    area
}

// ---------------------------------------------------------------------------
// AddRuleSimpleDialog
// ---------------------------------------------------------------------------
pub struct AddRuleSimpleDialog {
    root: gtk4::Window,
    input_rule: gtk4::Entry,
    accepted_cb: RefCell<Option<Box<dyn Fn()>>>,
    rejected_cb: RefCell<Option<Box<dyn Fn()>>>,
}

impl AddRuleSimpleDialog {
    pub fn new(initial_text: &str) -> AddRuleSimpleDialog {
        let root = crate::util::sub_window("");
        root.set_title(Some(if initial_text.is_empty() {
            "Add DNS rule"
        } else {
            "Edit DNS rule"
        }));
        root.set_size_request(460, 220);

        // Deviation: Qt frameless translucent dialog -> regular GTK window with
        // the same dark frame styling.
        let frame = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        frame.add_css_class("dlg-dark");
        frame.set_margin_top(20);
        frame.set_margin_bottom(20);
        frame.set_margin_start(24);
        frame.set_margin_end(24);

        let lbl = gtk4::Label::new(Some(if initial_text.is_empty() {
            "Add DNS rule"
        } else {
            "Edit DNS rule"
        }));
        lbl.add_css_class("dlg-title-16");
        lbl.set_halign(gtk4::Align::Start);
        frame.append(&lbl);

        let input_rule = gtk4::Entry::new();
        input_rule.set_placeholder_text(Some("e.g. ||example.com^ or 127.0.0.1 badsite.com"));
        input_rule.set_text(initial_text);
        input_rule.set_size_request(-1, 40);
        input_rule.add_css_class("dnsfe-dialog-entry");
        frame.append(&input_rule);

        frame.append(&spacer(8));

        let btn_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);

        let btn_save = gtk4::Button::new();
        btn_save.set_label("Save");
        btn_save.set_size_request(-1, 38);
        btn_save.add_css_class("green-btn");
        let d1 = root.clone();
        btn_save.connect_clicked(move |_| d1.close());
        btn_row.append(&btn_save);

        let btn_cancel = gtk4::Button::new();
        btn_cancel.set_label("Cancel");
        btn_cancel.set_size_request(-1, 38);
        btn_cancel.add_css_class("outline-btn");
        let d2 = root.clone();
        btn_cancel.connect_clicked(move |_| d2.close());
        btn_row.append(&btn_cancel);

        frame.append(&btn_row);
        root.set_child(Some(&frame));

        AddRuleSimpleDialog {
            root,
            input_rule,
            accepted_cb: RefCell::new(None),
            rejected_cb: RefCell::new(None),
        }
    }

    pub fn get_rule_text(&self) -> String {
        self.input_rule.text().trim().to_string()
    }

    // QDialog.exec() Accepted/Rejected -> callbacks fired on close.
    pub fn connect_accepted(&self, cb: Box<dyn Fn()>) {
        *self.accepted_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_rejected(&self, cb: Box<dyn Fn()>) {
        *self.rejected_cb.borrow_mut() = Some(cb);
    }

    pub fn present(&self) {
        self.root.present();
    }
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

// ---------------------------------------------------------------------------
// DnsFilterEditorWindow
// ---------------------------------------------------------------------------
const SAMPLE_ADG_RULES: [&str; 7] = [
    "||analytics.google.com^$dns",
    "||telemetry.microsoft.com^$dns",
    "||ads.facebook.com^$dns",
    "||adservice.google.com^$dns",
    "||app-measurement.com^$dns",
    "||crashlytics.com^$dns",
    "||doubleclick.net^$dns",
];

struct Inner {
    root: gtk4::Window,
    user_rules: RefCell<Vec<String>>,
    selected_filter_id: RefCell<String>,
    search_filter: gtk4::Entry,
    search_rule: gtk4::Entry,
    row_user_rules: gtk4::Box,
    row_adguard_filter: gtk4::Box,
    chk_user: gtk4::Label,
    chk_adg: gtk4::Label,
    lbl_user_name: gtk4::Label,
    lbl_user_count: gtk4::Label,
    lbl_user_tag: gtk4::Label,
    lbl_adg_name: gtk4::Label,
    lbl_adg_ver: gtk4::Label,
    lbl_adg_count: gtk4::Label,
    lbl_adg_tag: gtk4::Label,
    empty_state: gtk4::Box,
    rules_list: gtk4::ListBox,
    current_rule: RefCell<Option<String>>,
    closed_cb: RefCell<Option<Box<dyn Fn()>>>,
}

#[derive(Clone)]
pub struct DnsFilterEditorWindow {
    inner: Rc<Inner>,
}

impl DnsFilterEditorWindow {
    pub fn new() -> DnsFilterEditorWindow {
        let root = crate::util::sub_window("DNS Filter Editor");
        root.set_default_size(920, 600);
        root.set_size_request(800, 500);
        root.add_css_class("dnsfe-window");

        let main_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        root.set_child(Some(&main_layout));

        // Deviation: Qt QSplitter -> gtk4::Paned (fixed initial sizes 280/640).
        let splitter = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        main_layout.append(&splitter);

        // ----------------- LEFT PANE: filters list -----------------
        let left_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        left_widget.add_css_class("dnsfe-left");
        left_widget.set_margin_top(16);
        left_widget.set_margin_bottom(12);
        left_widget.set_margin_start(16);
        left_widget.set_margin_end(16);

        let search_filter = gtk4::Entry::new();
        search_filter.set_placeholder_text(Some("🔍  Search filter"));
        search_filter.set_size_request(-1, 34);
        search_filter.add_css_class("dnsfe-search");
        left_widget.append(&search_filter);

        let lbl_installed = gtk4::Label::new(Some("Installed filters"));
        lbl_installed.add_css_class("dnsfe-group-label");
        lbl_installed.set_halign(gtk4::Align::Start);
        left_widget.append(&lbl_installed);

        let filter_scroll = gtk4::ScrolledWindow::new();
        filter_scroll.set_vexpand(true);
        filter_scroll.add_css_class("home-scroll");

        let f_cont = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        f_cont.add_css_class("transparent-container");
        f_cont.set_margin_end(4);

        let lbl_grp_custom = gtk4::Label::new(Some("Custom filters"));
        lbl_grp_custom.add_css_class("dnsfe-group-label");
        lbl_grp_custom.set_halign(gtk4::Align::Start);
        f_cont.append(&lbl_grp_custom);

        // User rules row (initially active).
        let row_user_rules = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        row_user_rules.add_css_class("dnsfe-row-active");
        row_user_rules.set_margin_top(8);
        row_user_rules.set_margin_bottom(8);
        row_user_rules.set_margin_start(10);
        row_user_rules.set_margin_end(10);

        let chk_user = gtk4::Label::new(Some("✓"));
        chk_user.add_css_class("dnsfe-check");
        row_user_rules.append(&chk_user);

        let t_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        t_box.set_hexpand(true);
        let lbl_user_name = gtk4::Label::new(Some("User rules"));
        lbl_user_name.add_css_class("dnsfe-name");
        lbl_user_name.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_user_name);
        let lbl_user_count = gtk4::Label::new(Some("0 rules"));
        lbl_user_count.add_css_class("dnsfe-count");
        lbl_user_count.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_user_count);
        let lbl_user_tag = gtk4::Label::new(Some("#trusted"));
        lbl_user_tag.add_css_class("dnsfe-tag");
        lbl_user_tag.set_halign(gtk4::Align::Start);
        t_box.append(&lbl_user_tag);
        row_user_rules.append(&t_box);
        f_cont.append(&row_user_rules);

        f_cont.append(&spacer(6));

        let lbl_grp_gen = gtk4::Label::new(Some("General"));
        lbl_grp_gen.add_css_class("dnsfe-group-label");
        lbl_grp_gen.set_halign(gtk4::Align::Start);
        f_cont.append(&lbl_grp_gen);

        // AdGuard DNS filter row.
        let row_adguard_filter = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        row_adguard_filter.add_css_class("dnsfe-row");
        row_adguard_filter.set_margin_top(8);
        row_adguard_filter.set_margin_bottom(8);
        row_adguard_filter.set_margin_start(10);
        row_adguard_filter.set_margin_end(10);

        let chk_adg = gtk4::Label::new(Some("✓"));
        chk_adg.add_css_class("dnsfe-check");
        row_adguard_filter.append(&chk_adg);

        let t2_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        t2_box.set_hexpand(true);
        let lbl_adg_name = gtk4::Label::new(Some("AdGuard DNS filter"));
        lbl_adg_name.add_css_class("dnsfe-name");
        lbl_adg_name.set_halign(gtk4::Align::Start);
        t2_box.append(&lbl_adg_name);
        let lbl_adg_ver = gtk4::Label::new(Some("1.0.78.50 updated: today"));
        lbl_adg_ver.add_css_class("dnsfe-ver");
        lbl_adg_ver.set_halign(gtk4::Align::Start);
        t2_box.append(&lbl_adg_ver);
        let lbl_adg_count = gtk4::Label::new(Some("176765 rules"));
        lbl_adg_count.add_css_class("dnsfe-count");
        lbl_adg_count.set_halign(gtk4::Align::Start);
        t2_box.append(&lbl_adg_count);
        let lbl_adg_tag = gtk4::Label::new(Some("#trusted"));
        lbl_adg_tag.add_css_class("dnsfe-tag");
        lbl_adg_tag.set_halign(gtk4::Align::Start);
        t2_box.append(&lbl_adg_tag);
        row_adguard_filter.append(&t2_box);
        f_cont.append(&row_adguard_filter);

        filter_scroll.set_child(Some(&f_cont));
        left_widget.append(&filter_scroll);

        // Left bottom toolbar: +, ⚙.
        let left_toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let btn_add_filt = gtk4::Button::new();
        btn_add_filt.set_label("+");
        btn_add_filt.set_size_request(28, 28);
        btn_add_filt.add_css_class("dnsfe-tool-btn");
        left_toolbar.append(&btn_add_filt);

        // btn_cfg_filt has no handler in Python (kept as-is).
        let btn_cfg_filt = gtk4::Button::new();
        btn_cfg_filt.set_label("⚙");
        btn_cfg_filt.set_size_request(28, 28);
        btn_cfg_filt.add_css_class("dnsfe-tool-btn");
        left_toolbar.append(&btn_cfg_filt);

        left_widget.append(&left_toolbar);

        // ----------------- RIGHT PANE: filtering rules -----------------
        let right_widget = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        right_widget.add_css_class("dnsfe-right");
        right_widget.set_margin_top(16);
        right_widget.set_margin_bottom(12);
        right_widget.set_margin_start(18);
        right_widget.set_margin_end(18);

        let search_rule = gtk4::Entry::new();
        search_rule.set_placeholder_text(Some("🔍  Search rule"));
        search_rule.set_size_request(-1, 34);
        search_rule.add_css_class("dnsfe-search-rule");
        right_widget.append(&search_rule);

        // Header: Filtering rules + syntax link.
        let rh_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        rh_box.set_margin_top(4);
        rh_box.set_margin_bottom(4);

        let lbl_rh_title = gtk4::Label::new(Some("Filtering rules"));
        lbl_rh_title.add_css_class("lbl-header-10");
        rh_box.append(&lbl_rh_title);

        // Python: rh_box.addStretch() — pushes the syntax link to the right edge.
        // Without it the two labels sit next to each other and read as one
        // duplicated title ("Filtering rulesFiltering rules syntax").
        let rh_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        rh_spacer.set_hexpand(true);
        rh_box.append(&rh_spacer);

        let btn_syntax = gtk4::Button::new();
        btn_syntax.set_label("Filtering rules syntax");
        btn_syntax.add_css_class("green-link");
        btn_syntax.connect_clicked(move |_| {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard.com/kb/general/dns-filtering-syntax/")
                .spawn();
        });
        rh_box.append(&btn_syntax);

        right_widget.append(&rh_box);

        // Middle area: empty state OR rules list.
        let rules_stack = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        rules_stack.set_vexpand(true);

        let empty_state = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        empty_state.add_css_class("transparent-container");
        empty_state.set_valign(gtk4::Align::Center);

        empty_state.append(&mascot_widget());

        let lbl_no_rules = gtk4::Label::new(Some("There are no user rules yet"));
        lbl_no_rules.add_css_class("lbl-empty-12");
        empty_state.append(&lbl_no_rules);

        let btn_action_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        btn_action_row.set_halign(gtk4::Align::Center);

        let btn_add_rule = gtk4::Button::new();
        btn_add_rule.set_label("Add a rule");
        btn_add_rule.set_size_request(130, 36);
        btn_add_rule.add_css_class("fe-green-btn");
        btn_action_row.append(&btn_add_rule);

        let btn_import = gtk4::Button::new();
        btn_import.set_label("Import rules");
        btn_import.set_size_request(130, 36);
        btn_import.add_css_class("fe-import-btn");
        btn_action_row.append(&btn_import);

        empty_state.append(&btn_action_row);
        rules_stack.append(&empty_state);

        let rules_list = gtk4::ListBox::new();
        rules_list.add_css_class("dnsfe-list");
        rules_list.set_vexpand(true);
        rules_list.set_visible(false);
        rules_stack.append(&rules_list);

        right_widget.append(&rules_stack);

        // Right bottom toolbar: +, −, ⚙, ✏.
        let right_toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let btn_add_r = gtk4::Button::new();
        btn_add_r.set_label("+");
        btn_add_r.set_size_request(28, 28);
        btn_add_r.add_css_class("dnsfe-tool-btn");
        right_toolbar.append(&btn_add_r);

        let btn_del_r = gtk4::Button::new();
        btn_del_r.set_label("−");
        btn_del_r.set_size_request(28, 28);
        btn_del_r.add_css_class("dnsfe-tool-btn-del");
        right_toolbar.append(&btn_del_r);

        // btn_cfg_r has no handler in Python (kept as-is).
        let btn_cfg_r = gtk4::Button::new();
        btn_cfg_r.set_label("⚙");
        btn_cfg_r.set_size_request(28, 28);
        btn_cfg_r.add_css_class("dnsfe-tool-btn");
        right_toolbar.append(&btn_cfg_r);

        let btn_edit_r = gtk4::Button::new();
        btn_edit_r.set_label("✏");
        btn_edit_r.set_size_request(28, 28);
        btn_edit_r.add_css_class("dnsfe-tool-btn");
        right_toolbar.append(&btn_edit_r);

        right_widget.append(&right_toolbar);

        splitter.set_start_child(Some(&left_widget));
        splitter.set_end_child(Some(&right_widget));
        splitter.set_resize_end_child(true);
        splitter.set_position(280);

        let inner = Rc::new(Inner {
            root,
            user_rules: RefCell::new(Vec::new()),
            selected_filter_id: RefCell::new("user_rules".to_string()),
            search_filter,
            search_rule,
            row_user_rules,
            row_adguard_filter,
            chk_user,
            chk_adg,
            lbl_user_name,
            lbl_user_count,
            lbl_user_tag,
            lbl_adg_name,
            lbl_adg_ver,
            lbl_adg_count,
            lbl_adg_tag,
            empty_state,
            rules_list,
            current_rule: RefCell::new(None),
            closed_cb: RefCell::new(None),
        });

        // Row clicks (_select_filter).
        let i1 = inner.clone();
        let r1 = inner.row_user_rules.clone();
        r1.add_controller(row_click(&i1, "user_rules"));

        let i2 = inner.clone();
        let r2 = inner.row_adguard_filter.clone();
        r2.add_controller(row_click(&i2, "adguard_filter"));

        // Search rule textChanged -> filter list.
        let i3 = inner.clone();
        inner.search_rule.connect_changed(move |e| filter_rules_list(&i3, e.text().as_str()));

        // List selection tracks current rule text (QListWidget currentItem).
        let i4 = inner.clone();
        inner.rules_list.connect_row_selected(move |_l, row| {
            if let Some(row) = row {
                if let Some(lbl) = row.child() {
                    if let Some(lbl) = lbl.downcast_ref::<gtk4::Label>() {
                        *i4.current_rule.borrow_mut() = Some(lbl.text().to_string());
                    }
                }
            }
        });

        // Toolbar buttons.
        let i5 = inner.clone();
        btn_add_filt.connect_clicked(move |_| add_rule_action(&i5));
        let i6 = inner.clone();
        btn_add_r.connect_clicked(move |_| add_rule_action(&i6));
        let i7 = inner.clone();
        btn_add_rule.connect_clicked(move |_| add_rule_action(&i7));
        let i8 = inner.clone();
        btn_import.connect_clicked(move |_| import_rules_action(&i8));
        let i9 = inner.clone();
        btn_del_r.connect_clicked(move |_| delete_selected_rule(&i9));
        let i10 = inner.clone();
        btn_edit_r.connect_clicked(move |_| edit_selected_rule(&i10));

        // closeEvent -> closed signal.
        let i11 = inner.clone();
        inner.root.connect_close_request(move |_| {
            if let Some(cb) = i11.closed_cb.borrow().as_ref() {
                cb();
            }
            gtk4::glib::Propagation::Proceed
        });

        load_rules(&inner);

        DnsFilterEditorWindow { inner }
    }

    pub fn root(&self) -> &gtk4::Window {
        &self.inner.root
    }

    pub fn connect_closed(&self, cb: Box<dyn Fn()>) {
        *self.inner.closed_cb.borrow_mut() = Some(cb);
    }


    pub fn select_filter(&self, filter_id: &str) {
        select_filter(&self.inner, filter_id);
    }
}

fn row_click(inner: &Rc<Inner>, filter_id: &str) -> gtk4::GestureClick {
    let i = inner.clone();
    let fid = filter_id.to_string();
    let gesture = gtk4::GestureClick::new();
    gesture.connect_pressed(move |_g, n, _x, _y| {
        // ignore clicks while the row is disabled
        if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
        if n == 1 {
            select_filter(&i, &fid);
        }
    });
    gesture
}

fn select_filter(inner: &Rc<Inner>, filter_id: &str) {
    *inner.selected_filter_id.borrow_mut() = filter_id.to_string();
    if filter_id == "user_rules" {
        inner.row_user_rules.remove_css_class("dnsfe-row");
        inner.row_user_rules.add_css_class("dnsfe-row-active");
        inner.row_adguard_filter.remove_css_class("dnsfe-row-active");
        inner.row_adguard_filter.add_css_class("dnsfe-row");
        let rules = inner.user_rules.borrow().clone();
        populate_rules_list(inner, &rules);
    } else {
        inner.row_adguard_filter.remove_css_class("dnsfe-row");
        inner.row_adguard_filter.add_css_class("dnsfe-row-active");
        inner.row_user_rules.remove_css_class("dnsfe-row-active");
        inner.row_user_rules.add_css_class("dnsfe-row");
        populate_rules_list(inner, &SAMPLE_ADG_RULES.iter().map(|s| s.to_string()).collect::<Vec<String>>());
    }
}

pub fn load_rules(inner: &Rc<Inner>) {
    inner.user_rules.borrow_mut().clear();
    let path = dns_user_rules_file();
    if std::path::Path::exists(&path) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let clean = line.trim();
                if !clean.is_empty() && !clean.starts_with('!') {
                    inner.user_rules.borrow_mut().push(clean.to_string());
                }
            }
        }
    }

    inner
        .lbl_user_count
        .set_text(&format!("{} rules", inner.user_rules.borrow().len()));

    // Load live AdGuard DNS filter info from SQLite.
    let db_path = dns_db_path();
    if std::path::Path::exists(&db_path) {
        if let Ok(conn) = Connection::open(&db_path) {
            let row: Option<(Option<String>, Option<i64>, i64)> = conn
                .query_row(
                    "SELECT version, last_update_time, is_enabled FROM filter WHERE filter_id = 1;",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();
            if let Some((ver, up_time, enabled)) = row {
                let ver = ver.unwrap_or_else(|| "1.0.78.50".to_string());
                let time_str = match up_time {
                    Some(ts) => format_update_time(ts),
                    None => "today".to_string(),
                };
                inner.lbl_adg_ver.set_text(&format!("{} updated:{}", ver, time_str));
                if enabled == 1 {
                    inner.chk_adg.set_text("✓");
                    inner.chk_adg.remove_css_class("dnsfe-check-off");
                    inner.chk_adg.add_css_class("dnsfe-check");
                } else {
                    inner.chk_adg.set_text("");
                    inner.chk_adg.remove_css_class("dnsfe-check");
                    inner.chk_adg.add_css_class("dnsfe-check-off");
                }
            }
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM rules_list WHERE filter_id = 1;",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if count > 1 {
                inner.lbl_adg_count.set_text(&format!("{} rules", count));
            } else {
                inner.lbl_adg_count.set_text("176765 rules");
            }
        }
    }

    if *inner.selected_filter_id.borrow() == "user_rules" {
        let rules = inner.user_rules.borrow().clone();
        populate_rules_list(inner, &rules);
    }
}

// Python strftime "%-m/%-d/%Y %-I:%M:%S %p" via localtime.
fn format_update_time(ts: i64) -> String {
    unsafe {
        let t = ts as libc::time_t;
        let tm = libc::localtime(&t);
        if tm.is_null() {
            return "today".to_string();
        }
        let tm = &*tm;
        let hour12 = tm.tm_hour % 12;
        let hour12 = if hour12 == 0 { 12 } else { hour12 };
        let ampm = if tm.tm_hour >= 12 { "PM" } else { "AM" };
        format!(
            "{}/{}/{} {}:{:02}:{:02} {}",
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_year + 1900,
            hour12,
            tm.tm_min,
            tm.tm_sec,
            ampm
        )
    }
}

fn populate_rules_list(inner: &Rc<Inner>, rules: &[String]) {
    // Clear list rows.
    while let Some(row) = inner.rules_list.first_child() {
        inner.rules_list.remove(&row);
    }
    *inner.current_rule.borrow_mut() = None;

    if !rules.is_empty() {
        inner.empty_state.set_visible(false);
        inner.rules_list.set_visible(true);
        for r in rules {
            let row = gtk4::ListBoxRow::new();
            let lbl = gtk4::Label::new(Some(r));
            lbl.add_css_class("dnsfe-rule-row");
            lbl.set_xalign(0.0);
            row.set_child(Some(&lbl));
            inner.rules_list.append(&row);
        }
    } else {
        inner.rules_list.set_visible(false);
        inner.empty_state.set_visible(true);
    }
}

fn filter_rules_list(inner: &Rc<Inner>, text: &str) {
    let q = text.trim().to_lowercase();
    let target_list: Vec<String> = if *inner.selected_filter_id.borrow() == "user_rules" {
        inner.user_rules.borrow().clone()
    } else {
        SAMPLE_ADG_RULES.iter().map(|s| s.to_string()).collect()
    };
    if q.is_empty() {
        populate_rules_list(inner, &target_list);
    } else {
        let filtered: Vec<String> = target_list
            .iter()
            .filter(|r| r.to_lowercase().contains(&q))
            .cloned()
            .collect();
        populate_rules_list(inner, &filtered);
    }
}

fn add_rule_action(inner: &Rc<Inner>) {
    let dlg = AddRuleSimpleDialog::new("");
    let i = inner.clone();
    let entry = dlg.input_rule.clone();
    dlg.connect_accepted(Box::new(move || {
        let new_r = entry.text().trim().to_string();
        if !new_r.is_empty() && !i.user_rules.borrow().contains(&new_r) {
            i.user_rules.borrow_mut().push(new_r);
            save_rules_to_file(&i);
        }
    }));
    dlg.present();
}

fn edit_selected_rule(inner: &Rc<Inner>) {
    if *inner.selected_filter_id.borrow() != "user_rules" {
        show_message(None, "Read Only", "Only custom user rules can be edited.");
        return;
    }
    let old_r = match inner.current_rule.borrow().as_ref() {
        Some(r) => r.clone(),
        None => return,
    };
    let dlg = AddRuleSimpleDialog::new(&old_r);
    let i = inner.clone();
    let old = old_r.clone();
    let entry = dlg.input_rule.clone();
    dlg.connect_accepted(Box::new(move || {
        let new_r = entry.text().trim().to_string();
        if !new_r.is_empty() {
            let idx = i.user_rules.borrow().iter().position(|r| r == &old);
            if let Some(idx) = idx {
                i.user_rules.borrow_mut()[idx] = new_r;
                save_rules_to_file(&i);
            }
        }
    }));
    dlg.present();
}

fn delete_selected_rule(inner: &Rc<Inner>) {
    if *inner.selected_filter_id.borrow() != "user_rules" {
        show_message(None, "Read Only", "Only custom user rules can be deleted.");
        return;
    }
    let r_text = match inner.current_rule.borrow().as_ref() {
        Some(r) => r.clone(),
        None => return,
    };
    if let Some(pos) = inner.user_rules.borrow().iter().position(|r| r == &r_text) {
        inner.user_rules.borrow_mut().remove(pos);
        save_rules_to_file(inner);
    }
}

fn import_rules_action(inner: &Rc<Inner>) {
    let dlg = gtk4::FileChooserDialog::new(
        Some("Import DNS Rules"),
        None::<&gtk4::Window>,
        gtk4::FileChooserAction::Open,
        &[
            (&i18n::tr("btn_cancel"), gtk4::ResponseType::Cancel),
            ("Open", gtk4::ResponseType::Accept),
        ],
    );
    let i = inner.clone();
    dlg.connect_response(move |d, response| {
        if response != gtk4::ResponseType::Accept {
            return;
        }
        let path = match d.file().and_then(|f| f.path()) {
            Some(p) => p,
            None => return,
        };
        if std::path::Path::exists(&path) {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    for line in content.lines() {
                        let clean = line.trim();
                        if !clean.is_empty()
                            && !clean.starts_with('!')
                            && !i.user_rules.borrow().contains(&clean.to_string())
                        {
                            i.user_rules.borrow_mut().push(clean.to_string());
                        }
                    }
                    save_rules_to_file(&i);
                }
                Err(e) => show_message(
                    None,
                    "Import Error",
                    &format!("Failed to import rules:\n{}", e),
                ),
            }
        }
    });
    dlg.present();
}

fn save_rules_to_file(inner: &Rc<Inner>) {
    let path = dns_user_rules_file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::File::create(&path) {
        use std::io::Write;
        let _ = f.write_all(b"! DNS User Rules\n");
        for r in inner.user_rules.borrow().iter() {
            let _ = f.write_all(format!("{}\n", r).as_bytes());
        }
    }

    load_rules(inner);
}
