use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::feature_icons::FeatureIconsRow;
use crate::widgets::toggle_switch::ToggleSwitch;
use crate::tabs::home_network_section::HomeNetworkSection;
use crate::tabs::settings_https_filtering_view::SettingsHttpsFilteringView;
use crate::tabs::settings_filtering_mode_view::SettingsFilteringModeView;
use crate::tabs::settings_proxy_updates_view::SettingsProxyUpdatesView;
use crate::tabs::settings_website_allowlist_view::SettingsWebsiteAllowlistView;
use crate::tabs::settings_ev_certificates_view::SettingsEvCertificatesView;

// Internal stack page names (Python QStackedWidget pages, in insertion order).
const PAGE_MAIN: &str = "main";
const PAGE_HTTPS: &str = "https";
const PAGE_MODE: &str = "mode";
const PAGE_UPDATES: &str = "updates";
const PAGE_ALLOWLIST: &str = "allowlist";
const PAGE_EV: &str = "ev";

struct Inner {
    root: gtk4::Box,
    stack: gtk4::Stack,
    feature_icons: FeatureIconsRow,
    toggle_switch: ToggleSwitch,
    lbl_headline: gtk4::Label,
    lbl_slogan: gtk4::Label,
    lbl_detail: gtk4::Label,
    network_section: HomeNetworkSection,
    https_view: SettingsHttpsFilteringView,
    mode_view: SettingsFilteringModeView,
    updates_proxy_view: SettingsProxyUpdatesView,
    allowlist_view: SettingsWebsiteAllowlistView,
    ev_view: SettingsEvCertificatesView,
    is_running: RefCell<bool>,
    current_detail: RefCell<String>,
    /// True while the switch is being synced from backend status.
    /// Python relied on Qt's signal timing here; without this guard the status
    /// sync feeds back into `on_switch_toggled` and the app issues a real
    /// "stop" command, turning protection off by itself.
    syncing: RefCell<bool>,
    protection_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
    feature_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
    shortcut_cb: RefCell<Option<Box<dyn Fn(&str)>>>,
    browser_proxy_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
    home_https_cb: RefCell<Option<Box<dyn Fn(bool)>>>,
}

#[derive(Clone)]
pub struct HomeTab {
    inner: Rc<Inner>,
}

fn spacer(px: i32) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    b.set_size_request(-1, px);
    b
}

impl HomeTab {
    pub fn new() -> HomeTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("home-tab");

        let stack = gtk4::Stack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        // Page 0: main home view inside a scroll area.
        let main_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("home-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.add_css_class("home-container");
        // Python: layout.setContentsMargins(40, 20, 40, 30) + Qt.AlignCenter.
        // Without these the widgets sat flush against the content edges and the
        // card widths were driven by the widest child, which is what made the
        // Home page look laid out differently from the Qt original.
        container.set_margin_start(40);
        container.set_margin_end(40);
        container.set_margin_top(20);
        container.set_margin_bottom(30);
        // Python: layout.setAlignment(Qt.AlignmentFlag.AlignCenter). Keeping the
        // column at its natural width (instead of filling the viewport) is what
        // keeps the 500px network column at 500px; with Fill, the extra width was
        // handed down and the cards grew to ~570px.
        container.set_halign(gtk4::Align::Center);
        let feature_icons = FeatureIconsRow::new();
        feature_icons.root().set_halign(gtk4::Align::Center);
        container.append(feature_icons.root());
        container.append(&spacer(16));

        let lbl_headline = gtk4::Label::new(Some("Protection is enabled"));
        lbl_headline.add_css_class("home-headline");
        lbl_headline.set_halign(gtk4::Align::Center);
        container.append(&lbl_headline);
        container.append(&spacer(4));

        let lbl_slogan = gtk4::Label::new(Some("Surf safely — we've got your back"));
        lbl_slogan.add_css_class("home-slogan");
        lbl_slogan.set_halign(gtk4::Align::Center);
        container.append(&lbl_slogan);
        container.append(&spacer(20));

        let toggle_switch = ToggleSwitch::new(96, 50);
        toggle_switch.root().set_halign(gtk4::Align::Center);
        container.append(toggle_switch.root());
        container.append(&spacer(8));

        let lbl_detail = gtk4::Label::new(Some(""));
        lbl_detail.add_css_class("home-detail");
        lbl_detail.set_halign(gtk4::Align::Center);
        container.append(&lbl_detail);
        container.append(&spacer(22));

        let network_section = HomeNetworkSection::new();
        network_section.root().set_halign(gtk4::Align::Center);
        container.append(network_section.root());

        // Debug: `--measure` prints the allocated size of the Home sections once
        // the layout has settled, so width regressions can be spotted without
        // guessing at pixel budgets.
        if std::env::args().any(|a| a == "--measure") {
            let ns = network_section.root().clone();
            let cont = container.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(1500), move || {
                eprintln!(
                    "[measure] home container={}x{} network_section={}x{}",
                    cont.width(),
                    cont.height(),
                    ns.width(),
                    ns.height()
                );
                let m = ns.measure(gtk4::Orientation::Horizontal, -1);
                eprintln!(
                    "[measure] section min={} natural={} halign={:?} hexpand={}",
                    m.0,
                    m.1,
                    ns.halign(),
                    ns.hexpands()
                );
                let cm = cont.measure(gtk4::Orientation::Horizontal, -1);
                eprintln!("[measure] container min={} natural={}", cm.0, cm.1);
                let mut ch = cont.first_child();
                let mut j = 0;
                while let Some(c) = ch {
                    let mm = c.measure(gtk4::Orientation::Horizontal, -1);
                    eprintln!(
                        "[measure] cont.child[{j}] {} w={} natural={}",
                        c.type_().name(),
                        c.width(),
                        mm.1
                    );
                    j += 1;
                    ch = c.next_sibling();
                }
                let mut child = ns.first_child();
                let mut i = 0;
                while let Some(c) = child {
                    eprintln!(
                        "[measure]   child[{i}] {} w={} natural={} h={}",
                        c.type_().name(),
                        c.width(),
                        c.measure(gtk4::Orientation::Horizontal, -1).1,
                        c.height()
                    );
                    i += 1;
                    child = c.next_sibling();
                }
                gtk4::glib::ControlFlow::Break
            });
        }

