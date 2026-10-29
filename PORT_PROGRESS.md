# Port progress — Python → Rust (file by file)

Paths:
- Python source (canonical): `/home/pc/Desktop/Softlinux/cachyos software/AdGuard_CachyOS_GUI_App`
- Old Rust (untrusted, kept as reference only): `.../adguard-blocker-backup`
- New Rust project: `.../adguard-blocker-rebuilt`

## Architecture decisions (fixed, do not re-analyze)
- GUI framework: GTK4 (same as old port; Python uses PyQt6). Tray via `ksni` (StatusNotifierItem),
  notifications via `gtk4::gio::Notification` (Qt `showMessage` has no SNI equivalent).
- Single instance: Linux abstract unix socket, name `adguard_modern_single_instance_lock`
  (leading NUL implicit in `UnixAddr::new_abstract`) → drop-in compatible with the Python app.
- Event marshalling from worker threads to the GTK main loop: `std::sync::mpsc` + glib poll.
- i18n: translations embedded from `src/i18n_data.json` (535 keys, en+vi, regenerated from
  `adguard_modern/i18n.py` TRANSLATIONS — verified complete).
- Module layout mirrors the Python package: `src/backend.rs`, `src/i18n.rs`, `src/constants.rs`,
  `src/window.rs`, `src/widgets/`, `src/tabs/`.

## Done
1. **run_modern_gui.py** (entry point) → `src/main.rs`
   - Ported: abstract-socket single instance + SHOW wakeup, IPC listener thread, tray icon +
     6-item context menu with live retranslation, status-driven menu enable state, 30s pause timer
     with auto-resume + notifications, show/hide/toggle, `--autostart`/`--tray` start mode,
     app font (Ubuntu 10, antialias), window title "AdGuard", quit-on-last-window-closed disabled.
   - Checks: `cargo check` (pending deps below).
   - Pending deps used by this file: `backend` (set_protection, is_running, status_changed),
     `window` (main window assembly), `i18n` (tr + language_changed signal).
   - Known Python bug noted (not fixed, reported later): none found in this file.

2. **constants.py** → `src/constants.rs` — palette, APP_NAME/APP_VERSION, base/assets/icons/logo paths.
   Checks: `cargo check` pending (needs other modules). No pending deps.

3. **i18n.py** → `src/i18n.rs` — en/vi, settings.json persistence, `tr(key, default)`,
   language_changed subscribers. Data: `src/i18n_data.json` regenerated from the Python
   TRANSLATIONS dict (535 keys, verified complete against source). No pending deps.

4. **backend.py** → `src/backend.rs` — cli path lookup, browser proxy configure/disable,
   certificate install (exact bash command preserved), port check, CommandWorker/StatusWorker/
   FilterToggleWorker as worker-thread + main-loop dispatch, signals status_changed /
   action_finished / filter_toggled, SQLite group filter query, auto language filters,
   https filtering, filtering mode info.
   - Python bug recorded (not fixed in Rust): `re` is never imported in backend.py, so
     `get_auto_language_filters`, `is_https_filtering_enabled` and `get_filtering_mode_info`
     raise NameError and always return their defaults. Rust implements the intended regex
     behaviour — flagged as a deliberate deviation for the final report.
   - `needs_root` in CommandWorker is unused by Python; kept for interface parity.
   - Pending: none.

5. **main_window.py** → `src/window.rs` — window title/icon/size, sidebar + stack of 6 tabs,
   signal wiring (tab switch, home↔protection sync, apps→statistics, settings→home network
   refresh, status update fan-out, action_finished revert, 4s status timer, initial check,
   1.5s auto-start, start on Home, closeEvent hides to tray).
   - Pending deps (interfaces assumed, to be ported): `widgets/sidebar.rs`,
     `tabs/home_tab.rs`, `tabs/protection_tab.rs`, `tabs/statistics_tab.rs`,
     `tabs/apps_tab.rs`, `tabs/settings_tab.rs`, `tabs/updates_tab.rs`, `src/style.rs`.

6. **widgets/sidebar.py** → `src/widgets/sidebar.rs` — fixed 236px sidebar (collapses to 56),
   logo + "adguard" title, 6 exclusive nav buttons with SVG icons, Hide/Expand toggle with
   chevron swap, retranslate via i18n, `tab_selected`/`hide_requested` signals,
   `set_protection_state` is a no-op in Python (kept).
   - Environment note: gdk-pixbuf has no SVG loader here, so icons are rasterized with
     `rsvg-convert` and cached (`src/widgets/svg.rs`); fallback = symbolic icon name.
   - CSS for sidebar added to `src/style.rs` (style.rs grows file by file).

7. **widgets/toggle_switch.py** → `src/widgets/toggle_switch.rs` — cairo-painted switch
   (92x48 default, small variant), 220 ms InOutCubic animation, interpolated track colour,
   muted colours while CLI runs (Qt `isEnabled` == false), checkmark when ON / orange ring
   when OFF on large switches, click toggles, `toggled` signal.
   - Convention for all widgets/tabs: `#[derive(Clone)] struct X { inner: Rc<Inner> }` +
     `pub fn root(&self)`, callbacks stored as `Box<dyn Fn...>`.

