// Port of adguard_modern/tabs/settings_tab.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::svg;

use crate::tabs::{
    settings_about_view::SettingsAboutView,
    settings_advanced_view::SettingsAdvancedView,
    settings_bootstrap_view::SettingsBootstrapView,
    settings_ev_certificates_view::SettingsEvCertificatesView,
    settings_fallback_dns_view::SettingsFallbackDnsView,
    settings_filtering_mode_view::SettingsFilteringModeView,
    settings_frag_delay_view::SettingsFragDelayView,
    settings_general_view::SettingsGeneralView,
    settings_https_filtering_view::SettingsHttpsFilteringView,
    settings_license_view::SettingsLicenseView,
    settings_network_view::SettingsNetworkView,
    settings_proxy_updates_view::SettingsProxyUpdatesView,
    settings_secure_dns_view::SettingsSecureDnsView,
    settings_support_view::SettingsSupportView,
    settings_website_allowlist_view::SettingsWebsiteAllowlistView,
};

// Stack page names (Python used indices; names keep the same order).
const PAGE_MENU: &str = "menu";
const PAGE_APP: &str = "app";
const PAGE_LICENSE: &str = "license";
const PAGE_SUPPORT: &str = "support";
const PAGE_ABOUT: &str = "about";
const PAGE_NETWORK: &str = "network";
const PAGE_MODE: &str = "mode";
const PAGE_HTTPS: &str = "https";
const PAGE_UPDATES_PROXY: &str = "updates_proxy";
const PAGE_ALLOWLIST: &str = "allowlist";
const PAGE_EV: &str = "ev";
const PAGE_ADVANCED: &str = "advanced";
const PAGE_FRAG: &str = "frag";
const PAGE_FALLBACK: &str = "fallback";
const PAGE_BOOTSTRAP: &str = "bootstrap";
const PAGE_SECURE_DNS: &str = "secure_dns";

// ---------------------------------------------------------------------------
// SettingsItemRow
// ---------------------------------------------------------------------------
struct RowInner {
    key: String,
    root: gtk4::Box,
    lbl_title: gtk4::Label,
    lbl_desc: gtk4::Label,
    clicked_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

pub struct SettingsItemRow {
    inner: Rc<RowInner>,
}

impl SettingsItemRow {
    pub fn new(key: &str, icon_name: &str, title: &str, description: &str) -> SettingsItemRow {
        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        root.add_css_class("settings-item-row");
        root.set_margin_top(14);
        root.set_margin_bottom(14);
        root.set_margin_start(12);
        root.set_margin_end(16);

        // Left icon.
        let icon = svg::svg_image(icon_name, 24);
        icon.set_valign(gtk4::Align::Center);
        root.append(&icon);

        // Title & description.
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        text_box.set_hexpand(true);

        let lbl_title = gtk4::Label::new(Some(title));
        lbl_title.add_css_class("lbl-title");
        lbl_title.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_title);

        let lbl_desc = gtk4::Label::new(Some(description));
        lbl_desc.add_css_class("lbl-desc-9");
        lbl_desc.set_halign(gtk4::Align::Start);
        text_box.append(&lbl_desc);

        root.append(&text_box);

        // Chevron >.
        let chevron = svg::svg_image("chevron_right.svg", 16);
        chevron.set_valign(gtk4::Align::Center);
        root.append(&chevron);

        let inner = Rc::new(RowInner {
            key: key.to_string(),
            root,
            lbl_title,
            lbl_desc,
            clicked_cb: RefCell::new(None),
        });

        let i = inner.clone();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |_g, n, _x, _y| {
            // ignore clicks while the row is disabled
            if !_g.widget().map(|w| w.is_sensitive()).unwrap_or(true) { return; }
            if n == 1 {
                if let Some(cb) = i.clicked_cb.borrow().as_ref() {
                    cb(&i.key);
                }
            }
        });
        inner.root.add_controller(gesture);

        SettingsItemRow { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn update_text(&self, title: &str, desc: &str) {
        self.inner.lbl_title.set_text(title);
        self.inner.lbl_desc.set_text(desc);
    }

    pub fn connect_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.clicked_cb.borrow_mut() = Some(cb);
    }
}

