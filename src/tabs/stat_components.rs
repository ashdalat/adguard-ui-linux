// Port of adguard_modern/tabs/stat_components.py — the statistics summary bar
// and the company/domain list row.
use gtk4::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::constants as c;
use crate::widgets::svg;

struct Col {
    label: gtk4::Label,
    color: String,
    size: String,
    bold: bool,
}

struct ThreeColsInner {
    root: gtk4::Box,
    cols: Vec<Col>,
}

#[derive(Clone)]
pub struct StatThreeCols {
    inner: Rc<ThreeColsInner>,
}

fn markup(text: &str, color: &str, size: &str, bold: bool) -> String {
    let escaped = gtk4::glib::markup_escape_text(text);
    if bold {
        format!("<span color=\"{}\" font_size=\"{}\" font_weight=\"bold\">{}</span>", color, size, escaped)
    } else {
        format!("<span color=\"{}\" font_size=\"{}\">{}</span>", color, size, escaped)
    }
}

impl StatThreeCols {
    pub fn new(
        cols: [(&str, &str, &str); 3],
    ) -> StatThreeCols {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 24);
        root.set_margin_top(8);
        root.set_margin_bottom(16);

        let mut stored: Vec<Col> = Vec::new();
        for (val, lbl, color) in cols {
            let col_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            col_box.set_hexpand(true);

            let lbl_val = gtk4::Label::new(None);
            lbl_val.set_markup(&markup(val, color, "18pt", true));
            lbl_val.set_halign(gtk4::Align::Start);
            col_box.append(&lbl_val);

            // Python keeps the same colour unless it is the neutral #E5E5EA,
            // which is rendered as the muted text colour.
            let title_color = if color == "#E5E5EA" { c::COLOR_TEXT_MUTED } else { color };
            let lbl_title = gtk4::Label::new(None);
            lbl_title.set_markup(&markup(lbl, title_color, "10pt", false));
            lbl_title.set_halign(gtk4::Align::Start);
            col_box.append(&lbl_title);

            let gap = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            gap.set_size_request(-1, 6);
            col_box.append(&gap);

            let bar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            bar.add_css_class("stat-bar");
            bar.add_css_class(bar_class(color));
            bar.set_size_request(-1, 4);
            col_box.append(&bar);

            root.append(&col_box);
            stored.push(Col {
                label: lbl_val,
                color: color.to_string(),
                size: "18pt".to_string(),
                bold: true,
            });
        }

        StatThreeCols {
            inner: Rc::new(ThreeColsInner { root, cols: stored }),
        }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_values(&self, val1: &str, val2: &str, val3: &str) {
        for (i, val) in [val1, val2, val3].iter().enumerate() {
            if let Some(col) = self.inner.cols.get(i) {
                col.label.set_markup(&markup(val, &col.color, &col.size, col.bold));
            }
        }
    }
}

fn bar_class(color: &str) -> &'static str {
    match color {
        "#E5E5EA" | "#FFFFFF" => "bar-white",
        "#5EBB6A" => "bar-green",
        "#E5983B" | "#F5A623" | "#E67E22" => "bar-orange",
        "#E25642" | "#E05D52" => "bar-red",
        _ => "bar-white",
    }
}

struct RowInner {
    root: gtk4::Box,
    title: String,
    lbl_ads: gtk4::Label,
    lbl_tr: gtk4::Label,
    lbl_tot: gtk4::Label,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct StatListRow {
    inner: Rc<RowInner>,
}

impl StatListRow {
    pub fn new(
        icon_name: &str,
        title: &str,
        ads_count: &str,
        trackers_count: &str,
        total_count: &str,
        initial_letter: &str,
    ) -> StatListRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
        root.add_css_class("stat-row");
        root.set_size_request(-1, 46);
        root.set_margin_start(10);
        root.set_margin_end(14);

        let icons = c::icons_dir();
        let mut icon_path: Option<PathBuf> = None;
        if !icon_name.is_empty() {
            let cand1 = icons.join("companies").join(icon_name);
            let cand2 = icons.join(icon_name);
            if cand1.exists() {
                icon_path = Some(cand1);
            } else if cand2.exists() {
                icon_path = Some(cand2);
            }
        }
        if icon_path.is_none() && initial_letter.is_empty() {
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
                    img.add_css_class("svg-icon");
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
            let letter = if !initial_letter.is_empty() {
                initial_letter.to_string()
            } else if !title.is_empty() {
                title.chars().next().unwrap().to_uppercase().to_string()
            } else {
                "•".to_string()
            };
            let badge = gtk4::Label::new(Some(&letter));
            badge.add_css_class("stat-badge");
            badge.set_size_request(22, 22);
            root.append(&badge);
        }

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("stat-row-title");
        lbl_title.set_hexpand(true);
        lbl_title.set_halign(gtk4::Align::Start);
        root.append(&lbl_title);

        let nums = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
        let lbl_ads = colored(ads_count, "c-red", true);
        nums.append(&lbl_ads);
        nums.append(&slash());
        let tr_class = if trackers_count != "0" { "c-orange" } else { "c-orange-dark" };
        let lbl_tr = colored(trackers_count, tr_class, true);
        nums.append(&lbl_tr);
        nums.append(&slash());
        let lbl_tot = colored(total_count, "c-lightgray", true);
        nums.append(&lbl_tot);
        root.append(&nums);

        let chevron = c::icons_dir().join("chevron_right.svg");
        if chevron.exists() {
            root.append(&svg::svg_image("chevron_right.svg", 14));
        } else {
            let chev = gtk4::Label::new(Some("›"));
            chev.add_css_class("stat-chevron");
            root.append(&chev);
        }

        let inner = Rc::new(RowInner {
            root,
            title: title.to_string(),
            lbl_ads,
            lbl_tr,
            lbl_tot,
            clicked_cb: RefCell::new(None),
        });

        let i2 = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, _n, _x, _y| {
            if let Some(cb) = i2.clicked_cb.borrow().as_ref() {
                cb(&i2.title);
            }
        });
        inner.root.add_controller(gesture);

        StatListRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }

    pub fn update_counts(&self, ads: &str, trackers: &str, total: &str) {
        self.inner.lbl_ads.set_text(ads);
        self.inner.lbl_tr.set_text(trackers);
        self.inner.lbl_tot.set_text(total);
    }
}

fn colored(text: &str, class: &str, bold: bool) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.add_css_class("stat-num");
    l.add_css_class(class);
    if bold {
        l.add_css_class("bold");
    }
    l
}

fn slash() -> gtk4::Label {
    let l = gtk4::Label::new(Some("/"));
    l.add_css_class("stat-slash");
    l
}