8. **widgets/common_controls.py** → `src/widgets/common_controls.rs` — AdGuardBackButton
   (110x28, chevron + label, hover turns label #68BC71, `setText` strips "< "),
   NoWheelComboBox (GTK4 Dropdown is click-only, so wheel-ignoring is already equivalent).

9. **widgets/feature_icons.py** → `src/widgets/feature_icons.rs` — 4 feature icon buttons
   (ad_blocking/tracking/annoyance/dns) with green/gray icon swap, per-feature state map,
   overall_enabled gate, tooltip "title (enabled/disabled)", `feature_toggled` signal,
   retranslate on language change.

10. **widgets/privacy_level_row.py** → `src/widgets/privacy_level_row.rs` — RadioCircle (22x22
    cairo) + row with title/desc/warning, `clicked(level_id)`, `drilldown_requested`,
    `set_checked`, `update_text`. CSS added.

11. **widgets/dns_controls.py** → `src/widgets/dns_controls.rs` — RadioButtonCircle (20x20),
    DnsServerRow (select + drilldown hit area right 50px), dns_feature_icon (cairo drawings),
    dns_feature_row, combo_with_checkmark, AddCustomDnsDialog (get_server_info),
    SelectKnownDnsDialog (search filter, get_selected_server).
    - Deviation: GTK4 DropDown has no checkmark delegate → plain dropdown.
    - Deviation: cairo-rs has no `ellipse()` → bezier approximation helper `ellipse_path`.
    - Pending dep: `tabs/dns_data.rs` (`known_dns_providers()`).

12. **widgets/assistant_modal.py** → `src/widgets/assistant_modal.rs` — detect_system_browsers
    (xdg-settings + which, 6 browsers, default/others, Brave fallback), BrowserRow,
    collapsible "Other browsers", Legacy Assistant toggle (46x26, checked, no animation),
    "More details" opens URL with xdg-open.

13. **widgets/filter_editor_item.py** → `src/widgets/filter_editor_item.rs` — row with styled
    checkbox (white check icon), title/subtitle/tags, `set_active`, `toggled(id, bool)`,
    `selected(item)`, left-click selects.

14. **widgets/filter_editor_window.py** → `src/widgets/filter_editor_window.rs` — 1000x680 window,
    left pane (search + filter list + [+][⚙]), right pane (search + syntax link + rules header +
    mascot empty state + rules list + toolbar [+][−][⚙][✎]), DB grouping with order preserved,
    user filter item id 0, skip -2147483648, add/delete rule, raw rules editor, add custom filter,
    prompt_text/prompt_choice/show_message (QInputDialog/QMessageBox equivalents).

15. **widgets/filtering_log_inspector.py** → `src/widgets/filtering_log_inspector.rs` — 340px panel,
    General/Network/Preview tabs, `closed`, `block_rule_requested`, `set_entry` (status colours,
    block button, stealth visibility, domain parsing, network headers, preview box), retranslate.

16. **widgets/filtering_log_window.py** → `src/widgets/filtering_log_window.rs` — 1180x700 window,
    10 type tabs, search row, export/import/clear actions, 7-column table (ListBox rows with
    per-cell colours), empty state with mascot, scroll position preserved on re-render, live log
    parsing from access.log (root vs user, mtime/size gate, 40KB tail, CONNECT skip, per-domain
    display URL + type + status/rule/filter/source/process logic), 2s refresh timer,
    clear confirmation, export via FileChooserDialog, block rule → user.txt append.
    - Deviations: GTK4 has no per-window icon API (window.rs uses icon name); QFileDialog →
      FileChooserDialog; QMessageBox → small GTK windows.
    - `btn_import` and the `⚙` filter button have no handler in Python → kept as-is.

17. **widgets/stat_modals.py** → `src/widgets/stat_modals.rs` — WhySafeModal (450x360, privacy
    policy link via xdg-open, "Got it"), TurnOffStatsModal (450x210, orange "Turn off" + Cancel),
    ClearStatsModal (460x230, orange "Clear" + Cancel). Qt accept/reject → accepted/rejected
    callbacks. CSS added. All widget files are now ported; `cargo check` is clean except the
    pending `tabs` modules (window.rs imports + `tabs::dns_data` used by dns_controls).

18. **tabs/dns_data.py** → `src/tabs/dns_data.rs` — DNS_SERVERS_DATA (11 servers, order preserved,
    protocols + features) and KNOWN_DNS_PROVIDERS_LIBRARY (13 providers) exposed as
    `dns_servers_data()` and `known_dns_providers()`; `cargo check` clean for this file.

19. **tabs/placeholders.py** → `src/tabs/placeholders.rs` — create_placeholder_tab (centered title
    20pt + subtitle 11pt with expanding spacers) and the placeholder SettingsTab. CSS added.

20. **tabs/protection_data.py** → `src/tabs/protection_data.rs` — PROTECTION_MODULES (8 modules with
    icon/title/desc/active/has_switch/sub_items), ANNOYANCE_FILTER_IDS,
    get_module_translated_data (mod_map + sub_map through i18n), get_active_filter_ids
    (SQLite first, `adguard-cli filters list` regex fallback), read_machine_protection_status
    (proxy.yaml indent-based parser: ad_blocking, stealthmode incl. anti_dpi, dns_filtering,
    safebrowsing, stealth params). `cargo check` clean.

21. **tabs/stat_components.py** → `src/tabs/stat_components.rs` — StatThreeCols (3 columns: value
    label via pango markup so per-instance colours work, title label, 4px colour bar;
    update_values) and StatListRow (icon from icons/companies → icons → letter badge, title,
    ads/trackers/total counts with the Python colour rules, chevron, click → `clicked(title)`,
    update_counts). CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/stats_worker.py` → `src/tabs/stats_worker.rs`.

22. **tabs/stats_worker.py** → `src/tabs/stats_worker.rs` — COMPANY_MAP, get_log_paths (root vs
    user log dir, sorted access.log* glob, readability), format_count, format_bytes, StatsWorker as
    util::spawn_async worker delivering StatsResult (same dict keys) with time filter, sort_by,
    per-line parsing (CONNECT skip, bytes, BLOCKED/ID=/502/403, DNS, target/company resolution,
    Direct IP), bandwidth estimates, top-5 + all_companies, _empty_stats. Deviation: Python compares
    naive local datetimes; Rust compares naive-local seconds via libc::localtime_r gmtoff (same
    semantics). `cargo check` clean except pending tab imports.

## Next file
`adguard_modern/tabs/statistics_tab.py` → `src/tabs/statistics_tab.rs`.

23. **tabs/companies_view.py** → `src/tabs/companies_view.rs` — CompanyRow (icon from
    icons/companies, generic fallback, letter badge, counts with Python colour rules, chevron,
    click → clicked(name)) and CompaniesView (back button, calendar/sort header buttons, title,
    search box + entry with textChanged → filter_list, scrollable rows container,
    set_companies, signals back/time/sort/company). CSS added. `cargo check` clean.

24. **tabs/statistics_tab.py** → `src/tabs/statistics_tab.rs` — StatisticsTab: Stack (main view +
    companies view), header buttons with QMenu-equivalent GTK popovers (time/sort/more menus with
    ✓ marker), three-column sections, DNS row, companies container, View-all link button,
    refresh_stats via stats_worker (worker-running guard), retranslate_ui + language subscription,
    showEvent → connect_map refresh, turn-off/on stats, clear statistics (truncate + sudo -n
    fallback shell), WhySafeModal/TurnOff/ClearStats, FilteringLogDialog with search prefilled
    (added `set_search` to filtering_log_window). Deviation: QMenu.exec(pos) → GTK popover anchored
    to the button. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/home_network_section.py` → `src/tabs/home_network_section.rs`.

25. **tabs/home_network_section.py** → `src/tabs/home_network_section.rs` — Home network cards:
    HTTP proxy switch (yaml regex read/write + `adguard-cli config set http_proxy`), rich-text
    description with the `cert` link → install_certificate + info message, HTTPS filtering card,
    filtering-mode nav row (get_filtering_mode_info), browser routing card, retranslate_ui +
    language subscription, refresh_states. Deviations: subprocess `timeout=` not enforced;
    QMessageBox → `show_message` (made its parent optional because gtk4 0.9 has no widget
    `toplevel()` accessor); pango drops the Qt inline style on the link. CSS added.
    `cargo check` clean.

## Next file
`adguard_modern/tabs/home_tab.py` → `src/tabs/home_tab.rs`.

26. **tabs/settings_https_filtering_view.py** → `src/tabs/settings_https_filtering_view.rs` —
    HTTPS filtering subview: back button, master toggle (yaml regex write), certificate section with
    real date from `openssl x509 -enddate`, Reinstall (3-button dialog: install to system /
    regenerate local cert with `adguard-cli cert` + stdin "no") and Download (save dialog + copy),
    allowlist row with `{count}` exclusions, EV row, retranslate_ui + language subscription.
    Deviations: QMessageBox → GTK windows (no transient parent: gtk4 0.9 has no widget toplevel
    accessor); QFileDialog → FileChooserDialog; subprocess timeout not enforced. CSS added.
    `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_filtering_mode_view.py` → `src/tabs/settings_filtering_mode_view.rs`.

27. **tabs/settings_filtering_mode_view.py** → `src/tabs/settings_filtering_mode_view.rs` —
    Filtering mode view: back button, radio group (manual/auto, GTK CheckButton group + CSS
    indicator), Port entry (editingFinished → connect_activate), Proxy-for-updates card,
    _load_current_mode (yaml regex then `adguard-cli config get proxy_mode`), _save_mode (CLI +
    sudo -n + multiline proxy_mode rewrite + background `sudo -n restart` thread), port change,
    updates-proxy toggle with host guard, retranslate_ui + language subscription. Python
    blockSignals() guards → `syncing` flag. Deviation: editingFinished fires only on Enter
    (GTK4 Entry has no focus-out-finished signal). CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_proxy_updates_view.py` → `src/tabs/settings_proxy_updates_view.rs`.

28. **tabs/settings_proxy_updates_view.py** → `src/tabs/settings_proxy_updates_view.rs` — Proxy-for-updates
    subview: three radio options (system/none/custom, GTK CheckButton group), custom proxy fields
    (type dropdown via common_controls::no_wheel_combo + StringList, server, port, username,
    password), _load_config (DOTALL regexes), _save_config (six regex rewrites + sudo -n / plain CLI
    `config set outbound_proxy.enabled`), custom-box enable/dim on radio toggle, retranslate_ui +
    language subscription. Deviations: editingFinished → Entry activate (Enter only); blockSignals
    guard → `syncing` flag; opacity via `.dimmed` CSS class. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_ev_certificates_view.py` → `src/tabs/settings_ev_certificates_view.rs`.

29. **tabs/settings_ev_certificates_view.py** → `src/tabs/settings_ev_certificates_view.rs` —
    EV certificates subview: back + more (popover menu with Add website), title/subtitle, master
    toggle (filter_ev_certificates rewrite or append), search bar, "+ Add website", empty state,
    add-domain dialog (scheme strip, domain validation, dedupe, info message), retranslate_ui +
    language subscription. Deviation: QDialog.exec() → GTK window with button callbacks; QMenu →
    popover. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_website_allowlist_view.py` → `src/tabs/settings_website_allowlist_view.rs`.

30. **tabs/settings_website_allowlist_view.py** → `src/tabs/settings_website_allowlist_view.rs` —
    Website allowlist subview: real exclusions file read (`~/.local/share/adguard-cli/exclusions.txt`
    + /root), custom-domain set from `# Custom` section, search filter, pagination (8/page, up to 7
    page buttons with `...`), add domain (insert at top, append to both files), delete custom domain
    (rewrite both files), more-menu popover (Add website / Open exclusions file via xdg-open),
    retranslate_ui + language subscription. CSS added. `cargo check` clean (only deprecated
    `hide()` warnings, same as other files).

