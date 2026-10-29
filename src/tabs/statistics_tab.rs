// Port of adguard_modern/tabs/statistics_tab.py — the Statistics tab.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use crate::tabs::companies_view::CompaniesView;
use crate::tabs::stat_components::{StatListRow, StatThreeCols};
use crate::tabs::stats_worker;
use crate::widgets::filtering_log_window::FilteringLogDialog;
use crate::widgets::stat_modals::{ClearStatsModal, TurnOffStatsModal, WhySafeModal};
use crate::widgets::svg;

struct Inner {
    root: gtk4::Box,
    stack: gtk4::Stack,
    lbl_header: gtk4::Label,
    btn_calendar: gtk4::Button,
    btn_sort: gtk4::Button,
    btn_more: gtk4::Button,
    lbl_sec_requests: gtk4::Label,
    requests_cols: StatThreeCols,
    lbl_sec_dns: gtk4::Label,
    dns_row: StatListRow,
    lbl_sec_companies: gtk4::Label,
    comp_layout: gtk4::Box,
    btn_view_all: gtk4::Button,
    lbl_sec_data: gtk4::Label,
    data_usage_cols: StatThreeCols,
    companies_view: CompaniesView,
    current_time_filter: RefCell<String>,
    current_sort: RefCell<String>,
    stats_enabled: RefCell<bool>,
    worker_running: RefCell<bool>,
}

#[derive(Clone)]
pub struct StatisticsTab {
    inner: Rc<Inner>,
}

impl StatisticsTab {
    pub fn new() -> StatisticsTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");

        let stack = gtk4::Stack::new();
        stack.set_vexpand(true);
        root.append(&stack);
        let stack_c = stack.clone();

        // --- page 0: main statistics view ---
        let main_view = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        main_view.add_css_class("page-bg");
        main_view.set_margin_start(36);
        main_view.set_margin_end(36);
        main_view.set_margin_top(26);
        main_view.set_margin_bottom(16);

        let header_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let lbl_header = gtk4::Label::new(Some("Statistics"));
        lbl_header.add_css_class("page-title");
        header_layout.append(&lbl_header);
        let h_spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        h_spacer.set_hexpand(true);
        header_layout.append(&h_spacer);
        let btn_calendar = header_button("stat_calendar.svg");
        let btn_sort = header_button("stat_filter.svg");
        let btn_more = header_button("stat_more.svg");
        header_layout.append(&btn_calendar);
        header_layout.append(&btn_sort);
        header_layout.append(&btn_more);
        main_view.append(&header_layout);

        let c_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
        c_layout.set_margin_top(4);
        c_layout.set_margin_end(12);
        c_layout.set_margin_bottom(16);

        let lbl_sec_requests = section_title("Requests");
        c_layout.append(&lbl_sec_requests);
        let requests_cols = StatThreeCols::new([
            ("...", "ads", "#FF6B4A"),
            ("...", "trackers", "#E67E22"),
            ("...", "total", "#8E8E93"),
        ]);
        c_layout.append(requests_cols.root());

        let lbl_sec_dns = section_title("DNS");
        c_layout.append(&lbl_sec_dns);
        let dns_row = StatListRow::new("feat_dns.svg", "DNS requests", "0", "0", "0", "");
        c_layout.append(dns_row.root());

        let lbl_sec_companies = section_title("Companies");
        c_layout.append(&lbl_sec_companies);
        let comp_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        c_layout.append(&comp_layout);

        let btn_view_all = link_button("View all companies");
        c_layout.append(&btn_view_all);
        let btn_view_all_c = btn_view_all.clone();

