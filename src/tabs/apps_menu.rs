// Port of adguard_modern/tabs/apps_menu.py
// QMenu -> gtk4::Popover containing a vertical Box of menu-item buttons
// (CSS classes .menu-popover / .menu-item). Action dispatch via click handlers.
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

pub struct AppManagementMenu {
    root: gtk4::Popover,
    on_action: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl AppManagementMenu {
    pub fn new(parent: Option<&impl IsA<gtk4::Widget>>, on_action: Option<Box<dyn Fn(&str)>>) -> AppManagementMenu {
        let root = gtk4::Popover::new();
        root.add_css_class("menu-popover");
        if let Some(p) = parent {
            root.set_parent(p);
        }

        // (label, action_key)
        let items: Vec<(String, String)> = vec![
            (crate::i18n::tr("menu_route_all"), "route_all".to_string()),
            (crate::i18n::tr("menu_stop_route"), "stop_route".to_string()),
            (String::new(), String::new()),
            (crate::i18n::tr("menu_filter_all"), "filter_all".to_string()),
            (crate::i18n::tr("menu_stop_filter"), "stop_filter".to_string()),
            (String::new(), String::new()),
            (crate::i18n::tr("menu_open_stats"), "open_stats".to_string()),
            (String::new(), String::new()),
            (crate::i18n::tr("menu_reset_default"), "reset_default".to_string()),
        ];

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        for (label, _key) in &items {
            if label.is_empty() {
                let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
                sep.add_css_class("menu-separator");
                content.append(&sep);
                continue;
            }
            let btn = gtk4::Button::new();
            btn.add_css_class("menu-item");
            btn.set_label(label);
            btn.set_halign(gtk4::Align::Start);
            content.append(&btn);
        }

        root.set_child(Some(&content));

        let on_action = Rc::new(RefCell::new(on_action));
        let a = on_action.clone();

        // Walk the content box and wire each menu-item button's click.
        let mut idx = 0;
        let mut child = content.first_child();
        while let Some(w) = child.clone() {
            if let Some(btn) = w.clone().downcast::<gtk4::Button>().ok() {
                if let Some(key) = items.get(idx).and_then(|(_, k)| if k.is_empty() { None } else { Some(k.clone()) }) {
                    let a2 = a.clone();
                    btn.connect_clicked(move |_| {
                        if let Some(cb) = a2.borrow().as_ref() {
                            cb(&key);
                        }
                    });
                }
                idx += 1;
            }
            child = w.next_sibling();
        }

        AppManagementMenu { root, on_action: a }
    }

    pub fn root(&self) -> &gtk4::Popover {
        &self.root
    }

    pub fn popup(&self) {
        self.root.popup();
    }

    pub fn connect_action(&self, cb: Box<dyn Fn(&str)>) {
        *self.on_action.borrow_mut() = Some(cb);
    }
}