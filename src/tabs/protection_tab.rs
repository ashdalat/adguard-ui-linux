// Port of adguard_modern/tabs/protection_tab.py
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::backend;
use crate::i18n;
use crate::tabs::{
    anti_dpi_view, browsing_security_view, create_rule_view, custom_stealth_view,
    dns_filter_editor, dns_filters_view, dns_protection_view, dns_server_detail_view,
    dns_server_select_view, filter_group_view, language_filters_view, protection_data,
    extensions_view,
    protection_detail, protection_rows, stealth_param_view, tracking_protection_view,
    user_rules_view,
};
use crate::tabs::dns_data::DnsServerData;
use crate::tabs::adguard_assistant_dialog::AdGuardAssistantDialog;

type SubItemData = (String, String, String, bool, String, bool);

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

fn sub_items_data(module: &protection_data::ModuleData) -> Vec<SubItemData> {
    module
        .sub_items
        .iter()
        .map(|s| {
            (
                s.key.to_string(),
                s.title.to_string(),
                s.desc.to_string(),
                s.active,
                s.warning.to_string(),
                s.has_page,
            )
        })
        .collect()
}

struct Inner {
    root: gtk4::Box,
    stack: gtk4::Stack,
    lbl_header: gtk4::Label,
    rows: RefCell<HashMap<String, protection_rows::ProtectionItemRow>>,
    modules_data: RefCell<HashMap<String, protection_data::ModuleData>>,
    param_values: RefCell<HashMap<String, String>>,
    dpi_config: RefCell<anti_dpi_view::AntiDpiConfig>,

    detail_view: protection_detail::ProtectionDetailView,
    lang_view: language_filters_view::LanguageFiltersView,
    group_view: filter_group_view::FilterGroupDetailView,
    user_rules_view: user_rules_view::UserRulesView,
    create_rule_view: create_rule_view::CreateRuleView,
    tracking_view: tracking_protection_view::TrackingProtectionView,
    custom_stealth_view: custom_stealth_view::CustomStealthView,
    stealth_param_view: stealth_param_view::StealthParamView,
    anti_dpi_view: anti_dpi_view::AntiDpiView,
    dns_view: dns_protection_view::DnsProtectionView,
    dns_server_view: dns_server_select_view::DnsServerSelectView,
    dns_filters_view: dns_filters_view::DnsFiltersView,
    dns_user_rules_view: user_rules_view::UserRulesView,
    dns_server_detail_view: dns_server_detail_view::DnsServerDetailView,
    browsing_view: browsing_security_view::BrowsingSecurityView,
    extensions_view: extensions_view::ExtensionsView,

    dns_editor_win: RefCell<Option<dns_filter_editor::DnsFilterEditorWindow>>,
    module_toggled_cb: RefCell<Option<Box<dyn Fn(&str, bool)>>>,
}

#[derive(Clone)]
pub struct ProtectionTab {
    inner: Rc<Inner>,
}

impl ProtectionTab {
    pub fn new() -> ProtectionTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.add_css_class("page-bg");

        let stack = gtk4::Stack::new();
        root.append(&stack);

        // --- View 0: Main Protection Modules List ---
        let main_view = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        main_view.add_css_class("transparent-container");
        main_view.set_margin_top(28);
        main_view.set_margin_bottom(20);
        main_view.set_margin_start(36);
        main_view.set_margin_end(36);