        let lbl_sec_data = section_title("Data usage");
        c_layout.append(&lbl_sec_data);
        let data_usage_cols = StatThreeCols::new([
            ("...", "saved", c::COLOR_ADGUARD_GREEN),
            ("...", "uploaded", "#B197FC"),
            ("...", "downloaded", "#8E8E93"),
        ]);
        c_layout.append(data_usage_cols.root());

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&c_layout));
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");
        main_view.append(&scroll);

        // --- page 1: all companies view ---
        let companies_view = CompaniesView::new();

        let cv = companies_view.clone();
        let bc = btn_calendar.clone();
        let bs = btn_sort.clone();
        let bm = btn_more.clone();
        let inner = Rc::new(Inner {
            root,
            stack,
            lbl_header,
            btn_calendar,
            btn_sort,
            btn_more,
            lbl_sec_requests,
            requests_cols,
            lbl_sec_dns,
            dns_row,
            lbl_sec_companies,
            comp_layout,
            btn_view_all,
            lbl_sec_data,
            data_usage_cols,
            companies_view,
            current_time_filter: RefCell::new("all".to_string()),
            current_sort: RefCell::new("ads".to_string()),
            stats_enabled: RefCell::new(true),
            worker_running: RefCell::new(false),
        });

        stack_c.add_named(&main_view, Some("main"));
        stack_c.add_named(cv.root(), Some("companies"));

        let i2 = inner.clone();
        btn_view_all_c.connect_clicked(move |_| i2.stack.set_visible_child_name("companies"));
        let i3 = inner.clone();
        cv.connect_back_clicked(Box::new(move || i3.stack.set_visible_child_name("main")));
        let i4 = inner.clone();
        cv.connect_time_filter_clicked(Box::new(move || show_time_menu(&i4)));
        let i5 = inner.clone();
        cv.connect_sort_filter_clicked(Box::new(move || show_sort_menu(&i5)));
        cv.connect_company_clicked(Box::new(move |name: &str| open_company_log(name)));

        bc.connect_clicked({
            let i7 = inner.clone();
            move |_| show_menu(&i7.btn_calendar, menu_items(&i7, &[
                ("menu_24_hours", "24h"),
                ("menu_7_days", "7d"),
                ("menu_14_days", "14d"),
                ("menu_all_time", "all"),
            ], "time"))
        });
        bs.connect_clicked({
            let i8 = inner.clone();
            move |_| show_menu(&i8.btn_sort, menu_items(&i8, &[
                ("menu_sort_ads", "ads"),
                ("menu_sort_trackers", "trackers"),
                ("menu_sort_requests", "requests"),
            ], "sort"))
        });
        bm.connect_clicked({
            let i9 = inner.clone();
            move |_| show_more_menu(&i9)
        });

        let i11 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i11)));
        retranslate_ui(&inner);

        // Python showEvent → refresh_stats on first show
        let i10 = inner.clone();
        inner.root.connect_map(move |_| refresh_stats(&i10));

        refresh_stats(&inner);

        StatisticsTab { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn refresh_stats(&self) {
        refresh_stats(&self.inner);
    }
}

fn refresh_stats(inner: &Rc<Inner>) {
    if !*inner.stats_enabled.borrow() {
        return;
    }
    if *inner.worker_running.borrow() {
        return;
    }
    *inner.worker_running.borrow_mut() = true;
    let tf = inner.current_time_filter.borrow().clone();
    let sort = inner.current_sort.borrow().clone();
    let target = inner.clone();
    stats_worker::start(
        &tf,
        &sort,
        Box::new(move |res| {
            *target.worker_running.borrow_mut() = false;
            on_stats_ready(&target, &res);
        }),
    );
}

fn on_stats_ready(inner: &Rc<Inner>, data: &stats_worker::StatsResult) {
    inner.requests_cols.update_values(&data.blocked_ads, &data.blocked_trackers, &data.total_req);
    inner.dns_row.update_counts(&data.dns_ads, &data.dns_trackers, &data.dns_total);
    inner.data_usage_cols.update_values(&data.saved_str, &data.uploaded_str, &data.downloaded_str);

    while let Some(child) = inner.comp_layout.first_child() {
        inner.comp_layout.remove(&child);
    }
    for comp in &data.companies {
        let row = StatListRow::new(&comp.icon, &comp.name, &comp.ads, &comp.trackers, &comp.total, &comp.initial);
                row.connect_clicked(Box::new(move |n: &str| open_company_log(n)));
        inner.comp_layout.append(row.root());
    }

    inner.companies_view.set_companies(&data.all_companies);
}

fn open_company_log(name: &str) {
    let dlg = FilteringLogDialog::new();
    dlg.set_search(name);
    dlg.root().present();
}

fn show_time_menu(inner: &Rc<Inner>) {
    show_menu(&inner.btn_calendar, menu_items(inner, &[
        ("menu_24_hours", "24h"),
        ("menu_7_days", "7d"),
        ("menu_14_days", "14d"),
        ("menu_all_time", "all"),
    ], "time"));
}

fn show_sort_menu(inner: &Rc<Inner>) {
    show_menu(&inner.btn_sort, menu_items(inner, &[
        ("menu_sort_ads", "ads"),
        ("menu_sort_trackers", "trackers"),
        ("menu_sort_requests", "requests"),
    ], "sort"));
}

fn menu_items(inner: &Rc<Inner>, options: &[(&str, &str)], which: &str) -> Vec<(String, Box<dyn Fn()>)> {
    let current = if which == "time" {
        inner.current_time_filter.borrow().clone()
    } else {
        inner.current_sort.borrow().clone()
    };
    let target = inner.clone();
    let w = which.to_string();
    options
        .iter()
        .map(|(key, value)| {
            let label = if current == *value {
                format!("{}   ✓", i18n::tr(key))
            } else {
                i18n::tr(key)
            };
            let t = target.clone();
            let v = value.to_string();
            let ww = w.clone();
            (label, Box::new(move || {
                if ww == "time" {
                    *t.current_time_filter.borrow_mut() = v.clone();
                } else {
                    *t.current_sort.borrow_mut() = v.clone();
                }
                refresh_stats(&t);
            }) as Box<dyn Fn()>)
        })
        .collect()
}