## Next file
`adguard_modern/tabs/home_tab.py` → `src/tabs/home_tab.rs` (subview deps ported).

31. **tabs/home_tab.py** → `src/tabs/home_tab.rs` — Home tab: internal stack (main scroll page +
    HTTPS / filtering-mode / proxy-updates / allowlist / EV subview pages), feature icons row,
    headline/slogan/detail labels, 96x50 toggle switch, network section, back-navigation between
    nested subviews, `set_protection_state`, `refresh_network_states`, `set_feature_state`,
    retranslate_ui + language subscription. Deviations: Python handler methods
    `feature_shortcut_clicked`, `browser_proxy_toggled`, `home_https_toggled` exist but are never
    connected in main_window.py — kept as callback registries; `animate` kwarg → separate
    `set_protection_state_animate`. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/updates_tab.py` → `src/tabs/updates_tab.rs` (no unported deps).

32. **tabs/updates_tab.py** → `src/tabs/updates_tab.rs` — Updates tab: cairo spinner (25 ms rotation,
    round-cap green arc), CheckUpdateWorker via new `util::spawn_stream` (line-by-line progressive
    item resolution + staggered 200/450/700/950 ms final statuses), four UpdateItemRow
    (spinner/checkmark swap), check button, result label, `on_show()` auto-check, CLI version detect,
    retranslate_ui + language subscription. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_about_view.py` → `src/tabs/settings_about_view.rs`.

33. **tabs/settings_about_view.py** → `src/tabs/settings_about_view.rs` — About subview: real
    version detection (`adguard-cli -v` + regex), app title / CoreLibs / DnsLibs / Scriptlets /
    CLI-info labels, latest-version note, copyright, five green link buttons (webbrowser.open →
    xdg-open), back button. No retranslate_ui in Python (none added). CSS added.
    `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_support_view.py` → `src/tabs/settings_support_view.rs`.

34. **tabs/settings_support_view.py** → `src/tabs/settings_support_view.rs` — Support subview: back
    button, title, five SupportItemRow (icon + title/desc + chevron, left-click opens URL via
    xdg-open and emits `clicked(key)`). No retranslate_ui in Python. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_frag_delay_view.py` → `src/tabs/settings_frag_delay_view.rs`.

35. **tabs/settings_frag_delay_view.py** → `src/tabs/settings_frag_delay_view.rs` — Fragmentation
    delay subview: back/title/subtitle/input label, delay entry (digits-only, rewrite or append
    `split_delay_ms` in home + /root proxy.yaml), `delay_changed` signal, load_delay default 20,
    retranslate_ui + language subscription. Deviations: QLineEdit clear button (no GTK4 equivalent)
    omitted; blockSignals → `syncing` flag. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_bootstrap_view.py` → `src/tabs/settings_bootstrap_view.rs`.

36. **tabs/settings_bootstrap_view.py** → `src/tabs/settings_bootstrap_view.rs` — Bootstrap
    addresses subview: back/title/subtitle/note/input label, multi-line TextView (buffer-changed →
    rewrite `bootstraps` in proxy.yaml, join addresses or `default`), load default "default",
    `bootstraps_changed`, retranslate_ui + language subscription. Deviation: GTK4 TextView has no
    placeholder text. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_secure_dns_view.py` → `src/tabs/settings_secure_dns_view.rs`.

37. **tabs/settings_secure_dns_view.py** → `src/tabs/settings_secure_dns_view.rs` — Filter secure
    DNS subview: back, title + master switch, subtitle, dropdown frame (header + option panel with
    green checkmarks), SecureDnsOptionRow click → select mode, toggle expand/collapse with `^`/`v`
    arrow, load `filter_secure_dns_mode` (case-insensitive), write YAML (rewrite or append),
    enabled-state opacity via `.dns-dimmed`, `set_mode`, `mode_changed`, retranslate_ui +
    subscription. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_fallback_dns_view.py` → `src/tabs/settings_fallback_dns_view.rs`.

38. **tabs/settings_fallback_dns_view.py** → `src/tabs/settings_fallback_dns_view.rs` — Fallback DNS
    subview: scroll area, title + master switch, combo (system/custom), custom text area,
    timed-out section (switch + timeout entry), DNS exclusions section (switch + default exclusions
    text area), `fallbacks` YAML rewrite/append, `fallbacks_changed`, retranslate_ui (combo rebuilt
    with translations keeping index) + subscription. Deviations: TextView placeholder and QLineEdit
    clear button omitted; sw_timeout/sw_ex have no handlers in Python (preserved). CSS added.
    `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_network_view.py` → `src/tabs/settings_network_view.rs`.

39. **tabs/settings_network_view.py** → `src/tabs/settings_network_view.rs` — Network subview: back,
    title, scroll area, HTTP proxy section (switch + port entry + cert link label → message box),
    Proxy-for-updates card (click → open subview, host guard refuses localhost/127.0.0.1 and
    redirects, `sudo -n` + plain CLI `config set outbound_proxy.enabled`), `refresh_states()` is
    `pass` in Python (kept as no-op), `_create_nav_row` and `_on_https_toggled` exist but are never
    called in Python (ported as dead-code-marked functions). Deviations: editingFinished → Enter,
    subprocess timeout not enforced, linkActivated consumed (Propagation::Stop). CSS added.
    `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_general_view.py` → `src/tabs/settings_general_view.rs`.

40. **tabs/settings_general_view.py** → `src/tabs/settings_general_view.rs` — App settings subview:
    back + more popover (Reset settings with Yes/No window, Export/Import via FileChooserDialog +
    `adguard-cli export-settings/import-settings`), autostart toggle (XDG .desktop file create/
    rewrite + `systemctl --user enable/disable adguard.service`), usage-data toggle with privacy
    link, four dropdowns (language → i18n set_language, theme, update_channel, log_level) with
    `set_cli_config` (CLI + direct multiline YAML rewrite/append), nav rows for Filter editor
    (FilterEditorWindow) and Advanced, retranslate_ui + language subscription. Deviations:
    QDialog.exec → non-modal window, QMessageBox.question → Yes/No window, pango drops inline link
    style. CSS added. `cargo check` clean.