        let lbl_header = gtk4::Label::new(Some("Protection"));
        lbl_header.add_css_class("page-title");
        lbl_header.set_halign(gtk4::Align::Start);
        main_view.append(&lbl_header);

        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("transparent-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        container.add_css_class("transparent-container");
        container.set_margin_top(4);
        container.set_margin_bottom(16);
        container.set_margin_end(12);

        // Read real configuration from machine.
        let live_status = protection_data::read_machine_protection_status();
        let mut modules_data: HashMap<String, protection_data::ModuleData> = protection_data::protection_modules()
            .into_iter()
            .map(|m| (m.key.to_string(), m))
            .collect();
        for k in ["ad_blocking", "tracking", "annoyance", "dns", "browsing", "extensions"] {
            if let Some(m) = modules_data.get_mut(k) {
                match k {
                    "ad_blocking" => m.active = live_status.ad_blocking,
                    "tracking" => m.active = live_status.tracking,
                    "annoyance" => m.active = live_status.annoyance,
                    "dns" => m.active = live_status.dns,
                    "browsing" => m.active = live_status.browsing,
                    "extensions" => m.active = live_status.extensions,
                    _ => {}
                }
            }
        }

        let mut param_values: HashMap<String, String> = HashMap::new();
        param_values.insert("third_party_cookies".to_string(), "180".to_string());
        param_values.insert("first_party_cookies".to_string(), "4320".to_string());
        param_values.insert("referer".to_string(), String::new());
        param_values.insert("user_agent".to_string(), String::new());
        param_values.insert("mask_ip".to_string(), "208.54.104.184".to_string());
        param_values.insert("third_party_cookies".to_string(), live_status.params.third_party_cookies.clone());
        param_values.insert("first_party_cookies".to_string(), live_status.params.first_party_cookies.clone());
        param_values.insert("referer".to_string(), live_status.params.referer.clone());
        param_values.insert("user_agent".to_string(), live_status.params.user_agent.clone());
        param_values.insert("mask_ip".to_string(), live_status.params.mask_ip.clone());

        let dpi_config = anti_dpi_view::AntiDpiConfig {
            enabled: live_status.anti_dpi.enabled,
            client_hello: live_status.anti_dpi.client_hello,
            http_split: live_status.anti_dpi.http_split,
            space_juggling: live_status.anti_dpi.space_juggling,
        };

        let rows: HashMap<String, protection_rows::ProtectionItemRow> = HashMap::new();
        stack.add_named(&main_view, Some("main"));

        // Views 1..16.
        let detail_view = protection_detail::ProtectionDetailView::new();
        stack.add_named(detail_view.root(), Some("detail"));

        let lang_view = language_filters_view::LanguageFiltersView::new();
        stack.add_named(lang_view.root(), Some("lang"));

        let group_view = filter_group_view::FilterGroupDetailView::new();
        stack.add_named(group_view.root(), Some("group"));

        let user_rules_view = user_rules_view::UserRulesView::new();
        stack.add_named(user_rules_view.root(), Some("user_rules"));

        let create_rule_view = create_rule_view::CreateRuleView::new();
        stack.add_named(create_rule_view.root(), Some("create_rule"));

        let tracking_view = tracking_protection_view::TrackingProtectionView::new();
        stack.add_named(tracking_view.root(), Some("tracking"));

        let custom_stealth_view = custom_stealth_view::CustomStealthView::new();
        stack.add_named(custom_stealth_view.root(), Some("custom_stealth"));

        let stealth_param_view = stealth_param_view::StealthParamView::new();
        stack.add_named(stealth_param_view.root(), Some("stealth_param"));

        let anti_dpi_view = anti_dpi_view::AntiDpiView::new();
        stack.add_named(anti_dpi_view.root(), Some("anti_dpi"));

        let dns_view = dns_protection_view::DnsProtectionView::new();
        stack.add_named(dns_view.root(), Some("dns"));

        let dns_server_view = dns_server_select_view::DnsServerSelectView::new();
        stack.add_named(dns_server_view.root(), Some("dns_server"));

        let dns_filters_view = dns_filters_view::DnsFiltersView::new();
        stack.add_named(dns_filters_view.root(), Some("dns_filters"));

        let dns_user_file = home_dir().join(".local/share/adguard-cli/dns_user.txt");
        let dns_user_rules_view = user_rules_view::UserRulesView::new_with(
            dns_user_file,
            "DNS user rules",
            "Block, unblock, or redirect domains with your custom DNS rules",
        );
        stack.add_named(dns_user_rules_view.root(), Some("dns_user_rules"));

        let dns_server_detail_view = dns_server_detail_view::DnsServerDetailView::new();
        stack.add_named(dns_server_detail_view.root(), Some("dns_server_detail"));

        let browsing_view = browsing_security_view::BrowsingSecurityView::new();
        stack.add_named(browsing_view.root(), Some("browsing"));

        let extensions_view = extensions_view::ExtensionsView::new();
        stack.add_named(extensions_view.root(), Some("extensions"));

        let inner = Rc::new(Inner {
            root,
            stack,
            lbl_header,
            rows: RefCell::new(rows),
            modules_data: RefCell::new(modules_data),
            param_values: RefCell::new(param_values),
            dpi_config: RefCell::new(dpi_config),
            detail_view,
            lang_view,
            group_view,
            user_rules_view,
            create_rule_view,
            tracking_view,
            custom_stealth_view,
            stealth_param_view,
            anti_dpi_view,
            dns_view,
            dns_server_view,
            dns_filters_view,
            dns_user_rules_view,
            dns_server_detail_view,
            browsing_view,
            extensions_view,
            dns_editor_win: RefCell::new(None),
            module_toggled_cb: RefCell::new(None),
        });

        // Build protection module rows.
        let data_snapshot: Vec<protection_data::ModuleData> = inner.modules_data.borrow().values().cloned().collect();
        for data in data_snapshot {
            let row = protection_rows::ProtectionItemRow::new(
                data.key,
                data.icon,
                data.title,
                data.desc,
                data.active,
                data.has_switch,
            );
            if data.has_switch {
                let i = inner.clone();
                let key = data.key.to_string();
                row.connect_toggled(Box::new(move |_row_key, value| {
                    on_row_toggled(&i, &key, value);
                }));
            }
            let i = inner.clone();
            row.connect_clicked(Box::new(move |key| {
                open_detail_page(&i, key);
            }));
            container.append(row.root());
            inner.rows.borrow_mut().insert(data.key.to_string(), row);
        }

        // Python: main_layout.addWidget(scroll_area) with container inside.
        scroll.set_child(Some(&container));
        main_view.append(&scroll);

        // Wire sub-page signals.
        let i = inner.clone();
        inner.detail_view.connect_back_clicked(Box::new(move || show(&i, "main")));

        let i = inner.clone();
        inner.detail_view.connect_master_toggled(Box::new(move |key, value| {
            on_detail_master_toggled(&i, key, value);
        }));

        let i = inner.clone();
        inner.detail_view.connect_sub_item_toggled(Box::new(move |category, sub_key, checked| {
            on_detail_sub_item_toggled(&i, category, sub_key, checked);
        }));

        let i = inner.clone();
        inner.detail_view.connect_drilldown_requested(Box::new(move |sub_key| {
            on_detail_drilldown(&i, sub_key);
        }));

        let i = inner.clone();
        inner.lang_view.connect_back_clicked(Box::new(move || show(&i, "detail")));
        let i = inner.clone();
        inner.lang_view.connect_master_toggled(Box::new(move |is_active| {
            if let Some(r) = i.detail_view.sub_row("language_ads") {
                r.set_checked(is_active, true);
            }
        }));

        let i = inner.clone();
        inner.group_view.connect_back_clicked(Box::new(move || show(&i, "detail")));
        let i = inner.clone();
        inner.group_view.connect_master_toggled(Box::new(move |is_active| {
            if let Some(r) = i.detail_view.sub_row("basic") {
                r.set_checked(is_active, true);
            }
        }));

        let i = inner.clone();
        inner.user_rules_view.connect_back_clicked(Box::new(move || show(&i, "detail")));
        let i = inner.clone();
        inner.user_rules_view.connect_create_rule_clicked(Box::new(move || show(&i, "create_rule")));
        let i = inner.clone();
        inner.user_rules_view.connect_master_toggled(Box::new(move |is_active| {
            if let Some(r) = i.detail_view.sub_row("user_rules") {
                r.set_checked(is_active, true);
            }
        }));

        let i = inner.clone();
        inner.create_rule_view.connect_back_clicked(Box::new(move || show(&i, "user_rules")));
        let i = inner.clone();
        inner.create_rule_view.connect_rule_created(Box::new(move |_rule_str| {
            i.user_rules_view.refresh_rules();
        }));

        let i = inner.clone();
        inner.tracking_view.connect_back_clicked(Box::new(move || show(&i, "main")));
        let i = inner.clone();
        inner.tracking_view.connect_master_toggled(Box::new(move |is_active| {
            on_tracking_master_toggled(&i, is_active);
        }));
        let i = inner.clone();
        inner.tracking_view.connect_custom_drilldown_requested(Box::new(move || show(&i, "custom_stealth")));

        let i = inner.clone();
        inner.custom_stealth_view.connect_back_clicked(Box::new(move || show(&i, "tracking")));
        let i = inner.clone();
        inner.custom_stealth_view.connect_param_drilldown_requested(Box::new(move |key| {
            open_stealth_param_page(&i, key);
        }));

        let i = inner.clone();
        inner.stealth_param_view.connect_back_clicked(Box::new(move || show(&i, "custom_stealth")));
        let i = inner.clone();
        inner.stealth_param_view.connect_value_saved(Box::new(move |key, is_active, val| {
            on_stealth_param_saved(&i, key, is_active, &val);
        }));

        let i = inner.clone();
        inner.anti_dpi_view.connect_back_clicked(Box::new(move || show(&i, "custom_stealth")));
        let i = inner.clone();
        inner.anti_dpi_view.connect_config_changed(Box::new(move |cfg| {
            on_anti_dpi_changed(&i, cfg);
        }));

        let i = inner.clone();
        inner.dns_view.connect_back_clicked(Box::new(move || show(&i, "main")));
        let i = inner.clone();
        inner.dns_view.connect_master_toggled(Box::new(move |is_active| {
            on_dns_master_toggled(&i, is_active);
        }));
        let i = inner.clone();
        inner.dns_view.connect_dns_server_drilldown(Box::new(move || open_dns_server_page(&i)));
        let i = inner.clone();
        inner.dns_view.connect_dns_filters_drilldown(Box::new(move || open_dns_filters_page(&i)));
        let i = inner.clone();
        inner.dns_view.connect_dns_user_rules_drilldown(Box::new(move || open_dns_user_rules_page(&i)));

        let i = inner.clone();
        inner.dns_server_view.connect_back_clicked(Box::new(move || {
            i.dns_view.refresh_status();
            show(&i, "dns");
        }));
        let i = inner.clone();
        inner.dns_server_view.connect_server_changed(Box::new(move |name, url| {
            i.dns_view.card_dns_server().set_current_server(&name);
            let _ = url;
        }));
        let i = inner.clone();
        inner.dns_server_view.connect_drilldown_requested(Box::new(move |data| {
            open_dns_server_detail_page(&i, data);
        }));

        let i = inner.clone();
        inner.dns_filters_view.connect_back_clicked(Box::new(move || show(&i, "dns")));
        let i = inner.clone();
        inner.dns_filters_view.connect_master_toggled(Box::new(move |val| {
            i.dns_view.row_dns_filters().set_checked(val, false);
        }));

        let i = inner.clone();
        inner.dns_user_rules_view.connect_back_clicked(Box::new(move || show(&i, "dns")));
        let i = inner.clone();
        inner.dns_user_rules_view.connect_create_rule_clicked(Box::new(move || show(&i, "create_rule")));

        let i = inner.clone();
        inner.dns_server_detail_view.connect_back_clicked(Box::new(move || show(&i, "dns_server")));
        let i = inner.clone();
        inner.dns_server_detail_view.connect_server_selected(Box::new(move |name, upstream| {
            on_server_detail_selected(&i, &name, &upstream);
        }));

        let i = inner.clone();
        inner.browsing_view.connect_back_clicked(Box::new(move || show(&i, "main")));
        let i = inner.clone();
        inner.browsing_view.connect_master_toggled(Box::new(move |is_active| {
            on_browsing_master_toggled(&i, is_active);
        }));

        let i = inner.clone();
        inner.extensions_view.connect_back_clicked(Box::new(move || show(&i, "main")));
        let i = inner.clone();
        inner.extensions_view.connect_master_toggled(Box::new(move |is_active| {
            on_extensions_master_toggled(&i, is_active);
        }));

        let i = inner.clone();
        i18n::connect_language_changed(Box::new(move |_lang| retranslate_ui(&i)));

        retranslate_ui(&inner);

        ProtectionTab { inner }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.inner.root
    }