// ---------------------------------------------------------------------------
// SettingsTab
// ---------------------------------------------------------------------------
struct Inner {
    root: gtk4::Box,
    stack: gtk4::Stack,
    lbl_header: gtk4::Label,
    setting_rows: HashMap<String, SettingsItemRow>,
    app_view: SettingsGeneralView,
    license_view: SettingsLicenseView,
    support_view: SettingsSupportView,
    about_view: SettingsAboutView,
    network_view: SettingsNetworkView,
    mode_view: SettingsFilteringModeView,
    https_view: SettingsHttpsFilteringView,
    updates_proxy_view: SettingsProxyUpdatesView,
    allowlist_view: SettingsWebsiteAllowlistView,
    ev_view: SettingsEvCertificatesView,
    advanced_view: SettingsAdvancedView,
    frag_view: SettingsFragDelayView,
    fallback_view: SettingsFallbackDnsView,
    bootstrap_view: SettingsBootstrapView,
    secure_dns_view: SettingsSecureDnsView,
    mode_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
}

#[derive(Clone)]
pub struct SettingsTab {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl SettingsTab {
    pub fn new() -> SettingsTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");

        let stack = gtk4::Stack::new();

        // 0. Menu root view.
        let menu_view = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
        menu_view.add_css_class("page-bg");
        menu_view.set_margin_top(28);
        menu_view.set_margin_bottom(20);
        menu_view.set_margin_start(36);
        menu_view.set_margin_end(36);

        let lbl_header = gtk4::Label::new(Some("Settings"));
        lbl_header.add_css_class("page-title");
        lbl_header.set_halign(gtk4::Align::Start);
        menu_view.append(&lbl_header);
        menu_view.append(&spacer(6));

        let mut setting_rows: HashMap<String, SettingsItemRow> = HashMap::new();
        let items = [
            ("app_settings", "settings.svg", "App settings", "Manage theme, filtering, logging, and other app settings"),
            ("license", "feat_license.svg", "License", "Activate, renew, or upgrade your license"),
            ("support", "feat_support.svg", "Support", "Report an issue or leave feedback"),
            ("about", "feat_about.svg", "About", "App info and legal documents"),
        ];

        // Rows need the tab's Inner for the click handler; build rows first,
        // then wire them to on_item_clicked once Inner exists.
        let mut row_widgets: Vec<(String, SettingsItemRow)> = Vec::new();
        for (key, icon_fn, title, desc) in items {
            let row = SettingsItemRow::new(key, icon_fn, title, desc);
            menu_view.append(row.root());
            row_widgets.push((key.to_string(), row));
        }

        // 1..15 subviews.
        let app_view = SettingsGeneralView::new();
        let license_view = SettingsLicenseView::new();
        let support_view = SettingsSupportView::new();
        let about_view = SettingsAboutView::new();
        let network_view = SettingsNetworkView::new();
        let mode_view = SettingsFilteringModeView::new();
        let https_view = SettingsHttpsFilteringView::new();
        let updates_proxy_view = SettingsProxyUpdatesView::new();
        let allowlist_view = SettingsWebsiteAllowlistView::new();
        let ev_view = SettingsEvCertificatesView::new();
        let advanced_view = SettingsAdvancedView::new();
        let frag_view = SettingsFragDelayView::new();
        let fallback_view = SettingsFallbackDnsView::new();
        let bootstrap_view = SettingsBootstrapView::new();
        let secure_dns_view = SettingsSecureDnsView::new();

        stack.add_named(&menu_view, Some(PAGE_MENU));
        stack.add_named(app_view.root(), Some(PAGE_APP));
        stack.add_named(license_view.root(), Some(PAGE_LICENSE));
        stack.add_named(support_view.root(), Some(PAGE_SUPPORT));
        stack.add_named(about_view.root(), Some(PAGE_ABOUT));
        stack.add_named(network_view.root(), Some(PAGE_NETWORK));
        stack.add_named(mode_view.root(), Some(PAGE_MODE));
        stack.add_named(https_view.root(), Some(PAGE_HTTPS));
        stack.add_named(updates_proxy_view.root(), Some(PAGE_UPDATES_PROXY));
        stack.add_named(allowlist_view.root(), Some(PAGE_ALLOWLIST));
        stack.add_named(ev_view.root(), Some(PAGE_EV));
        stack.add_named(advanced_view.root(), Some(PAGE_ADVANCED));
        stack.add_named(frag_view.root(), Some(PAGE_FRAG));
        stack.add_named(fallback_view.root(), Some(PAGE_FALLBACK));
        stack.add_named(bootstrap_view.root(), Some(PAGE_BOOTSTRAP));
        stack.add_named(secure_dns_view.root(), Some(PAGE_SECURE_DNS));