## Next file
`adguard_modern/tabs/settings_advanced_view.py` → `src/tabs/settings_advanced_view.rs`.

41. **tabs/settings_advanced_view.py** → `src/tabs/settings_advanced_view.rs` — Advanced settings
    subview: back + more popover (red Reset-to-default item → Yes/No window with red Reset button),
    amber warning, search box filtering rows/categories, AdvancedSettingRow (title/desc/extra info,
    clickable variant), 12 switches writing proxy.yaml keys, 4 clickable rows opening frag-delay /
    fallback-DNS / bootstrap / secure-DNS subviews, `update_frag_delay`/`update_fallbacks`/
    `update_bootstraps`/`update_secure_dns`, factory-defaults rewrite of 15 keys, retranslate_ui +
    subscription. `_open_yaml` exists in Python but is never called (ported as dead-code-marked).
    CSS added. `cargo check` clean.

42. **tabs/settings_license_view.py** → `src/tabs/settings_license_view.rs` — License subview:
    back + more popover (Refresh license / Manage license via xdg-open / Reset license with Yes/No
    window + `adguard-cli reset-license`), three stacked pages (active info / not-active invite /
    activating), `_parse_license`, `is_activated`, activate worker: `libc::openpty` + child on the
    slave fd, poll(300 ms) loop, ANSI strip, auto-Enter on "How do you want", URL regex +
    xdg-open, speed-up regex, countdown regex; signals url_ready/countdown/speed_up via
    `util::spawn_stream` items, finished as result; send_key/cancel via control channel.
    Deviations: subprocess timeouts not enforced; send_key latency ≤ 0.3 s (channel vs direct
    write); `wait(timeout=5)` approximated by try_wait loop; webbrowser.open → xdg-open.
    CSS added. `cargo check` clean (only pending window.rs imports remain).

43. **tabs/settings_tab.py** → `src/tabs/settings_tab.rs` — Settings tab: menu root view with 4
    SettingsItemRow (icon + title/desc + chevron, click → navigate), 16-page stack wiring all
    subviews (back navigation to parent, nested-open wiring with load_* + navigate, change-signal
    forwarding frag/fallback/bootstrap/secure_dns → advanced), `filtering_mode_changed` registry,
    `refresh_network_states` (interface assumed by window.rs = network_view.refresh_states()),
    retranslate_ui (header + rows + app_view only, as in Python) + language subscription,
    navigate_to_* kept as dead-code-marked. Added `pub fn load_live_license` to license view.
    Deviation: Python `hasattr` guards → fields always present. CSS added. `cargo check` clean
    (only pending window.rs imports remain).

44. **tabs/protection_rows.py** → `src/tabs/protection_rows.rs` — ProtectionItemRow (icon 24,
    title/desc, divider + switch when has_switch, `›` chevron label otherwise, toggled(key,bool) +
    clicked(key) registries, left-click gesture) and SubFilterRow (title/desc/warning shown when
    OFF, divider only when has_subpage, switch, set_checked/is_checked/update_text, click only when
    has_subpage). Deviation: Python per-row stylesheet hover only when has_subpage → separate CSS
    class. CSS added. `cargo check` clean (only pending window.rs imports remain).

45. **tabs/protection_detail.py** → `src/tabs/protection_detail.rs` — Protection module detail
    subview: back button + options popover (Filter editor / Filtering log), title + master switch,
    description, amber warning, scrollable SubFilterRow container, set_module (rebuild rows,
    translation map, initial-off when master off), master/sub toggle logic with `syncing` guard
    (save/restore states, first-item fallback), drilldown registry, retranslate_ui + subscription.
    Deviation: QDialog.exec → non-modal window (no parent). CSS added. `cargo check` clean (only
    pending window.rs imports remain).

46. **tabs/language_filters_view.py** → `src/tabs/language_filters_view.rs` — Language filters
    drill-down: back, title + master switch, subtitle, auto-activate row (backend
    get/set_auto_language_filters), search box filtering rows by name/id, live SQLite rows
    (query_language_filters_db) with per-row toggle → backend::toggle_filter, master off saves and
    disables enabled fids / restores on, retranslate_ui + subscription ({time} placeholder
    substituted manually). Deviation: QLineEdit placeholder → GTK Entry Some(&str). CSS added.
    `cargo check` clean (only pending window.rs imports remain).

47. **tabs/filter_group_view.py** → `src/tabs/filter_group_view.rs` — Filter group detail subview:
    FilterItemRow (same shape as LanguageItemRow), back, title + master switch, subtitle, section
    header, scrollable rows from query_group_filters_db, load_group (rebuild rows), master toggle
    sets all rows + backend::toggle_filter each, retranslate_ui (title/desc only for group 1) +
    subscription. CSS added. `cargo check` clean (only pending window.rs imports remain).

## Next file
`adguard_modern/tabs/user_rules_view.py` → `src/tabs/user_rules_view.rs` (deps: toggle_switch ✓).