    pub fn connect_module_toggled(&self, cb: Box<dyn Fn(&str, bool)>) {
        *self.inner.module_toggled_cb.borrow_mut() = Some(cb);
    }

    pub fn set_module_state(&self, key: &str, is_active: bool) {
        let row = self.inner.rows.borrow().get(key).cloned();
        if let Some(r) = row {
            r.set_checked(is_active, true);
        }
        if let Some(m) = self.inner.modules_data.borrow_mut().get_mut(key) {
            m.active = is_active;
        }
        if key == "tracking" {
            self.inner.tracking_view.set_active(is_active);
        } else if key == "dns" {
            self.inner.dns_view.set_active(is_active);
        } else if key == "browsing" {
            self.inner.browsing_view.set_active(is_active);
        } else if self.inner.detail_view.current_key() == key {
            self.inner.detail_view.master_switch().set_checked(is_active, true);
        }
    }

    pub fn retranslate_ui(&self) {
        retranslate_ui(&self.inner);
    }

    /// Debug-only: show a named sub-page directly (see window.rs `--open-page`).
    /// Returns true when the page name exists in this tab's stack.
    pub fn debug_show_page(&self, name: &str) -> bool {
        show(&self.inner, name);
        self.inner.stack.visible_child_name().as_deref() == Some(name)
    }
}

