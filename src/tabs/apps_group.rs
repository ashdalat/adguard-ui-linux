// Port of adguard_modern/tabs/apps_group.py
use gtk4::prelude::*;

#[derive(Clone)]
pub struct AppCollapsibleGroup {
    root: gtk4::Box,
    arrow_lbl: gtk4::Label,
    title_lbl: gtk4::Label,
    content_box: gtk4::Box,
}

impl AppCollapsibleGroup {
    pub fn new(title: &str) -> AppCollapsibleGroup {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 2);

        // Header button.
        let header_btn = gtk4::Button::new();
        header_btn.add_css_class("app-group-header");

        let h_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        h_layout.set_margin_top(4);
        h_layout.set_margin_bottom(4);

        let arrow_lbl = gtk4::Label::new(Some("⌄"));
        arrow_lbl.add_css_class("app-group-arrow");
        h_layout.append(&arrow_lbl);

        let title_lbl = gtk4::Label::new(Some(title));
        title_lbl.add_css_class("lbl-14-bold-white");
        h_layout.append(&title_lbl);
        header_btn.set_child(Some(&h_layout));
        root.append(&header_btn);

        // Content container.
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        content_box.add_css_class("transparent-container");
        content_box.set_margin_bottom(8);
        root.append(&content_box);

        let a = arrow_lbl.clone();
        let c = content_box.clone();
        header_btn.connect_clicked(move |_| {
            let is_vis = c.get_visible();
            c.set_visible(!is_vis);
            a.set_text(if is_vis { "›" } else { "⌄" });
        });

        AppCollapsibleGroup {
            root,
            arrow_lbl,
            title_lbl,
            content_box: content_box,
        }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn set_title(&self, title: &str) {
        self.title_lbl.set_text(title);
    }

    pub fn toggle_collapse(&self) {
        let is_vis = self.content_box.get_visible();
        self.content_box.set_visible(!is_vis);
        self.arrow_lbl.set_text(if is_vis { "›" } else { "⌄" });
    }

    pub fn add_widget(&self, widget: &impl IsA<gtk4::Widget>) {
        self.content_box.append(widget);
    }

    pub fn insert_widget(&self, index: i32, widget: &impl IsA<gtk4::Widget>) {
        // gtk4 Box has no positional insert; walk the child list to find the
        // sibling currently at `index`, then insert the new widget right before
        // it (i.e. after the previous sibling, or prepend if it is first).
        let mut prev: Option<gtk4::Widget> = None;
        let mut i = 0;
        let mut child = self.content_box.first_child();
        while let Some(c) = child.clone() {
            if i == index {
                if let Some(p) = prev {
                    self.content_box.insert_child_after(widget, Some(&p));
                } else {
                    self.content_box.prepend(widget);
                }
                return;
            }
            prev = Some(c.clone());
            i += 1;
            child = c.next_sibling();
        }
        // index beyond the end: append.
        self.content_box.append(widget);
    }
}
