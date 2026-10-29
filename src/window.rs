// Port of adguard_modern/main_window.py — window assembly, tab stack, backend
// signal wiring, status timer.
use gtk4::glib;
use gtk4::prelude::*;

use crate::backend;
use crate::constants;
use crate::tabs::{apps_tab, home_tab, protection_tab, settings_tab, statistics_tab, updates_tab};
use crate::widgets::sidebar;

// tab_map order in the Python QStackedWidget
const TAB_ORDER: [&str; 6] = ["home", "protection", "statistics", "apps", "settings", "updates"];

thread_local! {
    static STATUS_HANDLER: std::cell::RefCell<Option<Box<dyn Fn(bool, &str)>>> =
        const { std::cell::RefCell::new(None) };
}

pub fn on_status_changed(_win: &gtk4::ApplicationWindow, running: bool, detail: &str) {
    STATUS_HANDLER.with(|h| {
        if let Some(cb) = h.borrow().as_ref() {
            cb(running, detail);
        }
    });
}

pub fn create(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let win = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("AdGuard")
        .default_width(960)
        .default_height(640)
        .build();
    win.set_size_request(850, 560);

    // Window/taskbar icon. Python uses app.setWindowIcon(QIcon(LOGO_PATH)); GTK4
    // has no per-window icon API, so the panel resolves the icon through the
    // application id: it must match a com.cachyos.adguard-blocker.desktop file
    // installed in ~/.local/share/applications (see packaging/setup_adguard_gui.sh).
    // "security-high" as icon name produced a generic shield in the taskbar.
    gtk4::Window::set_default_icon_name("adguard-blocker");
    let _ = &win;

    let main_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    main_layout.set_margin_start(0);
    main_layout.set_margin_end(0);
    main_layout.set_margin_top(0);
    main_layout.set_margin_bottom(0);
    win.set_child(Some(&main_layout));

    // Backend logic (signals connected through backend::connect_*)
    let sidebar = sidebar::Sidebar::new();
    main_layout.append(sidebar.root());

    let stack = gtk4::Stack::new();
    stack.set_vhomogeneous(false);
    stack.set_hhomogeneous(false);
    main_layout.append(&stack);

    // 1. Home Tab
    let home = home_tab::HomeTab::new();
    // 2. Protection Tab
    let protection = protection_tab::ProtectionTab::new();
    // 3. Statistics Tab
    let statistics = statistics_tab::StatisticsTab::new();
    // 4. App Management Tab
    let apps = apps_tab::AppManagementTab::new();
    // 5. Settings Tab
    let settings = settings_tab::SettingsTab::new();
    // 6. Updates Tab
    let updates = updates_tab::UpdatesTab::new();

    for name in TAB_ORDER {
        let w: &gtk4::Box = match name {
            "home" => home.root(),
            "protection" => protection.root(),
            "statistics" => statistics.root(),
            "apps" => apps.root(),
            "settings" => settings.root(),
            _ => updates.root(),
        };
        stack.add_named(w, Some(name));
    }

    // Sidebar navigation: tab_selected -> stack switch
    {
        let stack2 = stack.clone();
        let sidebar2 = sidebar.clone();
        let stats = statistics.clone();
        let upd = updates.clone();
        sidebar.connect_tab_selected(Box::new(move |tab_id: &str| {
            if TAB_ORDER.contains(&tab_id) {
                stack2.set_visible_child_name(tab_id);
                // Python's _on_tab_switched has no per-tab hooks, but tabs that
                // refresh on show (statistics/updates) are triggered here to keep
                // the same visible behaviour as the Qt version.
                if tab_id == "statistics" {
                    stats.refresh_stats();
                } else if tab_id == "updates" {
                    upd.on_show();
                }
                let _ = &sidebar2;
            }
        }));
    }

    // Home signals
    {
        let prot = protection.clone();
        home.connect_protection_toggled(Box::new(move |enable| {
            backend::set_protection(enable);
        }));
        home.connect_feature_toggled(Box::new(move |feature_key, is_active| {
            // Sync state to Protection Tab switch
            prot.set_module_state(feature_key, is_active);
        }));
        {
            let sidebar3 = sidebar.clone();
            let stack3 = stack.clone();
            home.connect_feature_shortcut_clicked(Box::new(move |_feature_key| {
                // Switch to Protection Tab
                sidebar3.set_active_tab("protection");
                stack3.set_visible_child_name("protection");
            }));
        }
    }

    // Protection module toggled -> sync Home feature icons
    {
        let home2 = home.clone();
        protection.connect_module_toggled(Box::new(move |module_key, is_active| {
            home2.set_feature_state(module_key, is_active);
        }));
    }

    // Apps ⋮ menu -> open Statistics
    {
        let stack2 = stack.clone();
        let sidebar2 = sidebar.clone();
        let stats = statistics.clone();
        apps.connect_open_statistics_requested(Box::new(move || {
            sidebar2.set_active_tab("statistics");
            stack2.set_visible_child_name("statistics");
            stats.refresh_stats();
        }));
    }

    // Settings filtering_mode_changed -> home.refresh_network_states()
    {
        let home2 = home.clone();
        settings.connect_filtering_mode_changed(Box::new(move |_mode| {
            home2.refresh_network_states();
        }));
    }

    // Home browser-proxy / HTTPS toggles -> settings network view refresh
    {
        let settings2 = settings.clone();
        home.connect_browser_proxy_toggled(Box::new(move |_chk| {
            settings2.refresh_network_states();
        }));
        let settings3 = settings.clone();
        home.connect_home_https_toggled(Box::new(move |_chk| {
            settings3.refresh_network_states();
        }));
    }

    // Backend signals
    {
        let home2 = home.clone();
        let sidebar2 = sidebar.clone();
        let apps2 = apps.clone();
        let settings2 = settings.clone();
        let handler: Box<dyn Fn(bool, &str)> = Box::new(move |is_running, detail| {
            home2.set_protection_state(is_running, detail);
            home2.refresh_network_states();
            sidebar2.set_protection_state(is_running);
            apps2.set_master_protection_state(is_running);
            settings2.refresh_network_states();
        });
        STATUS_HANDLER.with(|h| *h.borrow_mut() = Some(handler));
        backend::connect_status_changed(Box::new(move |is_running, detail| {
            STATUS_HANDLER.with(|h| {
                if let Some(cb) = h.borrow().as_ref() {
                    cb(is_running, detail);
                }
            });
        }));
        backend::connect_action_finished(Box::new(move |success, _out| {
            if !success {
                // Revert toggle switch if failed or cancelled
                backend::refresh_status();
            }
        }));
    }

    // Auto refresh timer (every 4 seconds)
    glib::timeout_add_local(std::time::Duration::from_secs(4), || {
        backend::refresh_status();
        glib::ControlFlow::Continue
    });

    // Initial status check
    backend::refresh_status();

    // Auto-enable protection on startup if proxy is not running (singleShot 1500)
    glib::timeout_add_local(std::time::Duration::from_millis(1500), || {
        if !backend::is_running() {
            backend::set_protection(true);
        }
        glib::ControlFlow::Break
    });

    // Start on Home tab by default (debug: --open-tab <home|protection|statistics|apps|settings|updates>)
    let initial_tab = debug_open_tab().unwrap_or_else(|| "home".to_string());
    let initial_tab = if TAB_ORDER.contains(&initial_tab.as_str()) {
        initial_tab
    } else {
        "home".to_string()
    };
    let debug_page = debug_open_page();
    sidebar.set_active_tab(&initial_tab);
    stack.set_visible_child_name(&initial_tab);

    // Debug: dive one level into a sub-page of the selected tab, so every
    // sub-view can be inspected without mouse input. Names are the stack page
    // names used by each tab (e.g. `--open-tab protection --open-page dns`).
    if let Some(page) = debug_page {
        let found = match initial_tab.as_str() {
            "home" => home.debug_show_page(&page),
            "protection" => protection.debug_show_page(&page),
            "settings" => settings.debug_show_page(&page),
            _ => false,
        };
        println!("[debug] --open-page {} on tab {}: {}", page, initial_tab, if found { "shown" } else { "NOT FOUND" });
    }

    win
}

/// Debug flag `--open-tab <id>`: opens the window directly on one tab.
/// Not part of the Python CLI surface; ignored when absent.
fn debug_open_tab() -> Option<String> {
    debug_arg("--open-tab")
}

/// Debug flag `--open-page <name>`: dives into a sub-page of the opened tab.
fn debug_open_page() -> Option<String> {
    debug_arg("--open-page")
}

fn debug_arg(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let idx = args.iter().position(|a| a == flag)?;
    args.get(idx + 1).cloned()
}