fn show(inner: &Rc<Inner>, name: &str) {
    inner.stack.set_visible_child_name(name);
}

fn emit(inner: &Rc<Inner>, key: &str, value: bool) {
    if let Some(cb) = inner.module_toggled_cb.borrow().as_ref() {
        cb(key, value);
    }
}

fn retranslate_ui(inner: &Rc<Inner>) {
    inner.lbl_header.set_text(&i18n::tr("nav_protection"));

    let module_keys_map: [(&str, (&str, &str)); 7] = [
        ("ad_blocking", ("feat_ad_blocking", "feat_ad_blocking_desc")),
        ("tracking", ("feat_tracking", "feat_tracking_desc")),
        ("annoyance", ("feat_annoyance", "feat_annoyance_desc")),
        ("dns", ("feat_dns", "feat_dns_desc")),
        ("browsing", ("feat_browsing", "feat_browsing_desc")),
        ("extensions", ("feat_extensions", "feat_extensions_desc")),
        ("firewall", ("feat_firewall", "feat_firewall_desc")),
    ];
    for (key, (t_k, d_k)) in module_keys_map.iter() {
        if let Some(row) = inner.rows.borrow().get(*key) {
            row.update_text(&i18n::tr(t_k), &i18n::tr(d_k));
        }
    }

    inner.detail_view.retranslate_ui();
    inner.lang_view.retranslate_ui();
    inner.group_view.retranslate_ui();
    inner.user_rules_view.retranslate_ui();
    inner.create_rule_view.retranslate_ui();
    inner.tracking_view.retranslate_ui();
    inner.custom_stealth_view.retranslate_ui();
    inner.stealth_param_view.retranslate_ui();
    inner.anti_dpi_view.retranslate_ui();
    inner.dns_view.retranslate_ui();
    inner.dns_server_view.retranslate_ui();
    inner.dns_filters_view.retranslate_ui();
    inner.dns_user_rules_view.retranslate_ui();
    inner.dns_server_detail_view.retranslate_ui();
    inner.browsing_view.retranslate_ui();
    inner.extensions_view.retranslate_ui();
}