48. **tabs/dns_filter_editor.py** → `src/tabs/dns_filter_editor.rs` — DNS Filter Editor window:
    MascotWidget (cairo DrawingArea 140x140, rounded rects/ellipse/checkmark via bezier),
    AddRuleSimpleDialog (460x220 window, entry + Save/Cancel → accepted/rejected callbacks),
    split-pane window: left filter rows (User rules + AdGuard DNS filter, check chip, name,
    count, #trusted tag), right rules ListBox with search filtering, +/⚙ toolbar buttons,
    empty-state label, rule count labels, SQLite query for AdGuard filter version/update time,
    add/edit/delete/import rules, save_rules_to_file with "! DNS User Rules" header,
    closeEvent → closed callback. Deviations: QSplitter → gtk4::Paned, frameless translucent
    QDialog → regular window, strftime %-m/%-d via libc::localtime, QFileDialog →
    FileChooserDialog (deprecated ctor warning). CSS added. `cargo check` clean (only pending
    window.rs imports remain).

49. **tabs/user_rules_view.py** → `src/tabs/user_rules_view.rs` — User rules view: UserRuleItemRow
    (monospace selectable label, ✕ delete button → delete_requested rewriting the rules file),
    back + more popover (Rule syntax → xdg-open KB URL, AdGuard Assistant → assistant modal),
    title + master switch, subtitle, search filtering rows, editor card with "Go to editor"
    (opens DnsFilterEditorWindow once, closed → refresh_rules), scroll rules list hidden
    initially, refresh_rules reads ~/.local/share/adguard-cli/user.txt skipping !/# lines,
    retranslate_ui with dns-vs-user title/desc switch. Interfaces for protection_tab:
    connect_back_clicked/create_rule_clicked (never emitted in Python, kept for parity)/
    master_toggled, set_master_checked, refresh_rules. Deviation: modal.exec() → non-modal
    present(). CSS added. `cargo check` clean (only pending window.rs imports remain).

50. **tabs/create_rule_view.py** → `src/tabs/create_rule_view.rs` — Create rule form: back + more
    popover (same as user_rules_view), title, rule-type/content/scope dropdowns (no_wheel_combo),
    domain entry, high-priority checkbox, Create/Cancel buttons; rule string built from type index +
    content modifiers + important, appended to user.txt, fields reset (blockSignals → syncing
    guard), rule_created + back_clicked registries, retranslate_ui rebuilds type combo keeping
    index. Deviation: content combo compares English literals in Python (never retranslated) →
    index mapping. CSS added (incl. missing .combo-no-wheel rule). `cargo check` clean (only
    pending window.rs imports remain).

51. **tabs/tracking_protection_view.py** → `src/tabs/tracking_protection_view.rs` — Tracking
    protection sub-page: '‹ Back', title + master switch, subtitle, How-it-works link (xdg-open),
    4 PrivacyLevelRows (standard/high/extreme/custom with drilldown), select_level runs the exact
    adguard-cli stealthmode command chains via backend::run_cmd_async, master toggle dim/enable rows
    (opacity) + backend command, set_active with syncing guard, retranslate_ui + subscription.
    Deviation: run_cmd_async needs explicit no-op callback (Python callback optional). CSS added.
    `cargo check` clean (only pending window.rs imports remain).

52. **tabs/custom_stealth_view.py** → `src/tabs/custom_stealth_view.rs` — Custom Stealth view:
    StealthSettingRow (title/desc/param badge/warning, divider+chevron drilldown rows with
    left-click gesture, switch), 4 sections (General/Tracking methods/Browser API/Miscellaneous)
    with 16 rows, ⋮ reset popover (standard/high/extreme presets → exact adguard-cli command
    chains incl. third_party_cookies_min), per-row toggle → cli_map commands, update_row_badge +
    badge retranslation with {val} substitution, back/popover, retranslate_ui + subscription.
    Deviation: QMenu on button → popover. CSS added. `cargo check` clean (only pending window.rs
    imports remain).

53. **tabs/stealth_param_view.py** → `src/tabs/stealth_param_view.rs` — Reusable stealth param
    editor: back, title + master switch, desc, hint, optional warning label, optional external link
    (single closure reading stored URL instead of disconnect/reconnect), field label, input box with
    ✕ clear button, set_config(key,...) with syncing guard, value_saved(key, bool, String) +
    back registries, per-key retranslate (cookies/referer/ua/mask_ip). Deviation: dim style →
    CSS class swap. CSS added. `cargo check` clean (only pending window.rs imports remain).

54. **tabs/anti_dpi_view.py** → `src/tabs/anti_dpi_view.rs` — Anti-DPI detail editor: master
    toggle, description, two int input boxes (ClientHello/HTTP fragment, default 1) with ✕ clear,
    HTTP space juggling switch + warning, set_config with syncing guard, config_changed emits
    AntiDpiConfig struct (Python dict), retranslate_ui + subscription. Deviation: QIntValidator(1,
    1500) approximated by digit-stripping on change (no clamping). CSS added. `cargo check` clean
    (only pending window.rs imports remain).

55. **tabs/dns_protection_view.py** → `src/tabs/dns_protection_view.rs` — DNS protection view:
    DnsServerCard (title/desc/current-server label with {name}, chevron, click gesture),
    DnsToggleRow (title/desc/divider/switch, toggled + clicked), back + options popover (Filtering
    log / Filter editor / Assistant), What-is-DNS green link, master toggle → backend command +
    proxy.yaml rewrite (user + /root paths), refresh_status parses proxy.yaml dns_filtering block
    (upstream/enabled regex) + name_map display, filters toggle → dns filters enable/disable 1,
    user-rules toggle is an intentional no-op (Python pass). CSS added. `cargo check` clean (only
    pending window.rs imports remain).

56. **tabs/dns_server_select_view.py** → `src/tabs/dns_server_select_view.rs` — DNS server
    selection page: BackButton, title/subtitle, System DNS standalone row + 3 groups (AdGuard/Public/
    Custom) from dns_servers_data, custom container, Add-custom / Select-from-list buttons,
    load_current_upstream proxy.yaml parse, radio selection incl. dynamic Custom row, server_changed +
    drilldown_requested(&DnsServerData) registries, _save_upstream rewrites proxy.yaml (user + /root)
    + backend command. Interface extensions to dns_controls.rs: DnsServerRow stores upstream/labels
    (accessors), AddCustomDnsDialog got connect_accepted (exec()==Accepted → non-modal + callback).
    CSS added. `cargo check` clean (only pending window.rs imports remain).

57. **tabs/dns_server_detail_view.py** → `src/tabs/dns_server_detail_view.rs` — DNS server detail
    sub-page: BackButton, title/subtitle, Official-website link, protocol combo (blockSignals →
    syncing guard; item data kept as parallel Vec since ProtocolCheckmarkDelegate/deviation),
    Features header + dns_feature_row list, Select-this-server button with selected/disabled state
    (currentData() == current upstream), proxy.yaml upstream rewrite + backend command,
    server_selected(name, upstream) registry, load_server(&DnsServerData, current_upstream).
    Interface extension: dns_data structs got #[derive(Clone)]. CSS added. `cargo check` clean
    (only pending window.rs imports remain).

58. **tabs/dns_filters_view.py** → `src/tabs/dns_filters_view.rs` — DNS filters page:
    DnsFilterItemRow (name + ID label, 44x24 switch), back, title + master, subtitle, search box
    filtering rows, "+ Open DNS filter editor" (editor created once, closed → load_filters),
    load_filters from agflm_dns.db (filter_id > 0 ORDER BY is_enabled DESC, title ASC) + master
    reflects has_enabled, per-row toggle → dns filters enable/disable + master update, master off
    disables checked rows / on enables filter 1. CSS added. `cargo check` clean (only pending
    window.rs imports remain).

59. **tabs/browsing_security_view.py** → `src/tabs/browsing_security_view.rs` — Browsing security
    sub-page: back, title + master, subtitle, 3 BrowsingOptionRows (notify/sound/stats with Privacy
    policy link), load_settings parses proxy.yaml safebrowsing block (case-insensitive enabled +
    send_anonymous_statistics), set_active + opacity dim, _save_yaml_setting rewrites the key inside
    the safebrowsing block (insert-if-missing), backend commands, registries + i18n subscription.
    CSS added. `cargo check` clean (only pending window.rs imports remain).

60. **tabs/extensions_view.py** → `src/tabs/extensions_view.rs` — Extensions/Userscripts view: back +
    header ⋮ popover (check-update via spawn + language-specific message, reset Yes/No), title + master
    switch, description + Learn more link, auto-detect card, section header + Add extension, card list.
    load_extensions: CLI `userscripts list` parse ([x]/ID regex), scan ~/.local/share/adguard-cli/userscripts
    *.meta.json (serde_json, mtime strftime "%B %d, %Y at %H:%M" via libc::localtime), 4 known defaults
    appended if missing. ExtensionCard: ✓/off checkbox button (CSS class swap), title/type/detail labels,
    ⋮ popover (Homepage/Reinstall/Edit/Storage/sep/Remove) → view handler (xdg-open, messages, Yes/No
    remove + file delete + reload, editor window map, storage dialog non-modal). AddExtensionDialog: URL
    entry + Browse (FileChooserDialog, deprecated API), Install (CLI install or local copy + meta.json),
    connect_accepted → load_extensions. UserscriptEditorWindow: monospace TextView, default template,
    Save writes js_path. CSS added. `cargo check` clean (only pending window.rs imports remain).

61. **tabs/protection_tab.py** → `src/tabs/protection_tab.rs` — Protection tab controller: gtk4::Stack
    with 17 named pages (main list + 16 sub-views), module rows built from protection_data (live status
    applied), param_values + dpi_config defaults + live overrides, all back/master/drilldown signals wired
    per Python, handlers (row_toggled, detail_master, sub_item_toggled, stealth param saved, anti_dpi
    changed, dns server/filters/user-rules navigation, server detail selected), set_module_state +
    connect_module_toggled (window.rs interface), retranslate. Accessors added to ported modules
    (ProtectionItemRow set_checked/is_checked + Clone, ProtectionDetailView sub_row/master_switch/
    current_key/sub_rows_checked, master_switch/row/load_current_upstream/current_upstream/
    update_radio_selection/row_dns_filters/load_settings/load_extensions/select_filter, Clone derives on
    all views, annoyance_filter_id mapping, window.rs connect_module_toggled Box fix). Assistant branch
    uses ported AdGuardAssistantModal as minimal interface (adguard_assistant_dialog.py pending, needs
    apps_scanner.py). `cargo check` clean (only pending window.rs apps_tab import remains).

62. **tabs/apps_config.py** → `src/tabs/apps_config.rs` — browser/proxy yaml path helpers,
    get_app_action_from_config (regex per exec_name), save_app_action (replace-or-append rule,
    `rfind("- name: '*'")` insert branch, adguard-cli restart via bash -c), save_filter_unknown_apps.
    `cargo check` clean.

63. **tabs/apps_scanner.py** → `src/tabs/apps_scanner.rs` — scan_real_system_apps: 3 .desktop search
    dirs sorted, Name/Icon/Exec/Categories/NoDisplay parsing, BROWSER_KEYS match (anime/proton
    exclusion), action from apps_config, route/filter/https flags, browsers/other/system buckets by
    category. AppEntry struct. `cargo check` clean.

64. **tabs/app_item_row.py** → `src/tabs/app_item_row.rs` — AppBadgeButton (route/filter/https,
    18x18, SVG icon via svg_texture + tooltip, click toggles + emits) and AppItemRow (26x26 resolved
    icon: theme lookup via IconTheme::for_display/has_icon, company SVGs, generic apps.svg fallback;
    name label, optional warning label, 3 badges, chevron; GestureClick emits clicked; get_states).
    Deviation: Python's explicit QPushButton-child exclusion relies on GTK4 child buttons consuming
    presses (observable behaviour identical). CSS added. `cargo check` clean.

65. **tabs/apps_group.py** → `src/tabs/apps_group.rs` — AppCollapsibleGroup: header
    button (arrow ^.→ + title label), collapsible content box, set_title,
    toggle_collapse, add_widget, insert_widget (gtk4 Box has no positional insert;
    walks child list via first_child/next_sibling and uses insert_child_after/prepend).
    #[derive(Clone)] added; add_widget/insert_widget take &impl IsA<Widget>. CSS:
    .app-group-header, .app-group-arrow. `cargo check` clean.

66. **tabs/apps_menu.py** → `src/tabs/apps_menu.rs` — AppManagementMenu: QMenu →
    gtk4::Popover with .menu-popover/.menu-item/.menu-separator CSS; route_all,
    stop_route, filter_all, stop_filter, open_stats, reset_default actions wired via
    button click handlers to on_action(&str) callback; popup(). `cargo check` clean.

67. **tabs/apps_tab.py** → `src/tabs/apps_tab.rs` — AppManagementTab: header +
    more-menu (⋮) popover, master-protection warning banner, subtitle with link,
    scroll area, filter-unknown-apps card (ToggleSwitch initial state read from
    proxy.yaml regex), search bar (filters all_app_rows by name), 4 collapsible
    groups (Browsers / Other[:15] / System apps[:12] / My apps) populated from
    scan_real_system_apps, "+ Add app" (FileChooserDialog → custom AppItemRow
    inserted before add-button), badge_changed → save_app_action with action
    derivation (default/bypass/bypass_https), menu actions (route_all/stop_route/
    filter_all/stop_filter/reset_default via non-blocking MessageDialog Yes/No,
    open_stats signal), set_master_protection_state, connect_open_statistics_requested,
    retranslate via i18n::connect_language_changed. Deviations: QMessageBox.question
    blocking exec() → non-blocking MessageDialog response handler (GTK4 has no
    blocking run()); QFileDialog → gtk4::FileChooserDialog. CSS added (apps-* classes).
    Accessors added to app_item_row (app_name/exec_bin/set_route/set_filter/set_https).
    #[derive(Clone)] on AppManagementTab + AppCollapsibleGroup. window.rs:
    connect_open_statistics_requested Box::new fix, settings2/3 clone fix.
    `cargo check` clean (whole project now compiles).

## Next file
`adguard_modern/tabs/adguard_assistant_dialog.py` → `src/tabs/adguard_assistant_dialog.rs`
(deps: apps_scanner ✓, assistant_modal widget ✓).

68. **tabs/adguard_assistant_dialog.py** → `src/tabs/adguard_assistant_dialog.rs` — AdGuard Assistant
    config dialog: frameless 520x520 window, #242427 card (border rgba(255,255,255,0.08), radius 12),
    header title-22 + ✕ close (24x24, hover), description + "More details" green link
    (xdg-open https://adguard.com/kb/general/assistant/), real browser detection via
    `scan_real_system_apps` with dedup by name, default browser from
    `xdg-settings get default-web-browser` (exec-substring or basename(file) match, fallback = first),
    default-browser row (icon/subtitle "Default browser"/status "Installed"), collapsible
    "Other browsers" section (⌃/⌄ label swap, default expanded), bottom stretch, Legacy Assistant card
    (#1E1E21, radius 10) with 46x26 ToggleSwitch default ON → `adguard-cli userscripts
    enable|disable adguard-assistant` (~/.local/bin/adguard-cli fallback to PATH).
    `_resolve_browser_icon`: icon theme → app_brave/app_fire/app_chrome/app_edge SVG → app_browser.svg,
    26x26. CSS added (.assistant-dialog-root/-card/-title-22/-close-btn/-desc-13/-more-btn/
    -other-toggle/-other-container/-browser-row/-browser-name/-browser-sub/-browser-status/
    -legacy-card/-legacy-title/-legacy-sub). Renamed the container class to `assistant-dialog-card`
    to avoid clashing with widgets/assistant_modal.py's `.assistant-card`. Frameless dialog kept
    non-modal at Construction time; protection_tab calls `present()`.
    `cargo check` clean.

69. **filter_db.py** → `src/filter_db.rs` — (backfilled record; file was written earlier without a
    progress entry). `db_path`/`user_rules_path`, `get_real_installed_filters` (agflm_standard.db
    query with vi/en COALESCE localisation, custom-group name, rules_list count, is_installed=1,
    ORDER BY display_number, tags: trusted + recommended for ids [2,11,3,18,19,214,1]),
    `count_user_rules`, `load_user_rules`, `save_user_rules` ("! User filter" header).
    Fix: last_update_time is now formatted with `libc::localtime_r` (was UTC civil-from-days),
    matching Python's `datetime.fromtimestamp(...).strftime("%d/%m/%Y %H:%M")`.
    `cargo check` clean.

## Status
All 69 Python files (excluding `__init__.py` and `legacy_archive/`) now have a Rust counterpart.

## Packaging (non-.py files from the Python project)
- `adguard_blocker_icon.png`, `i18n_export.json` and the whole `assets/` tree (60 icons) were copied
  from the Python source; the Rust project had no assets before, which made every icon lookup fail.
- `setup_adguard_gui.sh`, `uninstall_adguard_gui.sh`, `setup_cachyos_gui.sh`,
  `start_adguard_service.sh` → `packaging/` (same steps, paths, sudoers entry, NSS/CA policy files,
  systemd unit and autostart .desktop as the Python installers; only the launched target changed from
  `run_modern_gui.py` to the release binary). `setup_all_adguard.sh`, `setup_vpn_gui.sh`,
  `setup_bypass.sh`, `remove_bypass.sh` belong to the separate AdGuard VPN app, not this blocker.

## Debug/verification flags added (not part of the Python CLI surface)
- `--selftest` — prints whether each UI backend entry point really reaches adguard-cli
  (binary path, `--version`, `status`, port 3129, browser proxy, https filtering, auto language
  filters, filtering-mode info, group/language filter DB rows, installed filters, user rules,
  proxy.yaml presence).
- `--testcycle` — drives `backend::run_command_blocking("stop")` then `("start")`, i.e. the exact
  path the Home toggle uses, and reports the port before/after.
- `--open-tab <id>` — opens straight on one tab so each tab can be inspected without mouse input.
  GTK's option parser rejects unknown flags, so `main` passes a filtered argv via `run_with_args`.

## Real-run verification (this session)
- `--selftest` against the live machine: cli `AdGuard CLI v1.4.13`, `status` OK, proxy.yaml present in
  both `~` and `/root`, installed filters 11, language filters 54, group 1 filters 3 — all matching the
  Python run of the same functions.
- `--testcycle`: port 3129 true → stop → false → start → true, browser proxy re-enabled. CLI linkage
  confirmed by `journalctl`/`sudo` log entries whose PWD was the Rust project directory.
- All 6 tabs opened via `--open-tab` and screenshotted; GUI runs with no GTK warnings from our CSS.

## Bugs found and fixed during the real runs
1. `home_tab.rs` — the sub-`Stack` was created and populated but never attached to `root`
   (Python: `outer_layout.addWidget(self.stack)`), so the Home tab rendered empty. Fixed.
2. `protection_tab.rs` — `container` (the module rows) was never attached to `scroll`/`main_view`
   (Python: `main_layout.addWidget(scroll_area)`), so the Protection tab rendered empty. Fixed.
3. `settings_network_view.rs` — same missing `scroll.setWidget(container)` + `addWidget(scroll)`. Fixed.
4. `i18n.rs` — `i18n_data.json` is language-first (`{"en":{k:v},"vi":{k:v}}`) while Python's
   TRANSLATIONS is key-first; the lookup was inverted, so every label showed its raw key
   (`nav_home`, `nav_protection`, …). Now the JSON is inverted once at load. Fixed.
5. `filter_db.rs` — `last_update_time == 0` was formatted as `01/01/1970`; Python treats a falsy
   value as an empty string. Fixed to match (`if ts > 0`), and the timestamp now goes through
   `libc::localtime_r` like the other modules (was UTC civil-from-days).
6. `style.rs` — 7 CSS lines had double braces around colors that are `format!` placeholders
   (`{{WHITE}}`), so GTK reported "Expected a valid color" and dropped those rules; and one
   `width:` (invalid in GTK CSS) was replaced with `min-width:`. Also added a global `scrollbar`
   rule. Fixed.
7. `dns_filters_view.rs` — the search icon was appended unconditionally; Python guards with
   `os.path.exists` (the file is absent here). Added the existence guard.

## Session 2 (completion pass)

### Backup removed
The old Rust project `adguard-blocker-backup` (2.2 GB) was deleted on request. Its documentation
was preserved first in `docs/legacy/` (`OLD_PORT_REPORT.md`, `OLD_README.md`, `OLD_GIT_LOG.txt`).

### Sub-page navigation added for verification
`--open-page <name>` dives into a sub-page of the opened tab (paired with `--open-tab`), so every
sub-view can be opened without mouse input. Supported by `home`, `protection` and `settings`
(`debug_show_page`), printing `shown` / `NOT FOUND` for each request.

### Full sub-page sweep — every view renders
- Protection (16 pages): detail, lang, group, user_rules, create_rule, tracking, custom_stealth,
  stealth_param, anti_dpi, dns, dns_server, dns_filters, dns_user_rules, dns_server_detail,
  browsing, extensions — all rendered with real content, no GTK warnings.
- Settings (15 pages): app, license, support, about, network, mode, https, updates_proxy, allowlist,
  ev, advanced, frag, fallback, bootstrap, secure_dns — all rendered.
- Home (5 pages): https, mode, updates, allowlist, ev — all rendered.
- Tabs: home, protection, statistics, apps, settings, updates — all rendered.

### Tray and start modes verified on the real desktop
- `--tray`: no window shown; `org.kde.StatusNotifierItem-<pid>-1` registered on the session bus;
  tooltip reads `AdGuard - Bảo vệ đang bật`; menu has the 6 Python items in Vietnamese
  (Mở AdGuard / Bật bảo vệ / Tắt bảo vệ trong 30 giây / Tắt bảo vệ / Đóng AdGuard nhưng tiếp tục lọc /
  Thoát AdGuard).
- Driving the menu over D-Bus: item 1 shows the window; item 6 hides it while the process and the
  3129 proxy keep running; item 4 disables protection (port closes) and the menu itself flips
  (Bật enabled, Tắt disabled); item 2 re-enables it (port reopens); item 7 quits the GUI and releases
  the single-instance socket while the proxy keeps running.
- `--autostart`: starts hidden with the tray registered, no window.

### Bugs found and fixed in this pass (7 more)
8. **`format!("$1...")` broke every YAML rewrite** — 11 call sites in `settings_advanced_view.rs`,
   `settings_network_view.rs` and `apps_config.rs`. Rust's regex crate parses `$1true` as a group
   named `1true`; the capture is not found, so it is replaced with an **empty string**. That wiped
   values in proxy.yaml (e.g. `show_notifications: true` → `show_notifications:`). All sites now use
   the braced form (`${{1}}` inside `format!`, `${1}` in plain literals). Found by a unit test that
   asserted the rewritten value; the file went from 221 lines to an empty value, not a shorter file.
9. **`home_tab` sub-Stack never attached** (already listed above as #1 — same root cause class as #2/#3).
10. **Protection tab container never attached** (#2 above).
11. **settings_network_view scroll never attached** (#3 above).
12. **website allowlist never rendered on open** — `load_data()` filled `all_domains` but
   `render_current_page()` was only called by search/pagination handlers, so the list appeared empty
   even though 4285 domains were loaded. Python calls `_render_current_page()` at the end of
   `_init_ui()`. Now 4273 domains / 535 pages are shown, matching Python.
13. **tray menu was frozen** — ksni caches the menu layout, and the old code only stored the running
   flag, so the enabled/disabled items kept their creation-time state while the tooltip updated.
   The tray `Handle` is now kept in `TRAY_HANDLE` and `update()` is called from
   `update_tray_menu_state()`, matching Python's `action.setEnabled(...)`.
14. **`--tray` / `--autostart` were rejected by GTK** (`Unknown option --tray`): they are read from
   `std::env::args()` but were never stripped before `run_with_args`. Now filtered.

### Unit tests added (15, all passing)
- `i18n`: table is key-first (guards the lang-first/key-first regression), nav labels in en+vi match
  Python, fallback order matches Python, both languages cover the same 535 keys.
- `filter_db`: user-rule counting/saving matches Python, zero timestamp is not formatted as a date.
- `stats_worker`: `format_count` / `format_bytes` match Python, `COMPANY_MAP` has the same 56 entries.
- `apps_config`: action fallback matches Python (including that an empty exec name hits the catch-all
  `- name: '*'` rule → `bypass_https`), the catch-all regex rewrites correctly, and the substitution
  is run against this machine's real proxy.yaml to prove it does not corrupt unrelated lines.
- `settings_advanced_view`: every YAML replacement template keeps its literal text, and the two-line
  `har_writer` block survives the rewrite.

### Final checks
- `cargo build` (debug + release), `cargo test` (15 passed), `cargo clippy` (0 errors).
- `proxy.yaml` for both `~` and `/root` byte-identical to the pre-test backup afterwards.
- Real proxy restored and running (adguard.service active, port 3129 listening).

Remaining known deviation (recorded, not a bug I introduced): GTK emits
`GtkGizmo … (slider) reported min width -2` on some scrollbars. Verified unrelated to our CSS —
the same warning appears with our scrollbar rules removed — and comes from the environment's
Breeze GTK theme. Python/PyQt has no equivalent.

## Session 3 (launcher verification)
The app turned protection OFF by itself on every launch: status sync fed back into the switch
(`set_protection_state` -> `set_checked` -> `toggled` -> `on_switch_toggled` -> `set_protection(false)`),
and because "start" runs `systemctl restart`, the following 4 s poll read the restart gap as "off" and
sent another `stop` — 4 protection commands per launch and a killed proxy. Python guards this with
`blockSignals`; the Rust port had lost it. Fixed with an `Inner::syncing` flag in `home_tab.rs`, plus
the same feedback loop in `home_network_section.rs` (its 4 s refresh kept rewriting proxy.yaml and
calling adguard-cli). Verified: cold start now issues exactly one command and the port stays up.

Also added the launcher (`run.sh`, desktop entry) and fixed `--tray`/`--autostart`, which GTK's
option parser was rejecting.

## Session 4 — UI/UX defects reported by the owner
25. **Master switch did not affect the rows under it** (Bảo vệ → Chống theo dõi and every module
    page). Python calls `row.setEnabled(val)` **and** `setStyleSheet("opacity: 1.0/0.5")`; the Rust
    port only called `set_sensitive()`. GTK does not dim insensitive widgets, and the click
    controller attached to the row container kept firing, so the options were still selectable with
    the master switch off. Fixed: `PrivacyLevelRow::set_active_state()` sets sensitivity **and**
    opacity, and every gesture handler now bails out on an insensitive widget.
26. **⋮ button glued to the Back button.** Python: `back` -> `addStretch()` -> `more`. Eight top bars
    had no stretch. Added a hexpand spacer in create_rule, custom_stealth, dns_protection,
    protection_detail, settings_advanced, settings_general, settings_license, user_rules.
27. **Home page laid out differently from Python.** The content column had no
    `setContentsMargins(40, 20, 40, 30)` and used `halign=Fill`, so the fixed-width 500px network
    column was handed the leftover width. Measured on screen: cards were 641px, then 570px; Python is
    497px. Now **497px at x=349..846** (Python 497px at x=346..843).
28. **Qt `style='...'` on links rendered as raw HTML.** Pango rejects that attribute and then drops
    the whole label. Added `to_pango_link_markup()` plus a unit test over every i18n value containing
    HTML (`apps_subtitle`, `send_usage_data`, `http_proxy_desc`).
29. **Crash on "Check for updates"** — `*x.borrow_mut() = (*x.borrow() + n)` double-borrows the same
    RefCell and panics, aborting the process through a C callback. Fixed in `updates_tab.rs` and the
    identical pattern in `settings_secure_dns_view.rs`.
30. **DNS filter editor could not be closed** — 13 secondary windows used a bare
    `gtk4::Window::new()` with no application/transient parent, so the WM gave them no decoration.
    Added `util::sub_window()` and routed all of them through it.
31. **Editor header read "Filtering rulesFiltering rules syntax"** — missing stretch between title
    and link. Added.
32. **Drop-down had a smaller grey block inside the frame** — GTK's DropDown wraps a button whose own
    background showed through; flattened with `> button` / `popup > contents` CSS.

### CLI coverage check (the app is a wrapper)
Ran `adguard-cli --help-all` (17 commands) and compared with what the GUI actually drives. The Rust
port wraps at least as much as Python: start/stop/restart/status, license, config get/set,
list-add/remove (filters enable/disable), filters add/install/remove/update/set-trusted/set-title,
dns filters, userscripts list/install/remove/enable/disable, check-update, update, reset-license,
cert, export-settings, import-settings. `configure`, `speed`, `export-logs` and
`install-browser-integration` are not exposed by the Python GUI either, so they were left out.

### Session 4 checks
`cargo build` (debug + release) pass, `cargo test` **20 passed**, `cargo clippy` 0 errors, 0 stubs.
Verified on screen: master-switch dimming (options greyed and unclickable), Home card width
497px matching Python, drop-down flattened, link labels rendering as real links.

## Session 5 — tray icon colour + Python bug audit

33. **Tray icon showed the theme's generic shield instead of the AdGuard logo.** Two problems
    stacked:
    - `icon_name()` returned `"security-high"`. Per the SNI spec a host panel resolves `IconName`
      through the icon theme *first*, so the themed glyph always won and `IconPixmap` (the real
      logo) was never used. Python only ever sends the pixmap, so it never hit this.
    - After forcing the pixmap, the colour was still wrong: the byte reorder assumed the source
      was BGRA, but tracing `Texture::format()` showed `R8g8b8a8` (bytes R,G,B,A in memory).
      It was converting to `A,B,G,R` instead of the SNI-required `A,R,G,B`.
    Fixed: `icon_name()` now returns an empty string so the host falls back to `IconPixmap`, and
    the reorder is `A,R,G,B`. Confirmed by the owner looking at the panel.

### Python bug audit (whole tree, not just backend.py)
Ran an AST scan over all 70 Python files for modules used but never imported, and for `self.x`
used before assignment. Only one real bug exists:

- `adguard_modern/backend.py` uses `re.` **7 times with no `import re`**. Every call sits inside a
  `try/except Exception: pass`, so the `NameError` is swallowed and three functions silently
  return their fallback values instead of reading the real config:
  `get_auto_language_filters()` (always True), `is_https_filtering_enabled()` (always True),
  `get_filtering_mode_info()` (always defaults).
  The Rust port already implements the intended regex behaviour (a deliberate, recorded
  deviation) rather than copying the bug. The Python source was left untouched.

### UI-vs-CLI coverage re-check (owner asked whether UI features were left unported)
- `adguard-cli --help-all` lists 17 commands.
- All 41 `menu_*` / export / import / reset i18n keys are present in the Rust tree, each used in
  code (i18n table entry + call site), so no menu action is missing.
- No empty/no-op handler functions exist (only `open_url`, which is correct).
- `settings_general`'s ⋮ menu has all 3 Python items (reset / export / import settings).
- `configure`, `speed`, `export-logs`, `install-browser-integration` have **no UI in the Python app
  either**, so they were not added — the app is a partial wrapper by design, matching the original.

## Session 6 — replacing the old install (and a packaging bug it exposed)

Removed the old Python GUI install and installed the Rust build in its place:
- deleted `~/.local/share/adguard_modern_gui` (the old Python app, 4.0 MB) and the stale launchers
  and desktop entries (`~/.local/bin/adguard-gui`, `adguard-gui.desktop`, `adguard-modern.desktop`,
  `AdGuard_UI.desktop`, the autostart entry, `~/Desktop/AdGuard.desktop`);
- `packaging/remove-old-python-app.sh` automates exactly that and re-checks the VPN entries;
- the AdGuard VPN app is fully independent (`/opt/adguardvpn_cli`, its own desktop entries and
  autostart file) and was verified untouched.

### Bug found in our own packaging script (fixed)
`setup_adguard_gui.sh` ran `pacman -Sy --needed gtk4 librsvg nss`. That is a partial upgrade:
it refreshed the package database and upgraded only those packages, moving gtk4 4.22.5 -> 4.24.1
while glib2 stayed at 2.88.3. gtk4 4.24.1 needs `g_timeout_source_new_ns`, which only exists in
glib2 2.90.1, so **every GTK4 application on the machine stopped starting** (pamac-manager,
pavucontrol, zenity, loupe, handbrake, mission-center, shelly, plus our own GUI).

Recovery: downgraded the three packages from the pacman cache
(`pacman -U /var/cache/pacman/pkg/{gtk4-1:4.22.5,librsvg-2:2.62.3,nss-3.130}*.zst`) and verified
all affected applications start again and `ldd libgtk-4.so.1` reports no missing libraries.

Script fixes so it cannot happen again:
- step 2 no longer installs anything. It only *checks* for gtk4/librsvg/nss/curl/ca-certificates and
  prints the exact `sudo pacman -S --needed ...` line, leaving system-wide sync to `pacman -Syu`.
- the install directory is `chown`ed back to the desktop user (the script runs under sudo, so
  everything used to land as root and the GUI could not write its cache).
- the GUI is no longer launched from the script: it runs as root there, which would give the GUI
  root's HOME, no session bus, no tray and no theme. It now starts as the desktop user, detached.
