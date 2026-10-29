// Port of adguard_modern/tabs/apps_tab.py
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n;
use crate::widgets::toggle_switch::ToggleSwitch;

use super::app_item_row::AppItemRow;
use super::apps_config;
use super::apps_group::AppCollapsibleGroup;
use super::apps_menu::AppManagementMenu;
use super::apps_scanner;

#[derive(Clone)]
pub struct AppManagementTab {
    root: gtk4::Box,
    all_app_rows: Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>,
    open_stats_cb: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

impl AppManagementTab {
    pub fn new() -> AppManagementTab {
        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        root.set_margin_start(36);
        root.set_margin_end(36);
        root.set_margin_top(26);
        root.set_margin_bottom(16);
        root.add_css_class("apps-tab-root");

        let all_app_rows: Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>> =
            Rc::new(RefCell::new(Vec::new()));
        let open_stats_cb: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));

        // ---- Header row: "App management" + more menu (⋮) ----
        let header_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        let lbl_header = gtk4::Label::new(None);
        lbl_header.add_css_class("apps-header-22");
        header_layout.append(&lbl_header);

        let btn_more = gtk4::Button::new();
        btn_more.add_css_class("apps-more-btn");
        btn_more.set_size_request(32, 32);
        let m_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        m_layout.set_margin_start(6);
        m_layout.set_margin_end(6);
        m_layout.set_margin_top(6);
        m_layout.set_margin_bottom(6);
        let m_svg = crate::widgets::svg::svg_image("stat_more.svg", 32);
        m_layout.append(&m_svg);
        btn_more.set_child(Some(&m_layout));
        header_layout.append(&btn_more);

        root.append(&header_layout);

        // ---- Master protection inactive warning banner ----
        let banner_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        banner_box.add_css_class("apps-banner");
        banner_box.set_margin_start(12);
        banner_box.set_margin_end(12);
        banner_box.set_margin_top(8);
        banner_box.set_margin_bottom(8);
        let b_icon = crate::widgets::svg::svg_image("badge_shield_off.svg", 20);
        banner_box.append(&b_icon);
        let b_text = gtk4::Label::new(None);
        b_text.add_css_class("apps-banner-text");
        b_text.set_hexpand(true);
        b_text.set_wrap(true);
        banner_box.append(&b_text);
        banner_box.set_visible(false);
        root.append(&banner_box);

        // ---- Subtitle with link ----
        let lbl_sub = gtk4::Label::new(None);
        lbl_sub.add_css_class("apps-subtitle-10");
        lbl_sub.set_wrap(true);
        // apps_subtitle contains an <a href>; the Qt-style style='...' attribute
        // must be stripped for Pango (see home_network_section::to_pango_link_markup).
        lbl_sub.set_use_markup(true);
        root.append(&lbl_sub);

        let spacer6 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer6.set_size_request(-1, 6);
        root.append(&spacer6);

        // ---- Scroll area ----
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.add_css_class("apps-scroll");

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
        container.set_margin_start(0);
        container.set_margin_end(12);
        container.set_margin_top(4);
        container.set_margin_bottom(16);
        scroll.set_child(Some(&container));
        root.append(&scroll);

        // ---- 1. Filter unknown apps card ----
        let filter_unknown_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        filter_unknown_box.set_margin_bottom(6);

        let fu_text_layout = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        let lbl_fu_title = gtk4::Label::new(None);
        lbl_fu_title.add_css_class("apps-fu-title");
        fu_text_layout.append(&lbl_fu_title);
        let lbl_fu_desc = gtk4::Label::new(None);
        lbl_fu_desc.add_css_class("apps-fu-desc-9");
        lbl_fu_desc.set_wrap(true);
        fu_text_layout.append(&lbl_fu_desc);
        filter_unknown_box.append(&fu_text_layout);