fn open_detail_page(inner: &Rc<Inner>, key: &str) {
    if key == "tracking" {
        let current_state = inner
            .rows
            .borrow()
            .get("tracking")
            .map(|r| r.is_checked())
            .unwrap_or(false);
        inner.tracking_view.set_active(current_state);
        show(inner, "tracking");
        return;
    }

    if key == "dns" {
        let current_state = inner
            .rows
            .borrow()
            .get("dns")
            .map(|r| r.is_checked())
            .unwrap_or(false);
        inner.dns_view.set_active(current_state);
        inner.dns_view.refresh_status();
        show(inner, "dns");
        return;
    }

    if key == "browsing" {
        let current_state = inner
            .rows
            .borrow()
            .get("browsing")
            .map(|r| r.is_checked())
            .unwrap_or(false);
        inner.browsing_view.set_active(current_state);
        inner.browsing_view.load_settings();
        show(inner, "browsing");
        return;
    }

    if key == "extensions" {
        let current_state = inner
            .rows
            .borrow()
            .get("extensions")
            .map(|r| r.is_checked())
            .unwrap_or(false);
        inner.extensions_view.set_active(current_state);
        inner.extensions_view.load_extensions();
        show(inner, "extensions");
        return;
    }

    if key == "assistant" {
        AdGuardAssistantDialog::new().present();
        return;
    }

    if key == "vpn" {
        let vpn_bin = home_dir().join(".local/bin/adguard-vpn");
        let vpn_sh = home_dir().join(".local/share/adguard_vpn_gui/run_vpn.sh");
        if vpn_bin.exists() {
            let _ = std::process::Command::new(vpn_bin).spawn();
        } else if vpn_sh.exists() {
            let _ = std::process::Command::new("bash").arg(vpn_sh).spawn();
        } else if std::process::Command::new("which")
            .arg("adguard-vpn")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let _ = std::process::Command::new("adguard-vpn").spawn();
        } else {
            let _ = std::process::Command::new("xdg-open")
                .arg("https://adguard-vpn.com/")
                .spawn();
        }
        return;
    }

    if key == "annoyance" {
        let active_ids = protection_data::get_active_filter_ids();
        let is_active = protection_data::annoyance_filter_ids_map()
            .iter()
            .any(|(_, fid)| active_ids.contains(fid));

        let sub_items: Vec<SubItemData> = {
            let mut m = inner.modules_data.borrow_mut();
            match m.get_mut("annoyance") {
                Some(module) => {
                    for item in module.sub_items.iter_mut() {
                        let fid = protection_data::annoyance_filter_id(item.key);
                        item.active = fid.map(|f| active_ids.contains(&f)).unwrap_or(false);
                    }
                    module.active = is_active;
                    sub_items_data(module)
                }
                None => Vec::new(),
            }
        };

        if let Some(r) = inner.rows.borrow().get("annoyance").cloned() {
            r.set_checked(is_active, false);
        }

        let title = inner.modules_data.borrow().get("annoyance").map(|m| m.title).unwrap_or("");
        let desc = inner.modules_data.borrow().get("annoyance").map(|m| m.desc).unwrap_or("");
        inner.detail_view.set_module("annoyance", title, desc, is_active, &sub_items);
        show(inner, "detail");
        return;
    }

    let entry = inner.modules_data.borrow().get(key).cloned();
    if let Some(data) = entry {
        let current_state = inner
            .rows
            .borrow()
            .get(key)
            .map(|r| r.is_checked())
            .unwrap_or(false);
        inner.detail_view.set_module(data.key, data.title, data.desc, current_state, &sub_items_data(&data));
        show(inner, "detail");
    }
}