        scroll.set_child(Some(&container));
        main_container.append(&scroll);

        // Subviews.
        let https_view = SettingsHttpsFilteringView::new();
        let mode_view = SettingsFilteringModeView::new();
        let updates_proxy_view = SettingsProxyUpdatesView::new();
        let allowlist_view = SettingsWebsiteAllowlistView::new();
        let ev_view = SettingsEvCertificatesView::new();

        stack.add_named(&main_container, Some(PAGE_MAIN));
        stack.add_named(https_view.root(), Some(PAGE_HTTPS));
        stack.add_named(mode_view.root(), Some(PAGE_MODE));
        stack.add_named(updates_proxy_view.root(), Some(PAGE_UPDATES));
        stack.add_named(allowlist_view.root(), Some(PAGE_ALLOWLIST));
        stack.add_named(ev_view.root(), Some(PAGE_EV));
        stack.set_visible_child_name(PAGE_MAIN);
        root.append(&stack);

        let inner = Rc::new(Inner {
            root,
            stack,
            feature_icons,
            toggle_switch,
            lbl_headline,
            lbl_slogan,
            lbl_detail,
            network_section,
            https_view,
            mode_view,
            updates_proxy_view,
            allowlist_view,
            ev_view,
            is_running: RefCell::new(true),
            current_detail: RefCell::new(String::new()),
            syncing: RefCell::new(false),
            protection_cb: RefCell::new(None),
            feature_cb: RefCell::new(None),
            shortcut_cb: RefCell::new(None),
            browser_proxy_cb: RefCell::new(None),
            home_https_cb: RefCell::new(None),
        });

        // feature_icons.feature_toggled -> HomeTab.feature_toggled
        let i1 = inner.clone();
        inner
            .feature_icons
            .connect_feature_toggled(Box::new(move |key, is_active| {
                if let Some(cb) = i1.feature_cb.borrow().as_ref() {
                    cb(key, is_active);
                }
            }));

        // network_section signals -> stack navigation
        let i2 = inner.clone();
        inner.network_section.connect_open_https_requested(Box::new(move || {
            i2.stack.set_visible_child_name(PAGE_HTTPS);
        }));
        let i3 = inner.clone();
        inner.network_section.connect_open_mode_requested(Box::new(move || {
            i3.mode_view.load_current_mode();
            i3.stack.set_visible_child_name(PAGE_MODE);
        }));