        for (key, row) in row_widgets {
            setting_rows.insert(key, row);
        }

        root.append(&stack);

        let inner = Rc::new(Inner {
            root,
            stack,
            lbl_header,
            setting_rows,
            app_view,
            license_view,
            support_view,
            about_view,
            network_view,
            mode_view,
            https_view,
            updates_proxy_view,
            allowlist_view,
            ev_view,
            advanced_view,
            frag_view,
            fallback_view,
            bootstrap_view,
            secure_dns_view,
            mode_cb: RefCell::new(None),
        });

        // Row clicks.
        let i0 = inner.clone();
        for row in inner.setting_rows.values() {
            let i = i0.clone();
            row.connect_clicked(Box::new(move |key| on_item_clicked(&i, key)));
        }

        // Back navigation wiring (mirrors the Python lambdas).
        wire_back(&inner, PAGE_MENU, &inner.app_view);
        wire_back(&inner, PAGE_MENU, &inner.license_view);
        wire_back(&inner, PAGE_MENU, &inner.support_view);
        wire_back(&inner, PAGE_MENU, &inner.about_view);
        wire_back(&inner, PAGE_APP, &inner.network_view);
        wire_back(&inner, PAGE_UPDATES_PROXY, &inner.network_view);
        wire_back(&inner, PAGE_MENU, &inner.mode_view);
        wire_back(&inner, PAGE_MENU, &inner.https_view);
        wire_back(&inner, PAGE_HTTPS, &inner.allowlist_view);
        wire_back(&inner, PAGE_HTTPS, &inner.ev_view);
        wire_back(&inner, PAGE_APP, &inner.advanced_view);
        wire_back(&inner, PAGE_ADVANCED, &inner.frag_view);
        wire_back(&inner, PAGE_ADVANCED, &inner.fallback_view);
        wire_back(&inner, PAGE_ADVANCED, &inner.bootstrap_view);
        wire_back(&inner, PAGE_ADVANCED, &inner.secure_dns_view);

        // Nested-open wiring.
        let i_net = inner.clone();
        inner.app_view.connect_open_network_requested(Box::new(move || {
            i_net.network_view.refresh_states();
            i_net.stack.set_visible_child_name(PAGE_NETWORK);
        }));
        let i_adv = inner.clone();
        inner.app_view.connect_open_advanced_requested(Box::new(move || {
            i_adv.stack.set_visible_child_name(PAGE_ADVANCED);
        }));

        let i_up = inner.clone();
        inner.network_view.connect_open_updates_proxy_requested(Box::new(move || {
            i_up.stack.set_visible_child_name(PAGE_UPDATES_PROXY);
        }));

        let i_al = inner.clone();
        inner.https_view.connect_open_allowlist_requested(Box::new(move || {
            i_al.stack.set_visible_child_name(PAGE_ALLOWLIST);
        }));
        let i_ev = inner.clone();
        inner.https_view.connect_open_ev_requested(Box::new(move || {
            i_ev.stack.set_visible_child_name(PAGE_EV);
        }));

        // mode_changed -> filtering_mode_changed signal.
        let i_mode = inner.clone();
        inner.mode_view.connect_mode_changed(Box::new(move |mode| {
            if let Some(cb) = i_mode.mode_cb.borrow().as_ref() {
                cb(mode);
            }
        }));

        // Advanced <-> nested subview change signals.
        let i_frag = inner.clone();
        inner.frag_view.connect_delay_changed(Box::new(move |val| {
            i_frag.advanced_view.update_frag_delay(val);
        }));

        let i_fb = inner.clone();
        inner.fallback_view.connect_fallbacks_changed(Box::new(move |val| {
            i_fb.advanced_view.update_fallbacks(val);
        }));
        let i_bs = inner.clone();
        inner.bootstrap_view.connect_bootstraps_changed(Box::new(move |val| {
            i_bs.advanced_view.update_bootstraps(val);
        }));
        let i_sd = inner.clone();
        inner.secure_dns_view.connect_mode_changed(Box::new(move |mode| {
            i_sd.advanced_view.update_secure_dns(mode);
        }));