fn open_dns_server_page(inner: &Rc<Inner>) {
    inner.dns_server_view.load_current_upstream();
    show(inner, "dns_server");
}

fn open_dns_server_detail_page(inner: &Rc<Inner>, server_data: &DnsServerData) {
    let upstream = inner.dns_server_view.current_upstream();
    inner.dns_server_detail_view.load_server(server_data, &upstream);
    show(inner, "dns_server_detail");
}

fn on_server_detail_selected(inner: &Rc<Inner>, name: &str, upstream: &str) {
    inner.dns_server_view.update_radio_selection(upstream);
    inner.dns_view.card_dns_server().set_current_server(name);
    show(inner, "dns_server");
}

fn get_dns_editor(inner: &Rc<Inner>) -> dns_filter_editor::DnsFilterEditorWindow {
    let mut slot = inner.dns_editor_win.borrow_mut();
    if slot.is_none() {
        *slot = Some(dns_filter_editor::DnsFilterEditorWindow::new());
    }
    slot.as_ref().unwrap().clone()
}

fn open_dns_filters_page(inner: &Rc<Inner>) {
    let editor = get_dns_editor(inner);
    editor.select_filter("adguard_filter");
    editor.root().present();
}

fn open_dns_user_rules_page(inner: &Rc<Inner>) {
    let editor = get_dns_editor(inner);
    editor.select_filter("user_rules");
    editor.root().present();
}

fn on_dns_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    if let Some(r) = inner.rows.borrow().get("dns").cloned() {
        r.set_checked(is_active, true);
    }
    if let Some(m) = inner.modules_data.borrow_mut().get_mut("dns") {
        m.active = is_active;
    }
    emit(inner, "dns", is_active);
}

fn on_detail_drilldown(inner: &Rc<Inner>, sub_key: &str) {
    if sub_key == "basic" {
        let state = inner
            .detail_view
            .sub_row("basic")
            .map(|r| r.is_checked())
            .unwrap_or(true);
        inner
            .group_view
            .load_group(1, "Basic protection", "Removes most ads from websites", state);
        show(inner, "group");
    } else if sub_key == "language_ads" {
        if let Some(r) = inner.detail_view.sub_row("language_ads") {
            let state = r.is_checked();
            inner.lang_view.master_switch().set_checked(state, false);
        }
        show(inner, "lang");
    } else if sub_key == "user_rules" {
        if let Some(r) = inner.detail_view.sub_row("user_rules") {
            let state = r.is_checked();
            inner.user_rules_view.set_master_checked(state, false);
        }
        inner.user_rules_view.refresh_rules();
        show(inner, "user_rules");
    }
}

fn on_tracking_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    if let Some(r) = inner.rows.borrow().get("tracking").cloned() {
        r.set_checked(is_active, true);
    }
    if let Some(m) = inner.modules_data.borrow_mut().get_mut("tracking") {
        m.active = is_active;
    }
    backend::run_cmd_async(
        &format!("adguard-cli config set stealthmode.enabled {}", if is_active { "true" } else { "false" }),
        Box::new(|_ok, _out| ()),
    );
    emit(inner, "tracking", is_active);
}

fn on_browsing_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    if let Some(r) = inner.rows.borrow().get("browsing").cloned() {
        r.set_checked(is_active, true);
    }
    if let Some(m) = inner.modules_data.borrow_mut().get_mut("browsing") {
        m.active = is_active;
    }
    emit(inner, "browsing", is_active);
}