        // Subview back buttons.
        let i4 = inner.clone();
        inner.https_view.connect_back_requested(Box::new(move || {
            refresh_network_states(&i4);
            i4.stack.set_visible_child_name(PAGE_MAIN);
        }));
        let i5 = inner.clone();
        inner.mode_view.connect_back_requested(Box::new(move || {
            refresh_network_states(&i5);
            i5.stack.set_visible_child_name(PAGE_MAIN);
        }));
        let i6 = inner.clone();
        inner.mode_view.connect_mode_changed(Box::new(move |_mode| {
            refresh_network_states(&i6);
        }));
        let i7 = inner.clone();
        inner.mode_view.connect_open_updates_proxy_requested(Box::new(move || {
            i7.updates_proxy_view.load_config();
            i7.stack.set_visible_child_name(PAGE_UPDATES);
        }));
        let i8 = inner.clone();
        inner.updates_proxy_view.connect_back_requested(Box::new(move || {
            i8.stack.set_visible_child_name(PAGE_MODE);
        }));
        let i9 = inner.clone();
        inner.allowlist_view.connect_back_requested(Box::new(move || {
            i9.stack.set_visible_child_name(PAGE_HTTPS);
        }));
        let i10 = inner.clone();
        inner.https_view.connect_open_allowlist_requested(Box::new(move || {
            i10.stack.set_visible_child_name(PAGE_ALLOWLIST);
        }));
        let i11 = inner.clone();
        inner.ev_view.connect_back_requested(Box::new(move || {
            i11.stack.set_visible_child_name(PAGE_HTTPS);
        }));
        let i12 = inner.clone();
        inner.https_view.connect_open_ev_requested(Box::new(move || {
            i12.stack.set_visible_child_name(PAGE_EV);
        }));

        // Big toggle switch.
        let i13 = inner.clone();
        inner.toggle_switch.connect_toggled(Box::new(move |checked| {
            on_switch_toggled(&i13, checked);
        }));

        let i14 = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i14)));

        retranslate_ui(&inner);

        HomeTab { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    /// Debug-only: show a named sub-page directly (see window.rs `--open-page`).
    pub fn debug_show_page(&self, name: &str) -> bool {
        self.inner.stack.set_visible_child_name(name);
        self.inner.stack.visible_child_name().as_deref() == Some(name)
    }

    pub fn connect_protection_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.protection_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_feature_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.feature_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_feature_shortcut_clicked(&self, cb: Box<dyn Fn(&str)>) {
        *self.inner.shortcut_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_browser_proxy_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.browser_proxy_cb.borrow_mut() = Some(cb);
    }

    pub fn connect_home_https_toggled(&self, cb: Box<dyn Fn(bool)>) {
        *self.inner.home_https_cb.borrow_mut() = Some(cb);
    }

    pub fn set_feature_state(&self, key: &str, is_active: bool) {
        self.inner.feature_icons.set_feature_state(key, is_active);
    }

    pub fn refresh_network_states(&self) {
        refresh_network_states(&self.inner);
    }

    pub fn set_protection_state(&self, is_running: bool, detail: &str) {
        self.set_protection_state_animate(is_running, detail, true);
    }

    pub fn set_protection_state_animate(&self, is_running: bool, detail: &str, animate: bool) {
        *self.inner.is_running.borrow_mut() = is_running;
        *self.inner.current_detail.borrow_mut() = detail.to_string();
        self.inner.toggle_switch.root().set_sensitive(true);
        // Syncing a status snapshot must not look like a user click: the switch
        // would emit toggled and issue a real start/stop command.
        *self.inner.syncing.borrow_mut() = true;
        self.inner.toggle_switch.set_checked(is_running, animate);
        *self.inner.syncing.borrow_mut() = false;
        update_texts(&self.inner, is_running, detail);
        self.refresh_network_states();
    }

    pub fn retranslate_ui(&self) {
        let running = *self.inner.is_running.borrow();
        let detail = self.inner.current_detail.borrow().clone();
        update_texts(&self.inner, running, &detail);
        self.inner.feature_icons.retranslate_ui();
        self.inner.network_section.retranslate_ui();
        self.inner.https_view.retranslate_ui();
        self.inner.mode_view.retranslate_ui();
        self.inner.updates_proxy_view.retranslate_ui();
        self.inner.allowlist_view.retranslate_ui();
        self.inner.ev_view.retranslate_ui();
    }
}

fn refresh_network_states(inner: &Rc<Inner>) {
    inner.network_section.refresh_states();
}

