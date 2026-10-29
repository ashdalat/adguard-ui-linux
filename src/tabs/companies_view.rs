// Port of adguard_modern/tabs/companies_view.py — the all-companies list view.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::tabs::stats_worker::CompanyStat;
use crate::widgets::svg;

struct CompanyRowInner {
    root: gtk4::Box,
    name: String,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct CompanyRow {
    inner: Rc<CompanyRowInner>,
}

impl CompanyRow {
    pub fn new(
        name: &str,
        ads: &str,
        trackers: &str,
        total: &str,
        icon: &str,
        initial: &str,
    ) -> CompanyRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("company-row");
        root.set_size_request(-1, 46);
        root.set_margin_start(12);
        root.set_margin_end(16);

        let icons = c::icons_dir();
        let mut icon_path: Option<PathBuf> = None;
        if !icon.is_empty() {
            let cand = icons.join("companies").join(icon);
            if cand.exists() {
                icon_path = Some(cand);
            }
        }
        if icon_path.is_none() {
            let generic = icons.join("companies").join("company_generic.svg");
            if generic.exists() {
                icon_path = Some(generic);
            }
        }

        if let Some(path) = icon_path {
            match svg::svg_texture(&path, 22) {
                Some(tex) => {
                    let img = gtk4::Image::new();
                    img.set_paintable(Some(&tex));
                    img.set_size_request(22, 22);
                    root.append(&img);
                }
                None => {
                    let img = gtk4::Image::new();
                    img.set_icon_name(Some("image-symbolic"));
                    img.set_size_request(22, 22);
                    root.append(&img);
                }
            }
        } else {
            let badge_text = if !initial.is_empty() {
                initial.to_string()
            } else if !name.is_empty() {
                name.chars().next().unwrap().to_uppercase().to_string()
            } else {
                "•".to_string()
            };
            let badge = gtk4::Label::new(Some(&badge_text));
            badge.add_css_class("stat-badge");
            badge.set_size_request(22, 22);
            root.append(&badge);
        }

        let lbl_title = gtk4::Label::new(Some(name));
        lbl_title.add_css_class("stat-row-title");
        lbl_title.set_hexpand(true);
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        let nums = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
        nums.append(&num_label(ads, "c-red"));
        nums.append(&slash());
        let tr_class = if trackers != "0" { "c-orange" } else { "c-orange-dark" };
        nums.append(&num_label(trackers, tr_class));
        nums.append(&slash());
        nums.append(&num_label(total, "c-lightgray"));
        root.append(&nums);

        let chev = gtk4::Label::new(Some("›"));
        chev.add_css_class("stat-chevron");
        chev.add_css_class("chev-padded");
        root.append(&chev);

        let inner = Rc::new(CompanyRowInner {
            root,
            name: name.to_string(),
            clicked_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, _n, _x, _y| {
            if let Some(cb) = i2.clicked_cb.borrow().as_ref() {
                cb(&i2.name);
            }
        });
        inner.root.add_controller(gesture);

        CompanyRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }
}

fn num_label(text: &str, class: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("stat-num");
    l.add_css_class(class);
    l.add_css_class("bold");
    l
}

fn slash() -> gtk4::Label {
    let l = gtk4::Label::new(Some("/"));
    l.add_css_class("stat-slash");
    l
}

struct CompaniesViewInner {
    root: gtk4::Box,
    input_search: gtk4::Entry,
    rows_container: gtk4::Box,
    all_companies: RefCell<Vec<CompanyStat>>,
    back_cb: RefCell<Option<Box<dyn Fn()>>>,
    time_cb: RefCell<Option<Box<dyn Fn()>>>,
    sort_cb: RefCell<Option<Box<dyn Fn()>>>,
    company_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct CompaniesView {
    inner: Rc<CompaniesViewInner>,
}

impl CompaniesView {
    pub fn new() -> CompaniesView {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        root.add_css_class("page-bg");
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(24);
        root.set_margin_bottom(16);

        let top_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let btn_back = gtk4::Button::new();
        btn_back.set_label("‹ Back");
        btn_back.add_css_class("flat-link");
        top_bar.append(&btn_back);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        top_bar.append(&spacer);
        top_bar.append(&header_button("stat_calendar.svg"));
        top_bar.append(&header_button("stat_filter.svg"));
        root.append(&top_bar);

        let lbl_title = gtk4::Label::new(Some("Companies"));
        lbl_title.add_css_class("page-title");
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        search_box.add_css_class("search-box");
        search_box.set_size_request(-1, 42);
        let search_svg = c::icons_dir().join("search.svg");
        if search_svg.exists() {
            search_box.append(&svg::svg_image("search.svg", 16));
        } else {
            let lens = gtk4::Label::new(Some("🔍"));
            lens.add_css_class("c-muted");
            search_box.append(&lens);
        }
        let input_search = gtk4::Entry::new();
        input_search.set_placeholder_text(Some("Search"));
        input_search.add_css_class("search-input");
        input_search.set_hexpand(true);
        search_box.append(&input_search);
        root.append(&search_box);

        let rows_container = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        rows_container.set_margin_top(4);
        rows_container.set_margin_end(8);
        rows_container.set_margin_bottom(16);
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_child(Some(&rows_container));
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");
        root.append(&scroll);

        let entry = input_search.clone();
        let inner = Rc::new(CompaniesViewInner {
            root,
            input_search,
            rows_container,
            all_companies: RefCell::new(Vec::new()),
            back_cb: RefCell::new(None),
            time_cb: RefCell::new(None),
            sort_cb: RefCell::new(None),
            company_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        btn_back.connect_clicked(move |_| {
            if let Some(cb) = i2.back_cb.borrow().as_ref() {
                cb();
            }
        });
        let i3 = inner.clone();
        entry.connect_changed(move |_e| filter_list(&i3));

        CompaniesView { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn set_companies(&self, companies: &[CompanyStat]) {
        *self.inner.all_companies.borrow_mut() = companies.to_vec();
        filter_list(&self.inner);
    }

    pub fn connect_back_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.back_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_time_filter_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.time_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_sort_filter_clicked(&self, cb: Box<dyn Fn()>) {
        *self.inner.sort_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_company_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.company_cb.borrow_mut() = Some(cb);
    }
}

fn filter_list(inner: &Rc<CompaniesViewInner>) {
    let query = inner.input_search.text().trim().to_lowercase();
    let companies: Vec<CompanyStat> = inner.all_companies.borrow().clone();
    while let Some(child) = inner.rows_container.first_child() {
        inner.rows_container.remove(&child);
    }
    for comp in &companies {
        if !query.is_empty() && !comp.name.to_lowercase().contains(&query) {
            continue;
        }
        let row = CompanyRow::new(&comp.name, &comp.ads, &comp.trackers, &comp.total, &comp.icon, &comp.initial);
        let target = inner.clone();
        row.connect_clicked(Box::new(move |n: &str| {
            if let Some(cb) = target.company_cb.borrow().as_ref() {
                cb(n);
            }
        }));
        inner.rows_container.append(row.root());
    }
}

fn header_button(icon_fn: &str) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("flat-icon-btn");
    btn.set_size_request(32, 32);
    btn.set_child(Some(&svg::svg_image(icon_fn, 20)));
    btn
}