fn on_extensions_master_toggled(inner: &Rc<Inner>, is_active: bool) {
    if let Some(r) = inner.rows.borrow().get("extensions").cloned() {
        r.set_checked(is_active, true);
    }
    if let Some(m) = inner.modules_data.borrow_mut().get_mut("extensions") {
        m.active = is_active;
    }
    emit(inner, "extensions", is_active);
}

fn on_detail_sub_item_toggled(inner: &Rc<Inner>, category: &str, sub_key: &str, is_checked: bool) {
    if category == "annoyance" {
        if let Some(fid) = protection_data::annoyance_filter_id(sub_key) {
            backend::toggle_filter(fid, is_checked);
        }
        let has_any = inner.detail_view.sub_rows_checked();
        inner.detail_view.master_switch().set_checked(has_any, true);
        if let Some(r) = inner.rows.borrow().get("annoyance").cloned() {
            r.set_checked(has_any, true);
        }
        if let Some(m) = inner.modules_data.borrow_mut().get_mut("annoyance") {
            m.active = has_any;
        }
        emit(inner, "annoyance", has_any);
    }
}

fn on_row_toggled(inner: &Rc<Inner>, key: &str, value: bool) {
    if let Some(m) = inner.modules_data.borrow_mut().get_mut(key) {
        m.active = value;
    }
    if key == "annoyance" {
        for (_, fid) in protection_data::annoyance_filter_ids_map() {
            backend::toggle_filter(fid, value);
        }
    } else if key == "ad_blocking" {
        backend::run_cmd_async(
            &format!("adguard-cli config set ad_blocking_enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "tracking" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "dns" {
        backend::run_cmd_async(
            &format!("adguard-cli config set dns_filtering.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "browsing" {
        backend::run_cmd_async(
            &format!("adguard-cli config set safebrowsing.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    }
    emit(inner, key, value);
}

fn on_detail_master_toggled(inner: &Rc<Inner>, key: &str, value: bool) {
    if let Some(r) = inner.rows.borrow().get(key).cloned() {
        r.set_checked(value, true);
    }
    if let Some(m) = inner.modules_data.borrow_mut().get_mut(key) {
        m.active = value;
    }

    if key == "annoyance" {
        for (sub_key, fid) in protection_data::annoyance_filter_ids_map() {
            if let Some(r) = inner.detail_view.sub_row(sub_key) {
                r.set_checked(value, false);
            }
            backend::toggle_filter(fid, value);
        }
    } else if key == "ad_blocking" {
        backend::run_cmd_async(
            &format!("adguard-cli config set ad_blocking_enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "tracking" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "dns" {
        backend::run_cmd_async(
            &format!("adguard-cli config set dns_filtering.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "browsing" {
        backend::run_cmd_async(
            &format!("adguard-cli config set safebrowsing.enabled {}", if value { "true" } else { "false" }),
            Box::new(|_ok, _out| ()),
        );
    }

    emit(inner, key, value);
}

fn open_stealth_param_page(inner: &Rc<Inner>, key: &str) {
    let row = inner.custom_stealth_view.row(key);
    let is_active = row.map(|r| r.is_checked()).unwrap_or(true);
    let val = inner.param_values.borrow().get(key).cloned().unwrap_or_default();

    if key == "third_party_cookies" {
        inner.stealth_param_view.set_config(
            key,
            "Delete third-party cookies",
            "Prevents advertisers from tracking your behavior across multiple pages by limiting the lifetime of third-party cookies",
            "To block the cookies, set the lifetime to 0",
            "Lifetime, in minutes",
            if val.is_empty() { "180" } else { &val },
            is_active,
            "If you block these cookies completely, you may have to re-log in to some websites or face other cookie-related issues",
            "",
            "",
            "",
        );
    } else if key == "first_party_cookies" {
        inner.stealth_param_view.set_config(
            key,
            "Delete first-party cookies",
            "Prevents websites from remembering your details, such as your login credentials and language preferences, by limiting the lifetime of first-party cookies",
            "To block the cookies, set the lifetime to 0",
            "Lifetime, in minutes",
            if val.is_empty() { "4320" } else { &val },
            is_active,
            "If you block these cookies completely, some websites may break or stop working",
            "",
            "",
            "",
        );
    } else if key == "referer" {
        inner.stealth_param_view.set_config(
            key,
            "Hide Referer from third parties",
            "Prevents third parties from knowing what websites you visit",
            "The HTTP Referer header for requests will be replaced with the one below. To use the default Referer, leave the field blank",
            "Custom Referer",
            &val,
            is_active,
            "",
            "Referer examples ↗",
            "https://adguard.com/kb/general/stealth-mode/#hide-referer",
            "https://example.com",
        );
    } else if key == "user_agent" {
        inner.stealth_param_view.set_config(
            key,
            "Hide User-Agent",
            "Removes identifying information from the User-Agent header",
            "Your User-Agent will be replaced with the below info. To use the default User-Agent, leave the field blank",
            "Custom User-Agent",
            &val,
            is_active,
            "",
            "User-Agent examples ↗",
            "https://adguard.com/kb/general/stealth-mode/#hide-user-agent",
            "User-Agent",
        );
    } else if key == "mask_ip" {
        inner.stealth_param_view.set_config(
            key,
            "Mask IP address",
            "Tricks websites into believing you are a proxy with the IP address below. To use the default IP address, leave the field blank",
            "",
            "Custom IP address",
            if val.is_empty() { "208.54.104.184" } else { &val },
            is_active,
            "",
            "How it works ↗",
            "https://adguard.com/kb/general/stealth-mode/#mask-ip",
            "",
        );
    } else if key == "anti_dpi" {
        let row = inner.custom_stealth_view.row("anti_dpi");
        let is_active = row.map(|r| r.is_checked()).unwrap_or(false);
        let cfg = inner.dpi_config.borrow().clone();
        inner.anti_dpi_view.set_config(is_active, cfg.client_hello, cfg.http_split, cfg.space_juggling);
        show(inner, "anti_dpi");
        return;
    }

    show(inner, "stealth_param");
}

fn on_anti_dpi_changed(inner: &Rc<Inner>, cfg: &anti_dpi_view::AntiDpiConfig) {
    *inner.dpi_config.borrow_mut() = cfg.clone();
    if let Some(r) = inner.custom_stealth_view.row("anti_dpi") {
        r.set_checked(cfg.enabled, false);
    }

    backend::run_cmd_async(
        &format!("adguard-cli config set stealthmode.anti_dpi.enabled {}", if cfg.enabled { "true" } else { "false" }),
        Box::new(|_ok, _out| ()),
    );
    backend::run_cmd_async(
        &format!("adguard-cli config set stealthmode.anti_dpi.client_hello_split_fragment_size {}", cfg.client_hello),
        Box::new(|_ok, _out| ()),
    );
    backend::run_cmd_async(
        &format!("adguard-cli config set stealthmode.anti_dpi.http_split_fragment_size {}", cfg.http_split),
        Box::new(|_ok, _out| ()),
    );
    backend::run_cmd_async(
        &format!("adguard-cli config set stealthmode.anti_dpi.http_space_juggling {}", if cfg.space_juggling { "true" } else { "false" }),
        Box::new(|_ok, _out| ()),
    );
}

fn on_stealth_param_saved(inner: &Rc<Inner>, key: &str, is_active: bool, val: &str) {
    inner.param_values.borrow_mut().insert(key.to_string(), val.to_string());
    inner.custom_stealth_view.update_row_badge(key, is_active, val);

    let active_str = if is_active { "true" } else { "false" };
    if key == "third_party_cookies" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.block_third_party_cookies {}", active_str),
            Box::new(|_ok, _out| ()),
        );
        if !val.is_empty() && val.chars().all(|c| c.is_ascii_digit()) {
            backend::run_cmd_async(
                &format!("adguard-cli config set stealthmode.block_third_party_cookies_min {}", val),
                Box::new(|_ok, _out| ()),
            );
        }
    } else if key == "first_party_cookies" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.block_first_party_cookies {}", active_str),
            Box::new(|_ok, _out| ()),
        );
        if !val.is_empty() && val.chars().all(|c| c.is_ascii_digit()) {
            backend::run_cmd_async(
                &format!("adguard-cli config set stealthmode.block_first_party_cookies_min {}", val),
                Box::new(|_ok, _out| ()),
            );
        }
    } else if key == "referer" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.remove_referrer_from_third_party_requests {}", active_str),
            Box::new(|_ok, _out| ()),
        );
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.custom_referrer '{}'", val),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "user_agent" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.hide_user_agent {}", active_str),
            Box::new(|_ok, _out| ()),
        );
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.custom_user_agent '{}'", val),
            Box::new(|_ok, _out| ()),
        );
    } else if key == "mask_ip" {
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.hide_ip {}", active_str),
            Box::new(|_ok, _out| ()),
        );
        backend::run_cmd_async(
            &format!("adguard-cli config set stealthmode.custom_ip '{}'", val),
            Box::new(|_ok, _out| ()),
        );
    }
}