        // Read initial state from proxy.yaml
        let mut fu_initial = true;
        for pp in apps_config::proxy_yaml_paths() {
            if let Ok(c) = std::fs::read_to_string(&pp) {
                let re = regex::Regex::new(r#"- name:\s*['\"]?\*['\"]?\s*\n\s*action:\s*['\"]?(\w+)['\"]?"#).unwrap();
                if let Some(m) = re.captures(&c) {
                    if m.get(1).map(|x| x.as_str().to_lowercase()) == Some("bypass".to_string()) {
                        fu_initial = false;
                    }
                }
            }
        }

        let switch_fu = ToggleSwitch::new(46, 26);
        switch_fu.set_checked(fu_initial, false);
        filter_unknown_box.append(switch_fu.root());
        container.append(&filter_unknown_box);

        // ---- 2. Search bar ----
        let search_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        search_box.set_size_request(-1, 40);
        search_box.add_css_class("apps-search-box");
        search_box.set_margin_start(10);
        search_box.set_margin_end(10);

        let search_icon = crate::widgets::svg::svg_image("search.svg", 16);
        search_box.append(&search_icon);

        let search_input = gtk4::Entry::new();
        search_input.add_css_class("apps-search-entry");
        search_input.set_placeholder_text(Some("Search"));
        search_input.set_hexpand(true);
        search_box.append(&search_input);
        container.append(&search_box);

        let spacer6b = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer6b.set_size_request(-1, 6);
        container.append(&spacer6b);

        // ---- 3. Load real applications from Linux system ----
        let (browsers, other_apps, system_apps) = apps_scanner::scan_real_system_apps();

        // Group: Browsers
        let group_browsers = AppCollapsibleGroup::new("Browsers");
        for b in &browsers {
            let row = AppItemRow::new(
                &b.icon,
                &b.name,
                &b.exec,
                b.route,
                b.filter,
                b.https,
                "",
            );
            let rows = all_app_rows.clone();
            let exec_bin = b.exec.clone();
            let name = b.name.clone();
            let is_br = true;
            row.connect_badge_changed(Box::new(move |eb, _badge, _state| {
                Self::handle_badge(&rows, eb);
                let _ = (&exec_bin, &name, is_br);
            }));
            group_browsers.add_widget(row.root());
            all_app_rows.borrow_mut().push((row, b.name.clone(), true, b.exec.clone()));
        }
        container.append(group_browsers.root());

        // Group: Other
        let group_other = AppCollapsibleGroup::new("Other");
        for o in other_apps.iter().take(15) {
            let warn = if o.name.to_lowercase().contains("steam") || o.name.to_lowercase().contains("game") {
                "Filtering may cause issues"
            } else {
                ""
            };
            let row = AppItemRow::new(&o.icon, &o.name, &o.exec, o.route, o.filter, o.https, warn);
            let rows = all_app_rows.clone();
            let exec_bin = o.exec.clone();
            row.connect_badge_changed(Box::new(move |eb, _badge, _state| {
                Self::handle_badge(&rows, eb);
                let _ = &exec_bin;
            }));
            group_other.add_widget(row.root());
            all_app_rows.borrow_mut().push((row, o.name.clone(), false, o.exec.clone()));
        }
        container.append(group_other.root());

        // Group: System apps
        let group_sys = AppCollapsibleGroup::new("System apps");
        for s in system_apps.iter().take(12) {
            let row = AppItemRow::new(&s.icon, &s.name, &s.exec, s.route, s.filter, s.https, "");
            let rows = all_app_rows.clone();
            let exec_bin = s.exec.clone();
            row.connect_badge_changed(Box::new(move |eb, _badge, _state| {
                Self::handle_badge(&rows, eb);
                let _ = &exec_bin;
            }));
            group_sys.add_widget(row.root());
            all_app_rows.borrow_mut().push((row, s.name.clone(), false, s.exec.clone()));
        }
        container.append(group_sys.root());

        // Group: My apps
        let group_my = AppCollapsibleGroup::new("My apps");
        let btn_add_app = gtk4::Button::new();
        btn_add_app.add_css_class("apps-add-app-btn");
        btn_add_app.set_label("+ Add app");
        group_my.add_widget(&btn_add_app);
        container.append(group_my.root());

        // Wire search
        let ar_for_search = all_app_rows.clone();
        search_input.connect_changed(move |entry| {
            let query = entry.text().to_string().trim().to_lowercase();
            for (row, name, _, _) in ar_for_search.borrow().iter() {
                let visible = query.is_empty() || name.to_lowercase().contains(&query);
                row.root().set_visible(visible);
            }
        });

        // Wire filter-unknown switch
        switch_fu.connect_toggled(Box::new(move |val| {
            apps_config::save_filter_unknown_apps(val);
        }));

        // Wire add app
        let ar_for_add = all_app_rows.clone();
        let gm_for_add = group_my.clone();
        btn_add_app.connect_clicked(move |_| {
            let dlg = gtk4::FileChooserDialog::new(
                Some("Select Application Executable or .desktop file"),
                None::<&gtk4::Window>,
                gtk4::FileChooserAction::Open,
                &[("_Cancel", gtk4::ResponseType::Cancel), ("_Open", gtk4::ResponseType::Accept)],
            );
            dlg.set_current_folder(Some(&gtk4::gio::File::for_path("/usr/bin")));
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("Applications"));
            filter.add_pattern("*.desktop");
            filter.add_pattern("*");
            dlg.add_filter(&filter);
            let ar = ar_for_add.clone();
            let gm = gm_for_add.clone();
            dlg.connect_response(move |d, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        let file_path = file.path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                        if !file_path.is_empty() {
                            let name = std::path::Path::new(&file_path)
                                .file_name()
                                .map(|n| n.to_string_lossy().replace(".desktop", ""))
                                .unwrap_or_default();
                            let name = if name.is_empty() {
                                "App".to_string()
                            } else {
                                let mut c = name.chars();
                                match c.next() {
                                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                    None => name,
                                }
                            };
                            let row = AppItemRow::new("apps", &name, &file_path, true, true, true, "");
                            let rows = ar.clone();
                            let eb = file_path.clone();
                            row.connect_badge_changed(Box::new(move |eb2, _b, _s| {
                                Self::handle_badge(&rows, eb2);
                                let _ = &eb;
                            }));
                            let count = {
                                let mut cnt = 0;
                                let mut ch = gm.root().first_child();
                                while let Some(c) = ch {
                                    cnt += 1;
                                    ch = c.next_sibling();
                                }
                                cnt
                            };
                            gm.insert_widget(count - 1, row.root());
                            row.root().set_visible(true);
                            ar.borrow_mut().push((row, name.clone(), false, file_path.clone()));
                        }
                    }
                }
                d.close();
            });
            dlg.show();
        });

        // Wire more menu
        let ar_for_menu = all_app_rows.clone();
        let switch_for_menu = switch_fu.clone();
        let stats_cb = open_stats_cb.clone();
        let banner_for_menu = banner_box.clone();
        btn_more.connect_clicked(move |btn| {
            let ar = ar_for_menu.clone();
            let sw = switch_for_menu.clone();
            let st = stats_cb.clone();
            let bn = banner_for_menu.clone();
            let menu = AppManagementMenu::new(
                Some(btn),
                Some(Box::new(move |action| {
                    Self::handle_menu_action(action, &ar, &sw, &st, &bn);
                })),
            );
            menu.popup();
        });

        let lh = lbl_header.clone();
        let bt = b_text.clone();
        let ls = lbl_sub.clone();
        let lft = lbl_fu_title.clone();
        let lfd = lbl_fu_desc.clone();
        let si = search_input.clone();
        let gb = group_browsers.clone();
        let go = group_other.clone();
        let gs = group_sys.clone();
        let gm = group_my.clone();
        let ba = btn_add_app.clone();
        i18n::connect_language_changed(Box::new(move |_lang| {
            Self::retranslate_ui(&lh, &bt, &ls, &lft, &lfd, &si, &gb, &go, &gs, &gm, &ba);
        }));

        Self::retranslate_ui(
            &lbl_header,
            &b_text,
            &lbl_sub,
            &lbl_fu_title,
            &lbl_fu_desc,
            &search_input,
            &group_browsers,
            &group_other,
            &group_sys,
            &group_my,
            &btn_add_app,
        );

        AppManagementTab {
            root,
            all_app_rows,
            open_stats_cb,
        }
    }

    fn retranslate_ui(
        lbl_header: &gtk4::Label,
        b_text: &gtk4::Label,
        lbl_sub: &gtk4::Label,
        lbl_fu_title: &gtk4::Label,
        lbl_fu_desc: &gtk4::Label,
        search_input: &gtk4::Entry,
        group_browsers: &AppCollapsibleGroup,
        group_other: &AppCollapsibleGroup,
        group_sys: &AppCollapsibleGroup,
        group_my: &AppCollapsibleGroup,
        btn_add_app: &gtk4::Button,
    ) {
        lbl_header.set_text(&i18n::tr("nav_apps"));
        b_text.set_text(&i18n::tr("apps_banner_warning"));
        lbl_sub.set_markup(&crate::tabs::home_network_section::to_pango_link_markup(
            &i18n::tr("apps_subtitle"),
            crate::constants::COLOR_TEXT_MUTED,
        ));
        lbl_fu_title.set_text(&i18n::tr("apps_filter_unknown_title"));
        lbl_fu_desc.set_text(&i18n::tr("apps_filter_unknown_desc"));
        search_input.set_placeholder_text(Some(&i18n::tr("flog_search_placeholder")));
        group_browsers.set_title(&i18n::tr("apps_group_browsers"));
        group_other.set_title(&i18n::tr("apps_group_other"));
        group_sys.set_title(&i18n::tr("apps_group_system"));
        group_my.set_title(&i18n::tr("apps_group_my"));
        btn_add_app.set_label(&i18n::tr("apps_add_app"));
    }

    fn handle_menu_action(
        action: &str,
        rows: &Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>,
        switch_fu: &ToggleSwitch,
        stats_cb: &Rc<RefCell<Option<Box<dyn Fn()>>>>,
        _banner: &gtk4::Box,
    ) {
        match action {
            "route_all" => Self::set_all_route(rows, true),
            "stop_route" => Self::set_all_route(rows, false),
            "filter_all" => Self::set_all_filter(rows, true),
            "stop_filter" => Self::set_all_filter(rows, false),
            "open_stats" => {
                if let Some(cb) = stats_cb.borrow().as_ref() {
                    cb();
                }
            }
            "reset_default" => Self::reset_defaults(rows, switch_fu),
            _ => {}
        }
    }

    fn set_all_route(rows: &Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>, enable: bool) {
        for (row, _, is_br, exec_bin) in rows.borrow().iter() {
            row.set_route(enable);
            apps_config::save_app_action(
                exec_bin,
                if enable { "default" } else { "bypass" },
                *is_br,
                true,
                true,
                true,
            );
        }
    }

    fn set_all_filter(rows: &Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>, enable: bool) {
        for (row, _, is_br, exec_bin) in rows.borrow().iter() {
            row.set_filter(enable);
            apps_config::save_app_action(
                exec_bin,
                if enable { "default" } else { "bypass" },
                *is_br,
                true,
                true,
                true,
            );
        }
    }

    fn reset_defaults(
        rows: &Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>,
        switch_fu: &ToggleSwitch,
    ) {
        // QMessageBox.question(...).exec() is blocking in Qt; GTK4 has no
        // blocking dialog, so the reset runs in the Yes-response handler.
        let dlg = gtk4::MessageDialog::builder()
            .message_type(gtk4::MessageType::Question)
            .buttons(gtk4::ButtonsType::YesNo)
            .text("Are you sure you want to reset all app management settings to default?")
            .build();
        dlg.set_title(Some("Reset to default"));
        let rows = rows.clone();
        let switch_fu = switch_fu.clone();
        dlg.connect_response(move |d, resp| {
            if resp == gtk4::ResponseType::Yes {
                switch_fu.set_checked(true, true);
                for (row, _, is_br, exec_bin) in rows.borrow().iter() {
                    row.set_route(true);
                    row.set_filter(true);
                    row.set_https(*is_br);
                    apps_config::save_app_action(exec_bin, "default", *is_br, true, true, true);
                }
            }
            d.close();
        });
        dlg.show();
    }

    fn handle_badge(rows: &Rc<RefCell<Vec<(AppItemRow, String, bool, String)>>>, exec_bin: &str) {
        let is_br = rows
            .borrow()
            .iter()
            .any(|(_, _, b, e)| *b && e == exec_bin);
        for (row, _, _, e) in rows.borrow().iter() {
            if e == exec_bin {
                let (is_route, is_filter, is_https) = row.get_states();
                let mut action = "default";
                if !is_route || !is_filter {
                    action = "bypass";
                } else if !is_https && is_br {
                    action = "bypass_https";
                }
                apps_config::save_app_action(exec_bin, action, is_br, is_route, is_filter, is_https);
                break;
            }
        }
    }

    pub fn root(&self) -> &gtk4::Box {
        &self.root
    }

    pub fn set_master_protection_state(&self, is_running: bool) {
        self.root.set_visible(true);
        // banner_box visibility toggled via stored child; simplest is to walk.
        // The banner is the second child (index 1) of root.
        if let Some(banner) = self.root.first_child().and_then(|h| h.next_sibling()) {
            banner.set_visible(!is_running);
        }
    }

    pub fn connect_open_statistics_requested(&self, cb: Box<dyn Fn()>) {
        *self.open_stats_cb.borrow_mut() = Some(cb);
    }
}
