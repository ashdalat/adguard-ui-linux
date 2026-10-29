// Port of adguard_modern/widgets/feature_icons.py
use gtk4::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::constants as c;
use crate::i18n;
use super::svg;

struct ButtonInfo {
    btn: gtk4::Button,
    image: gtk4::Image,
    green: &'static str,
    gray: &'static str,
    i18n_key: &'static str,
}

struct Inner {
    root: gtk4::Box,
    overall_enabled: RefCell<bool>,
    feature_states: RefCell<HashMap<String, bool>>,
    buttons: HashMap<String, ButtonInfo>,
    toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
}

#[derive(Clone)]
pub struct FeatureIconsRow {
    inner: Rc<Inner>,
}

const ITEMS: [(&str, &str, &str, &str); 4] = [
    ("feat_adblock.svg", "feat_adblock_gray.svg", "feat_ad_blocking", "ad_blocking"),
    ("feat_tracking.svg", "feat_tracking_gray.svg", "feat_tracking", "tracking"),
    ("feat_annoyance.svg", "feat_malware_gray.svg", "feat_annoyance", "annoyance"),
    ("feat_dns.svg", "feat_browsing_gray.svg", "feat_dns", "dns"),
];

impl FeatureIconsRow {
    pub fn new() -> FeatureIconsRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 24);
        root.add_css_class("feature-icons-row");

        let mut buttons = HashMap::new();
        let mut feature_states = HashMap::new();
        for (_, _, _, key) in ITEMS {
            feature_states.insert(key.to_string(), true);
        }

        for (green_fn, gray_fn, i18n_key, key) in ITEMS {
            let btn = gtk4::Button::new();
            btn.add_css_class("feature-icon-btn");
            btn.set_size_request(36, 36);
            btn.set_tooltip_text(Some(&i18n::tr(i18n_key)));

            let image = svg::svg_image(green_fn, 24);
            btn.set_child(Some(&image));
            root.append(&btn);

            buttons.insert(
                key.to_string(),
                ButtonInfo {
                    btn,
                    image,
                    green: green_fn,
                    gray: gray_fn,
                    i18n_key,
                },
            );
        }

        let inner = Rc::new(Inner {
            root,
            overall_enabled: RefCell::new(true),
            feature_states: RefCell::new(feature_states),
            buttons,
            toggled_cb: RefCell::new(None),
        });
        let row = FeatureIconsRow { inner: inner.clone() };

        for (key, info) in inner.buttons.iter() {
            let r2 = row.clone();
            let k = key.clone();
            info.btn.connect_clicked(move |_| r2.on_button_clicked(&k));
        }

        i18n::connect_language_changed(Box::new({
            let r2 = row.clone();
            move |_| r2.retranslate_ui()
        }));
        row.retranslate_ui();

        row
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_feature_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.toggled_cb.borrow_mut() = Some(cb);
    }

    fn on_button_clicked(&self, key: &str) {
        let new_state = *self
            .inner
            .feature_states
            .borrow()
            .get(key)
            .unwrap_or(&true);
        let new_state = !new_state;
        self.inner.feature_states.borrow_mut().insert(key.to_string(), new_state);
        self.update_icon(key);
        if let Some(cb) = self.inner.toggled_cb.borrow().as_ref() {
            cb(key, new_state);
        }
    }

    pub fn set_feature_state(&self, key: &str, is_active: bool) {
        if self.inner.feature_states.borrow().contains_key(key) {
            self.inner
                .feature_states
                .borrow_mut()
                .insert(key.to_string(), is_active);
            self.update_icon(key);
        }
    }

    pub fn set_enabled_state(&self, is_enabled: bool) {
        *self.inner.overall_enabled.borrow_mut() = is_enabled;
        let keys: Vec<String> = self.inner.feature_states.borrow().keys().cloned().collect();
        for key in keys {
            self.update_icon(&key);
        }
    }

    fn update_icon(&self, key: &str) {
        let info = match self.inner.buttons.get(key) {
            Some(i) => i,
            None => return,
        };
        let active = *self.inner.overall_enabled.borrow()
            && *self.inner.feature_states.borrow().get(key).unwrap_or(&true);
        let filename = if active { info.green } else { info.gray };
        if let Some(tex) = svg::svg_texture(&c::icons_dir().join(filename), 24) {
            info.image.set_paintable(Some(&tex));
        }
        let state = *self.inner.feature_states.borrow().get(key).unwrap_or(&true);
        let status_text = if state { i18n::tr("status_enabled") } else { i18n::tr("status_disabled") };
        let title = i18n::tr(info.i18n_key);
        info.btn.set_tooltip_text(Some(&format!("{} ({})", title, status_text)));
    }

    pub fn retranslate_ui(&self) {
        let keys: Vec<String> = self.inner.feature_states.borrow().keys().cloned().collect();
        for key in keys {
            self.update_icon(&key);
        }
    }
}