        // Advanced open-row wiring (load + navigate).
        let i3 = inner.clone();
        inner.advanced_view.connect_open_frag_delay_requested(Box::new(move || {
            i3.frag_view.load_delay();
            i3.stack.set_visible_child_name(PAGE_FRAG);
        }));
        let i4 = inner.clone();
        inner.advanced_view.connect_open_fallback_dns_requested(Box::new(move || {
            i4.fallback_view.load_config();
            i4.stack.set_visible_child_name(PAGE_FALLBACK);
        }));
        let i5 = inner.clone();
        inner.advanced_view.connect_open_bootstrap_requested(Box::new(move || {
            i5.bootstrap_view.load_bootstraps();
            i5.stack.set_visible_child_name(PAGE_BOOTSTRAP);
        }));
        let i6 = inner.clone();
        inner.advanced_view.connect_open_secure_dns_requested(Box::new(move || {
            i6.secure_dns_view.load_config();
            i6.stack.set_visible_child_name(PAGE_SECURE_DNS);
        }));

        let i7 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i7)));

        retranslate_ui(&inner);

        SettingsTab { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_filtering_mode_changed(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.mode_cb.borrow_mut() = Some(cb);
    }

    // Interface assumed by window.rs (Python: settings_tab.network_view.refresh_states()).
    pub fn refresh_network_states(&self) {
        self.inner.network_view.refresh_states();
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    // Python methods never called from main_window.py (kept for parity).
    #[allow(dead_code)]
    pub fn navigate_to_filtering_mode(&self) {
        self.inner.network_view.refresh_states();
        self.inner.stack.set_visible_child_name(PAGE_MODE);
    }

    #[allow(dead_code)]
    pub fn navigate_to_https_filtering(&self) {
        self.inner.network_view.refresh_states();
        self.inner.stack.set_visible_child_name(PAGE_HTTPS);
    }

    #[allow(dead_code)]
    pub fn navigate_to_updates_proxy(&self) {
        self.inner.stack.set_visible_child_name(PAGE_UPDATES_PROXY);
    }

    /// Debug-only: show a named sub-page directly (see window.rs `--open-page`).
    pub fn debug_show_page(&self, name: &str) -> bool {
        self.inner.stack.set_visible_child_name(name);
        self.inner.stack.visible_child_name().as_deref() == Some(name)
    }
}

fn on_item_clicked(inner: &Rc<Inner>, key: &str) {
    match key {
        "app_settings" => inner.stack.set_visible_child_name(PAGE_APP),
        "license" => {
            inner.license_view.load_live_license();
            inner.stack.set_visible_child_name(PAGE_LICENSE);
        }
        "support" => inner.stack.set_visible_child_name(PAGE_SUPPORT),
        "about" => inner.stack.set_visible_child_name(PAGE_ABOUT),
        _ => {}
    }
}

fn wire_back(inner: &Rc<Inner>, target: &str, view: &dyn BackConnectable) {
    let i = inner.clone();
    let target = target.to_string();
    view.connect_back(Box::new(move || {
        i.stack.set_visible_child_name(&target);
    }));
}

// Small trait so all subviews can be wired uniformly.
trait BackConnectable {
    fn connect_back(&self, cb: Box<dyn Fn()>);
}

macro_rules! impl_back {
    ($t:ty) => {
        impl BackConnectable for $t {
            fn connect_back(&self, cb: Box<dyn Fn()>) {
                self.connect_back_requested(cb);
            }
        }
    };
}

impl_back!(SettingsGeneralView);
impl_back!(SettingsLicenseView);
impl_back!(SettingsSupportView);
impl_back!(SettingsAboutView);
impl_back!(SettingsNetworkView);
impl_back!(SettingsFilteringModeView);
impl_back!(SettingsHttpsFilteringView);
impl_back!(SettingsProxyUpdatesView);
impl_back!(SettingsWebsiteAllowlistView);
impl_back!(SettingsEvCertificatesView);
impl_back!(SettingsAdvancedView);
impl_back!(SettingsFragDelayView);
impl_back!(SettingsFallbackDnsView);
impl_back!(SettingsBootstrapView);
impl_back!(SettingsSecureDnsView);

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.lbl_header.set_text(&i18n::tr("settings_title"));
    let row_keys: [(&str, &str, &str); 4] = [
        ("app_settings", "settings_app_title", "settings_app_desc"),
        ("license", "settings_license_title", "settings_license_desc"),
        ("support", "settings_support_title", "settings_support_desc"),
        ("about", "settings_about_title", "settings_about_desc"),
    ];
    for (key, t_k, d_k) in row_keys {
        if let Some(row) = inner.setting_rows.get(key) {
            row.update_text(&i18n::tr(t_k), &i18n::tr(d_k));
        }
    }
    // Python only retranslates app_view here (other subviews subscribe to
    // i18n themselves).
    inner.app_view.retranslate_ui();
}