fn show_more_menu(inner: &Rc<Inner>) {
    let target = inner.clone();
    let mut items: Vec<(String, Box<dyn Fn()>)> = Vec::new();
    items.push((i18n::tr("menu_refresh"), {
        let t = target.clone();
        Box::new(move || refresh_stats(&t)) as Box<dyn Fn()>
    }));
    items.push((i18n::tr("menu_why_safe"), {
        let t = target.clone();
        Box::new(move || {
            let modal = WhySafeModal::new();
            modal.root().present();
            let _ = &t;
        }) as Box<dyn Fn()>
    }));
    items.push((i18n::tr("flog_win_title"), {
        let t = target.clone();
        Box::new(move || {
            let dlg = FilteringLogDialog::new();
            dlg.root().present();
            let _ = &t;
        }) as Box<dyn Fn()>
    }));
    if *inner.stats_enabled.borrow() {
        items.push((i18n::tr("menu_turn_off_stats"), {
            let t = target.clone();
            Box::new(move || confirm_turn_off_stats(&t)) as Box<dyn Fn()>
        }));
    } else {
        items.push((i18n::tr("menu_turn_on_stats"), {
            let t = target.clone();
            Box::new(move || {
                *t.stats_enabled.borrow_mut() = true;
                refresh_stats(&t);
            }) as Box<dyn Fn()>
        }));
    }
    items.push((i18n::tr("menu_clear_stats"), {
        let t = target.clone();
        Box::new(move || clear_statistics(&t)) as Box<dyn Fn()>
    }));
    show_menu(&inner.btn_more, items);
}

fn confirm_turn_off_stats(inner: &Rc<Inner>) {
    let modal = TurnOffStatsModal::new();
    let target = inner.clone();
    modal.connect_accepted(Box::new(move || {
        *target.stats_enabled.borrow_mut() = false;
        target.requests_cols.update_values("0", "0", "0");
        target.dns_row.update_counts("0", "0", "0");
        target.data_usage_cols.update_values("0 KB", "0 KB", "0 MB");
        while let Some(child) = target.comp_layout.first_child() {
            target.comp_layout.remove(&child);
        }
        target.companies_view.set_companies(&[]);
    }));
    modal.root().present();
}

fn clear_statistics(inner: &Rc<Inner>) {
    let target = inner.clone();
    let modal = ClearStatsModal::new();
    modal.connect_accepted(Box::new(move || {
        for lp in stats_worker::get_log_paths() {
            match std::fs::OpenOptions::new().write(true).truncate(true).open(&lp) {
                Ok(_) => {}
                Err(_) => {
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "sudo -n truncate -s 0 '{}' 2>/dev/null || truncate -s 0 '{}' 2>/dev/null",
                            lp.display(),
                            lp.display()
                        ))
                        .status();
                }
            }
        }
        refresh_stats(&target);
    }));
    modal.root().present();
}

fn show_menu(anchor: &gtk4::Button, items: Vec<(String, Box<dyn Fn()>)>) {
    let popover = gtk4::Popover::new();
    popover.set_parent(anchor);
    popover.add_css_class("menu-popover");
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    for (label, cb) in items {
        let item = gtk4::Button::new();
        item.set_label(&label);
        item.add_css_class("menu-item");
        let pop = popover.clone();
        item.connect_clicked(move |_| {
            cb();
            pop.hide();
        });
        menu_box.append(&item);
    }
    popover.set_child(Some(&menu_box));
    popover.present();
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.lbl_header.set_text(&i18n::tr("nav_statistics"));
    inner.lbl_sec_requests.set_text(&i18n::tr("stat_requests"));
    inner.lbl_sec_dns.set_text(&i18n::tr("stat_dns"));
    inner.lbl_sec_companies.set_text(&i18n::tr("stat_companies"));
    inner.lbl_sec_data.set_text(&i18n::tr("stat_data_usage"));
    inner.btn_view_all.set_label(&i18n::tr("stat_view_all_companies"));
}

fn section_title(title: &str) -> gtk4::Label {
    let lbl = gtk4::Label::new(Some(title));
    lbl.add_css_class("section-title");
    lbl
}

fn link_button(text: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.set_label(text);
    btn.add_css_class("link-btn");
    btn
}

fn header_button(icon_fn: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("flat-icon-btn");
    btn.set_size_request(32, 32);
    btn.set_child(Some(&svg::svg_image(icon_fn, 20)));
    btn
}