fn on_switch_toggled(inner: &Rc<Inner>, is_checked: bool) {
    // Only a real user interaction may drive protection; ignore the echo from
    // set_protection_state (see Inner::syncing).
    if *inner.syncing.borrow() {
        return;
    }
    // Python disables the switch while the backend applies the change.
    inner.toggle_switch.root().set_sensitive(false);
    inner.feature_icons.set_enabled_state(is_checked);
    if is_checked {
        inner.lbl_headline.set_text(&i18n::tr("enabling_protection"));
    } else {
        inner.lbl_headline.set_text(&i18n::tr("disabling_protection"));
    }
    inner.lbl_slogan.set_text(&i18n::tr("give_us_moment"));

    if let Some(cb) = inner.protection_cb.borrow().as_ref() {
        cb(is_checked);
    }
}

fn update_texts(inner: &Rc<Inner>, is_running: bool, detail: &str) {
    inner.feature_icons.set_enabled_state(is_running);
    if is_running {
        inner.lbl_headline.set_text(&i18n::tr("protection_enabled"));
        inner.lbl_slogan.set_text(&i18n::tr("slogan_enabled"));
        // Python: first line only when the detail contains a newline.
        let clean = if !detail.is_empty() && detail.contains('\n') {
            detail.split('\n').next().unwrap_or(detail)
        } else {
            detail
        };
        inner.lbl_detail.set_text(clean);
    } else {
        inner.lbl_headline.set_text(&i18n::tr("protection_disabled"));
        inner.lbl_slogan.set_text(&i18n::tr("slogan_disabled"));
        inner.lbl_detail.set_text("");
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    let running = *inner.is_running.borrow();
    let detail = inner.current_detail.borrow().clone();
    update_texts(inner, running, &detail);
    inner.feature_icons.retranslate_ui();
    inner.network_section.retranslate_ui();
    inner.https_view.retranslate_ui();
    inner.mode_view.retranslate_ui();
    inner.updates_proxy_view.retranslate_ui();
    inner.allowlist_view.retranslate_ui();
    inner.ev_view.retranslate_ui();
}

#[cfg(test)]
mod tests {
    /// Guards the feedback-loop bug found by running the real GUI: syncing the
    /// switch from a backend status snapshot must NOT be treated as a user click.
    /// Without the `syncing` guard, set_protection_state -> SetChecked -> toggled
    /// -> on_switch_toggled -> backend.set_protection(false) turned the proxy off
    /// by itself (observed on a cold start: the app issued a real `stop`).
    ///
    /// This mirrors the guard logic without needing a GTK main loop.
    struct Guard {
        syncing: std::cell::Cell<bool>,
        commands: std::cell::RefCell<Vec<bool>>,
    }

    impl Guard {
        fn new() -> Guard {
            Guard {
                syncing: std::cell::Cell::new(false),
                commands: std::cell::RefCell::new(Vec::new()),
            }
        }

        /// set_protection_state_animate(): programmatic sync of the switch
        fn sync_from_status(&self, is_running: bool) {
            self.syncing.set(true);
            self.switch_emits_toggled(is_running);
            self.syncing.set(false);
        }

        /// ToggleSwitch::set_checked(): emits toggled only when the value changed
        fn switch_emits_toggled(&self, checked: bool) {
            self.on_switch_toggled(checked);
        }

        /// on_switch_toggled(): a real command is only issued for user input
        fn on_switch_toggled(&self, is_checked: bool) {
            if self.syncing.get() {
                return;
            }
            self.commands.borrow_mut().push(is_checked);
        }
    }

    #[test]
    fn status_sync_does_not_issue_protection_commands() {
        let g = Guard::new();
        g.sync_from_status(false);
        g.sync_from_status(true);
        g.sync_from_status(true);
        assert!(
            g.commands.borrow().is_empty(),
            "status sync must not send start/stop, got {:?}",
            g.commands.borrow()
        );
    }

    #[test]
    fn user_click_does_issue_the_command() {
        let g = Guard::new();
        g.on_switch_toggled(false);
        assert_eq!(&*g.commands.borrow(), &[false]);
    }

    #[test]
    fn cold_start_sequence_matches_python() {
        // Python: initial refresh reports "not running", then the 1500 ms timer
        // calls set_protection(True) once. The status echo must not cancel it.
        let g = Guard::new();
        g.sync_from_status(false); // initial refresh_status()
        g.on_switch_toggled(true); // _auto_start_protection -> set_protection(True)
        g.sync_from_status(true); // status_changed after the proxy came up
        assert_eq!(
            &*g.commands.borrow(),
            &[true],
            "exactly one start command, never a spurious stop"
        );
    }
}
