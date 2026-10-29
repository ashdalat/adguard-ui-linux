// CSS accumulated from the per-file Qt stylesheets, ported file by file.
use crate::constants as c;

pub fn css() -> String {
    format!(
        r#"
window {{ background-color: {BG}; color: {WHITE}; }}
/* GTK4 scrollbars: the movable part is a `slider` node and needs both
   min-width and min-height, otherwise GTK reports negative gizmo sizes. */
scrollbar {{ background: transparent; min-width: 6px; min-height: 6px; }}
scrollbar.vertical slider {{ background-color: #3A3A3D; min-width: 6px; min-height: 25px; border-radius: 3px; }}
scrollbar.horizontal slider {{ background-color: #3A3A3D; min-width: 25px; min-height: 6px; border-radius: 3px; }}
scrollbar slider:hover {{ background-color: #55555A; }}

/* widgets/sidebar.py */
.sidebar {{ background-color: {SIDEBAR_BG}; border-right: 1px solid {BORDER}; }}
.sidebar-btn {{ background: transparent; border: none; border-radius: 8px; padding: 0 14px; min-height: 46px; }}
.sidebar-btn:hover {{ background-color: {SIDEBAR_HOVER}; }}
.sidebar-btn.active {{ background-color: {SIDEBAR_ACTIVE}; }}
.sidebar .lbl {{ color: {WHITE}; font-weight: normal; }}
.sidebar .lbl.bold {{ color: {WHITE}; font-weight: bold; }}
.sidebar .title {{ color: {WHITE}; font-size: 15pt; font-weight: bold; }}
.svg-icon {{ background: transparent; }}

/* widgets/feature_icons.py */
.feature-icon-btn {{ background: transparent; border: none; border-radius: 6px; padding: 6px; }}
.feature-icon-btn:hover {{ background-color: #38383C; }}
.feature-icons-row {{ padding: 0; }}

/* widgets/privacy_level_row.py */
.privacy-row {{ background: transparent; border-radius: 8px; }}
.privacy-row:hover {{ background-color: #202024; }}
.lbl-title {{ color: {WHITE}; font-size: 11pt; font-weight: bold; }}
.lbl-desc {{ color: {MUTED}; font-size: 12px; }}
.lbl-warning {{ color: #E5983B; font-size: 11px; }}
.divider-v {{ background-color: #38383D; min-width: 1px; min-height: 24px; }}
.chevron-btn {{ background: transparent; border: none; color: #8E8E93; font-size: 20px; font-weight: bold; min-width: 28px; min-height: 28px; padding: 0; }}
.chevron-btn:hover {{ color: {WHITE}; }}

/* widgets/dns_controls.py */
.dns-server-row {{ background: transparent; border-radius: 8px; }}
.dns-server-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.provider-row {{ background-color: #242427; border: 1px solid #333336; border-radius: 8px; }}
.provider-row:hover {{ background-color: #2C2C30; border-color: #4E4E54; }}
.lbl-desc-9 {{ color: {MUTED}; font-size: 9pt; }}
.dlg-dark {{ background-color: #1F1F22; border: 1px solid #333336; border-radius: 12px; }}
.dlg-title-18 {{ color: {WHITE}; font-size: 18pt; font-weight: bold; }}
.dlg-title-16 {{ color: {WHITE}; font-size: 16pt; font-weight: bold; }}
.dlg-close {{ background: transparent; border: none; color: {MUTED}; font-size: 16px; font-weight: 300; min-width: 28px; min-height: 28px; padding: 0; }}
.dlg-close:hover {{ color: {WHITE}; }}
.input-dark {{ background-color: #2C2C2F; color: {WHITE}; border: 1px solid #3E3E44; border-radius: 8px; padding: 0 14px; font-size: 12px; }}
.input-dark:focus {{ border-color: {GREEN}; }}
.green-btn {{ background-color: {GREEN}; color: {WHITE}; border: none; border-radius: 8px; font-size: 13px; font-weight: bold; padding: 0 14px; }}
.green-btn:hover {{ background-color: {GREEN_HOVER}; }}
.outline-btn {{ background: transparent; color: {WHITE}; border: 1px solid #4E4E54; border-radius: 8px; font-size: 13px; font-weight: bold; padding: 0 14px; }}
.outline-btn:hover {{ background-color: #28282C; border-color: #66666C; }}

/* widgets/assistant_modal.py */
.assistant-card {{ background-color: #202023; border: 1px solid #38383C; border-radius: 12px; }}
.assistant-title {{ color: {WHITE}; font-size: 18pt; font-weight: bold; }}
.green-link {{ background: transparent; border: none; color: {GREEN}; font-size: 12px; font-weight: 500; padding: 0; }}
.green-link:hover {{ color: {GREEN_HOVER}; text-decoration: underline; }}
.collapse-btn {{ background: transparent; border: none; color: {WHITE}; font-size: 13px; font-weight: bold; padding: 4px 8px; }}
.browser-row {{ background: transparent; border-radius: 8px; }}
.browser-row:hover {{ background-color: #2A2A2E; }}
.lbl-title-10 {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.lbl-sub-8 {{ color: {MUTED}; font-size: 8pt; }}

/* widgets/filter_editor_item.py */
.fe-item {{ background: transparent; border-radius: 6px; }}
.fe-item:hover {{ background-color: #26262A; }}
.fe-item-active {{ background-color: #2D2D32; }}
.lbl-tags-8 {{ color: #7A7A80; font-size: 8pt; }}
checkbutton check {{ background-color: transparent; border: 1px solid #505056; border-radius: 4px; min-width: 17px; min-height: 17px; }}
checkbutton check:hover {{ border-color: {GREEN}; }}
checkbutton check:checked {{ background-color: {GREEN}; border-color: {GREEN}; }}

/* widgets/filter_editor_window.py */
.fe-window {{ background-color: #1A1A1D; color: {WHITE}; }}
.fe-header {{ background-color: #202024; border-bottom: 1px solid #2F2F34; }}
.fe-search {{ background-color: #242428; border: 1px solid #36363B; border-radius: 6px; }}
.search-entry {{ background: transparent; color: {WHITE}; border: none; font-size: 12px; }}
.fe-tool-btn {{ background: #26262A; color: {WHITE}; border: 1px solid #38383C; border-radius: 4px; font-size: 15px; padding: 0; min-width: 28px; min-height: 28px; }}
.fe-tool-btn:hover {{ background-color: #35353B; }}
.fe-center {{ background-color: #1D1D20; border: 1px solid #2B2B30; border-radius: 8px; }}
.fe-green-btn {{ background-color: {GREEN}; color: {WHITE}; font-weight: bold; font-size: 13px; border-radius: 6px; padding: 0 18px; border: none; }}
.fe-green-btn:hover {{ background-color: {GREEN_HOVER}; }}
.fe-import-btn {{ background-color: #274D34; color: {WHITE}; font-weight: bold; font-size: 13px; border-radius: 6px; padding: 0 18px; border: 1px solid #3E7350; }}
.fe-import-btn:hover {{ background-color: #2E5C3E; }}
.fe-raw-view {{ background-color: #17171A; color: {GREEN}; border: 1px solid #38383D; border-radius: 6px; padding: 8px; font-family: monospace; font-size: 10pt; }}
.fe-rule-row {{ background-color: #222226; border-radius: 4px; }}
.fe-rule-row:hover {{ background-color: #2C2C32; }}
.lbl-group-9 {{ color: {MUTED}; font-size: 9pt; font-weight: bold; }}
.lbl-header-10 {{ color: #A0A0A5; font-size: 10pt; font-weight: bold; }}
.lbl-empty-12 {{ color: #A0A0A5; font-size: 12pt; }}
.lbl-num-9 {{ color: #7A7A80; font-family: monospace; font-size: 9pt; }}
.lbl-rule-10 {{ color: {WHITE}; font-family: monospace; font-size: 10pt; }}
.lbl-white-10 {{ color: {WHITE}; font-size: 10pt; }}
.opt-row {{ background: transparent; border: none; border-radius: 6px; padding: 6px 10px; color: {WHITE}; }}
.opt-row:hover {{ background-color: #26262A; }}

/* widgets/filtering_log_inspector.py */
.inspector {{ background-color: #1A1A1D; border-left: 1px solid #28282D; }}
.inspector-header {{ background-color: #1F1F23; border-bottom: 1px solid #28282D; }}
.inspector-page {{ background-color: #17171A; }}
.tab-btn {{ background: transparent; border: none; color: {MUTED}; font-size: 12px; padding: 4px 2px; }}
.tab-btn:hover {{ color: #C8C8CD; }}
.tab-active {{ color: {WHITE}; border-bottom: 2px solid {GREEN}; font-weight: bold; }}
.inspector-title {{ color: {WHITE}; font-weight: bold; font-size: 12px; }}
.inspector-value {{ color: #C0C0C6; font-size: 11px; }}
.inspector-status {{ font-size: 12px; font-weight: bold; }}
.status-green {{ color: {GREEN}; }}
.status-red {{ color: #E05D52; }}
.lbl-11 {{ color: {MUTED}; font-size: 11px; }}
.link-red {{ background: transparent; border: none; color: #E05D52; font-size: 11px; font-weight: 500; padding: 0; }}
.link-red:hover {{ color: #FF7B6E; }}
.link-green {{ background: transparent; border: none; color: {GREEN}; font-size: 11px; font-weight: 500; padding: 0; }}
.link-green:hover {{ color: #72D27E; }}

/* widgets/filtering_log_window.py */
.flog-window {{ background-color: #1A1A1D; color: {WHITE}; }}
.flog-header {{ background-color: #202024; border-bottom: 1px solid #28282D; }}
.flog-title {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.flog-subbar {{ background-color: #1E1E22; border-bottom: 1px solid #26262B; }}
.flog-tab {{ background: transparent; border: none; color: {MUTED}; font-size: 12px; padding: 4px 2px; }}
.flog-tab:hover {{ color: #D0D0D5; }}
.flog-tab.active {{ color: {WHITE}; font-weight: bold; border-bottom: 2px solid {GREEN}; }}
.flog-action-green {{ background: transparent; border: none; color: {GREEN}; font-size: 12px; padding: 0; }}
.flog-action-green:hover {{ color: {GREEN_HOVER}; }}
.flog-action-red {{ background: transparent; border: none; color: #E05D52; font-size: 12px; font-weight: 500; padding: 0; }}
.flog-action-red:hover {{ color: #FF6B5E; }}
.flog-searchrow {{ background-color: #19191C; border-bottom: 1px solid #25252A; }}
.flog-searchbox {{ background-color: #222226; border: 1px solid #333338; border-radius: 6px; }}
.flog-emoji {{ color: {MUTED}; }}
.flog-gear {{ background: transparent; border: none; color: {GREEN}; font-size: 16px; min-width: 28px; min-height: 28px; padding: 0; }}
.flog-center {{ background-color: #161618; }}
.flog-empty {{ color: {MUTED}; font-size: 13pt; }}
.flog-headrow {{ background-color: #1A1A1E; border-bottom: 1px solid #28282E; }}
.flog-head {{ color: {MUTED}; font-size: 12px; font-weight: bold; }}
.flog-table {{ background-color: #161618; }}
.flog-table row {{ background: transparent; border-bottom: 1px solid #1D1D21; }}
.flog-table row:hover {{ background-color: #28282D; }}
.flog-cell {{ color: #C0C0C5; font-size: 12px; }}
.flog-cell-name {{ color: #D6D6DC; font-size: 12px; }}
.flog-cell-status {{ font-size: 9pt; font-weight: bold; }}
.flog-cell-proc {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.c-red {{ color: #E05D52; }}
.c-green {{ color: {GREEN}; }}
.c-muted {{ color: #A0A0A6; }}
.c-orange {{ color: #E5983B; }}
.c-blue {{ color: #4EA1F3; }}
.c-purple {{ color: #A855F7; }}
.c-sky {{ color: #38BDF8; }}

/* widgets/stat_modals.py */
.stat-modal {{ background-color: #1A1A1D; color: {WHITE}; }}
.stat-card {{ background-color: #212124; border: 1px solid #38383D; border-radius: 12px; }}
.stat-title {{ color: {WHITE}; font-size: 17pt; font-weight: bold; }}
.stat-body {{ color: #D2D2D6; font-size: 13px; }}
.stat-green-btn {{ background-color: {GREEN}; color: #FFFFFF; font-size: 13px; font-weight: bold; border: none; border-radius: 8px; padding: 0 14px; min-height: 42px; }}
.stat-green-btn:hover {{ background-color: {GREEN_HOVER}; }}
.stat-orange-btn {{ background-color: #EF5A3B; color: #FFFFFF; font-size: 13px; font-weight: bold; border: none; border-radius: 8px; padding: 0 14px; min-height: 42px; }}
.stat-orange-btn:hover {{ background-color: #FA6445; }}
.stat-cancel-btn {{ background-color: #242428; color: #FFFFFF; font-size: 13px; font-weight: bold; border: 1px solid #4A4A52; border-radius: 8px; padding: 0 14px; min-height: 42px; }}
.stat-cancel-btn:hover {{ background-color: #323238; border-color: #5E5E68; }}

/* tabs/placeholders.py */
.placeholder-tab {{ background-color: {BG}; }}
.placeholder-title {{ color: {WHITE}; font-size: 20pt; font-weight: bold; }}
.placeholder-sub {{ color: {MUTED}; font-size: 11pt; }}

/* tabs/stat_components.py */
.bar-white {{ background-color: #E5E5EA; }}
.bar-green {{ background-color: {GREEN}; }}
.bar-orange {{ background-color: #E5983B; }}
.bar-red {{ background-color: #E25642; }}
.stat-row {{ background: transparent; border-radius: 8px; }}
.stat-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.stat-row-title {{ color: {WHITE}; font-size: 11pt; font-weight: bold; }}
.stat-badge {{ background-color: #38383E; color: #FFFFFF; font-weight: bold; font-size: 11px; border-radius: 4px; }}
.stat-num {{ font-size: 12px; }}
.stat-num.bold {{ font-weight: bold; }}
.stat-slash {{ color: #55555A; font-size: 12px; }}
.stat-chevron {{ color: #8E8E93; font-size: 16px; font-weight: 300; }}
.stat-bar {{ border-radius: 2px; }}
.c-orange-dark {{ color: #E67E22; }}
.c-lightgray {{ color: #DCDCE0; }}

/* widgets/common_controls.py */
.back-btn {{ background: transparent; border: none; padding: 0; }}
.back-btn .lbl {{ color: {WHITE}; }}
.back-btn:hover .lbl {{ color: #68BC71; }}

/* tabs/companies_view.py */
.page-bg {{ background-color: {BG}; }}
.page-title {{ color: {WHITE}; font-size: 22pt; font-weight: bold; }}
.flat-link {{ background: transparent; border: none; color: #8E8E93; font-size: 13px; font-weight: 500; padding: 4px 8px; }}
.flat-link:hover {{ color: #FFFFFF; }}
.flat-icon-btn {{ background: transparent; border: none; border-radius: 6px; padding: 6px; }}
.flat-icon-btn:hover {{ background-color: {SIDEBAR_HOVER}; }}
.search-box {{ background-color: #242428; border: 1px solid #36363B; border-radius: 8px; padding: 0 14px; }}
.search-input {{ background: transparent; border: none; color: #FFFFFF; font-size: 13px; }}
.company-row {{ background: transparent; border-radius: 8px; }}
.company-row:hover {{ background-color: #28282D; }}
.chev-padded {{ padding-left: 2px; }}
.transparent-scroll {{ background: transparent; border: none; }}

/* tabs/statistics_tab.py */
.section-title {{ color: {WHITE}; font-size: 15pt; font-weight: bold; margin-top: 6px; }}
.link-btn {{ background: transparent; border: none; color: {WHITE}; font-size: 11pt; padding: 6px 8px; }}
.link-btn:hover {{ color: {GREEN}; }}
.menu-popover {{ background-color: #2B2B30; border: 1px solid #3E3E44; border-radius: 8px; padding: 6px 0px; }}
.menu-item {{ background: transparent; border: none; color: #E2E2E6; font-size: 13px; font-weight: 500; padding: 9px 24px 9px 16px; }}
.menu-item:hover {{ background-color: #3C3C43; color: #FFFFFF; }}

/* tabs/home_network_section.py */
.settings-card {{ background-color: #242428; border: 1px solid #333338; border-radius: 10px; padding: 14px 16px; }}
/* Python: home_network_section.setFixedWidth(500), i.e. NO growth. GTK has no
   fixed-width property (max-width is not supported), so the column is kept at
   500px by NOT letting it expand and putting the extra room in the parent. */
.home-net-fixed {{ min-width: 500px; }}
.settings-card-clickable:hover {{ border: 1px solid {GREEN}; background-color: #2A2A2E; }}
.nav-row {{ background: transparent; border-radius: 8px; padding: 10px 12px 10px 8px; }}
.nav-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.nav-row-title {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}

/* tabs/settings_https_filtering_view.py */
.lbl-sub-11 {{ color: {MUTED}; font-size: 11pt; }}
.lbl-10 {{ color: {MUTED}; font-size: 10pt; }}
.toggle-item {{ background: transparent; border-radius: 8px; padding: 6px 8px; }}
.toggle-item-hover:hover {{ background-color: {SIDEBAR_HOVER}; }}
.outline-btn-10 {{ background-color: transparent; border: 1px solid #55555A; border-radius: 8px; color: #FFFFFF; padding: 0 18px; font-weight: 500; font-size: 10pt; }}
.outline-btn-10:hover {{ background-color: #2C2C30; border: 1px solid #707076; }}

/* tabs/settings_filtering_mode_view.py */
.mode-option {{ background: transparent; border-radius: 8px; padding: 6px 12px 6px 8px; }}
.mode-option:hover {{ background-color: {SIDEBAR_HOVER}; }}
checkbutton.radio-indicator check {{ min-width: 20px; min-height: 20px; border-radius: 10px; border: 2px solid #55555A; background: transparent; }}
checkbutton.radio-indicator check:checked {{ border-color: {GREEN}; background-color: {GREEN}; }}
.port-input {{ background-color: #262629; border: 1px solid #38383E; border-radius: 8px; padding-left: 14px; color: {WHITE}; font-size: 10pt; font-weight: 500; }}
.port-input:focus {{ border-color: {GREEN}; }}
.port-input:disabled {{ color: #55555A; border-color: #2D2D32; }}

/* tabs/settings_proxy_updates_view.py */
.proxy-radio-box {{ background: transparent; border-radius: 8px; padding: 4px 12px 4px 8px; }}
.proxy-radio-box:hover {{ background-color: {SIDEBAR_HOVER}; }}
.proxy-input {{ background-color: #262629; border: 1px solid #38383E; border-radius: 8px; padding-left: 14px; color: {WHITE}; font-size: 10pt; font-weight: 500; }}
.proxy-input:focus {{ border-color: {GREEN}; }}
.dimmed {{ opacity: 0.5; }}

/* tabs/settings_ev_certificates_view.py */
.ev-add-btn {{ background: transparent; border: none; color: {GREEN}; font-size: 11pt; font-weight: 600; padding: 4px 0; }}
.ev-add-btn:hover {{ color: {GREEN_HOVER}; }}
.dlg-input {{ background-color: #1A1A1C; color: #FFFFFF; border: 1px solid #3E3E44; border-radius: 6px; padding: 0 12px; font-size: 11pt; }}
.dlg-input:focus {{ border-color: {GREEN}; }}

/* tabs/settings_website_allowlist_view.py */
.allowlist-row {{ background-color: transparent; border-radius: 6px; }}
.allowlist-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.allowlist-domain {{ color: #FFFFFF; background: transparent; font-size: 11pt; font-weight: 500; }}
.allowlist-empty {{ color: {MUTED}; font-size: 11pt; padding: 20px 0; }}
.pag-btn {{ border: none; border-radius: 6px; font-size: 11pt; }}
.pag-btn-normal {{ background-color: #242427; color: #FFFFFF; }}
.pag-btn-normal:hover {{ background-color: #2C2C30; }}
.pag-btn-active {{ background-color: {GREEN}; color: #FFFFFF; font-weight: bold; }}
.pag-btn-disabled {{ background-color: #242427; color: #55555A; }}
.pag-ellipsis {{ color: {MUTED}; font-size: 11pt; }}
.pag-del {{ background: transparent; border: none; color: #888888; font-size: 13px; }}
.pag-del:hover {{ color: #FF5252; }}
.menu-separator {{ color: {MUTED}; font-size: 10pt; padding: 6px 16px; }}
.dlg-title {{ color: #FFFFFF; font-size: 13pt; font-weight: bold; }}

/* tabs/home_tab.py */
.home-tab {{ background-color: {BG}; }}
.home-container {{ background-color: {BG}; }}
.home-scroll {{ background-color: transparent; border: none; }}
.home-scroll scrollbar {{ background: transparent; min-width: 6px; }}
.home-scroll scrollbar.vertical slider {{ background-color: #3A3A3D; min-width: 6px; min-height: 25px; border-radius: 3px; }}
.home-scroll scrollbar.horizontal slider {{ background-color: #3A3A3D; min-width: 25px; min-height: 6px; border-radius: 3px; }}
.home-scroll scrollbar slider:hover {{ background-color: #55555A; }}
.home-headline {{ color: {WHITE}; font-size: 24pt; font-weight: bold; }}
.home-slogan {{ color: {SUBTITLE}; font-size: 11pt; }}
.home-detail {{ color: {MUTED}; font-size: 9pt; }}

/* tabs/updates_tab.py */
.updates-btn {{ background-color: transparent; border: 1px solid #444448; border-radius: 6px; color: {WHITE}; font-weight: bold; font-size: 10pt; padding: 0 14px; }}
.updates-btn:hover {{ background-color: {SIDEBAR_HOVER}; border-color: #66666B; }}
.updates-btn:disabled {{ color: {MUTED}; border-color: #333336; }}
.updates-result {{ color: {GREEN}; font-size: 9pt; }}

/* tabs/settings_about_view.py */
.lbl-white-12-bold {{ color: {WHITE}; font-size: 12pt; font-weight: bold; }}
.about-link {{ background: transparent; border: none; color: {GREEN}; font-size: 10pt; font-weight: 500; padding: 4px 0; }}
.about-link:hover {{ color: #72CF7E; text-decoration-line: underline; }}

/* tabs/settings_support_view.py */
.support-row {{ background-color: transparent; border-radius: 8px; }}
.support-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.support-title {{ color: {WHITE}; font-size: 11pt; font-weight: bold; background: transparent; }}

/* tabs/settings_frag_delay_view.py */
.frag-input {{ background-color: #363639; color: #FFFFFF; border: 1px solid #46464B; border-radius: 8px; padding: 0 14px; font-size: 11pt; }}
.frag-input:focus {{ border-color: {GREEN}; }}
.lbl-10-medium-white {{ color: {WHITE}; font-size: 10pt; font-weight: 500; }}

/* tabs/settings_bootstrap_view.py */
.bs-edit {{ background-color: #363639; color: #FFFFFF; border: 1px solid #46464B; border-radius: 8px; padding: 12px 14px; font-size: 10.5pt; font-family: 'Inter', monospace; }}
.bs-edit:focus {{ border-color: {GREEN}; }}

/* tabs/settings_secure_dns_view.py */
.dns-option-row {{ background-color: transparent; border-radius: 6px; }}
.dns-option-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.dns-dropdown-frame {{ background-color: #262629; border: 1px solid #3E3E44; border-radius: 8px; }}
.dns-dimmed {{ opacity: 0.4; }}
.dns-header {{ background-color: #363639; border: 1px solid #46464B; border-radius: 8px; }}
.dns-header:hover {{ background-color: #3D3D42; }}
.dns-panel {{ background-color: #2D2D30; border: 1px solid #3E3E44; border-top: none; border-bottom-left-radius: 8px; border-bottom-right-radius: 8px; }}
.dns-arrow {{ color: {MUTED}; font-size: 11pt; font-weight: bold; }}

/* tabs/settings_fallback_dns_view.py */
.transparent-container {{ background: transparent; }}
.fb-combo {{ background-color: #363639; color: #FFFFFF; border: 1px solid #46464B; border-radius: 8px; padding: 0 14px; font-size: 10.5pt; }}
.fb-edit {{ background-color: #363639; color: #FFFFFF; border: 1px solid #46464B; border-radius: 8px; padding: 10px 12px; font-size: 10.5pt; font-family: 'Inter', monospace; }}
.fb-edit:focus {{ border-color: {GREEN}; }}
.fb-edit-10 {{ background-color: #363639; color: #FFFFFF; border: 1px solid #46464B; border-radius: 8px; padding: 10px 12px; font-size: 10pt; font-family: 'Inter', monospace; }}
.fb-edit-10:focus {{ border-color: {GREEN}; }}
.lbl-11-bold-white {{ color: {WHITE}; font-size: 11pt; font-weight: bold; }}

/* tabs/settings_network_view.py */
.lbl-10-white {{ color: {WHITE}; font-size: 10pt; }}

/* tabs/settings_general_view.py */
.lbl-10-bold-white {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.combo-general {{ background-color: #262629; color: #FFFFFF; border: 1px solid #38383E; border-radius: 8px; padding: 0 14px; font-size: 10pt; font-weight: 500; }}
.combo-general:hover {{ border-color: #484850; }}
.combo-general > button {{ background-color: transparent; background-image: none; border: none; box-shadow: none; padding: 0 4px; }}
.combo-general > button:hover {{ background-color: transparent; }}
.combo-general button > * {{ background-color: transparent; }}

/* tabs/settings_advanced_view.py */
.adv-row-clickable {{ background-color: transparent; border-radius: 8px; }}
.adv-row-clickable:hover {{ background-color: {SIDEBAR_HOVER}; }}
.adv-warn {{ color: #D9822B; font-size: 10pt; font-weight: 500; background: transparent; }}
.adv-cat-header {{ color: {WHITE}; font-size: 13pt; font-weight: bold; margin-top: 10px; }}
.adv-extra {{ color: {MUTED}; font-size: 9pt; font-weight: 500; background: transparent; margin-top: 4px; }}
.red-btn {{ background-color: #D32F2F; color: #FFFFFF; border: none; border-radius: 6px; padding: 6px 18px; font-weight: bold; font-size: 10pt; min-width: 70px; }}
.red-btn:hover {{ background-color: #B71C1C; }}
.menu-item-red {{ background: transparent; border: none; color: #FA5C5C; font-size: 10pt; font-weight: 500; padding: 8px 18px; }}
.menu-item-red:hover {{ background-color: #38383E; color: #FA5C5C; }}

/* tabs/settings_license_view.py */
.lbl-13-bold-white {{ color: {WHITE}; font-size: 13pt; font-weight: bold; }}
.lbl-12-bold-white {{ color: {WHITE}; font-size: 12pt; font-weight: bold; }}
.lbl-12-muted {{ color: {MUTED}; font-size: 12px; }}
.lbl-12-green {{ color: {GREEN}; font-size: 12px; }}
.license-card {{ background-color: #252528; border: 1px solid #3A3A40; border-radius: 12px; }}
.license-btn {{ background-color: #1D6B47; color: #FFFFFF; border: 1px solid #22A455; border-radius: 6px; font-size: 14px; font-weight: 600; padding: 0 12px; }}
.license-btn:hover {{ background-color: #28C063; color: #FFFFFF; }}
.license-btn-small {{ background-color: #1D6B47; color: #FFFFFF; border: 1px solid #22A455; border-radius: 6px; font-size: 13px; font-weight: 600; padding: 0 12px; }}
.license-btn-small:hover {{ background-color: #28C063; color: #FFFFFF; }}
.license-btn-dim {{ background-color: #1E3B25; color: #A3E4B0; border: 1px solid #3E6B44; border-radius: 6px; font-size: 13px; font-weight: 600; padding: 0 12px; }}
.license-btn-dim:hover {{ background-color: #28C063; color: #FFFFFF; }}
.license-red-btn {{ background-color: #241616; color: #EF5350; border: 1px solid #8E3535; border-radius: 6px; font-size: 13px; font-weight: 600; padding: 0 12px; }}
.license-red-btn:hover {{ background-color: #331C1C; border-color: #F87171; color: #FFA4A4; }}

/* tabs/settings_tab.py */
.settings-item-row {{ background-color: transparent; border-radius: 8px; }}
.settings-item-row:hover {{ background-color: {SIDEBAR_HOVER}; }}

/* tabs/protection_rows.py */
.protection-item-row {{ background-color: transparent; border-radius: 8px; }}
.protection-item-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.subfilter-row {{ background-color: transparent; border-radius: 8px; }}
.subfilter-row-clickable:hover {{ background-color: {SIDEBAR_HOVER}; }}
.chevron-label {{ color: #8E8E93; font-size: 20px; font-weight: bold; background: transparent; }}
.divider-v-2 {{ background-color: #55555A; min-width: 1px; }}

/* tabs/protection_detail.py */
.detail-back-btn {{ color: {WHITE}; background: transparent; border: none; font-size: 14px; font-weight: 500; padding: 4px 6px; border-radius: 4px; }}
.detail-back-btn:hover {{ background-color: #28282C; color: {GREEN}; }}
.detail-more-btn {{ background: transparent; border: none; border-radius: 4px; padding: 0; min-width: 28px; min-height: 28px; }}
.detail-more-btn:hover {{ background-color: #28282C; }}
.detail-warning {{ color: #E5983B; font-size: 10pt; font-weight: bold; background: transparent; }}
.detail-menu-popover {{ background-color: #242428; border: 1px solid #38383C; border-radius: 8px; padding: 6px; }}
.detail-menu-item {{ background: transparent; border: none; color: #FFFFFF; padding: 8px 18px; border-radius: 4px; font-size: 13px; }}
.detail-menu-item:hover {{ background-color: #2F2F35; color: {GREEN}; }}

/* tabs/language_filters_view.py */
.language-item-row {{ background-color: transparent; border-radius: 8px; }}
.language-item-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.lbl-16-bold-white {{ color: {WHITE}; font-size: 16pt; font-weight: bold; }}

/* tabs/filter_group_view.py */
.filter-item-row {{ background-color: transparent; border-radius: 8px; }}
.filter-item-row:hover {{ background-color: {SIDEBAR_HOVER}; }}

/* tabs/dns_filter_editor.py */
.dnsfe-window {{ background-color: #18181B; }}
.dnsfe-left {{ background-color: #1E1E21; }}
.dnsfe-right {{ background-color: #18181B; }}
.dnsfe-search {{ background-color: #262629; color: {WHITE}; border: 1px solid #333336; border-radius: 6px; padding: 0 10px; font-size: 11px; }}
.dnsfe-search:focus {{ border-color: {GREEN}; }}
.dnsfe-search-rule {{ background-color: #242427; color: {WHITE}; border: 1px solid #333336; border-radius: 6px; padding: 0 10px; font-size: 11px; }}
.dnsfe-search-rule:focus {{ border-color: {GREEN}; }}
.dnsfe-group-label {{ color: {MUTED}; font-size: 10pt; padding-top: 4px; background: transparent; }}
.dnsfe-row {{ background-color: transparent; border-radius: 6px; }}
.dnsfe-row:hover {{ background-color: #28282C; }}
.dnsfe-row-active {{ background-color: #2E2E33; border-radius: 6px; }}
.dnsfe-check {{ background-color: {GREEN}; color: {WHITE}; border-radius: 4px; font-weight: bold; font-size: 12px; min-width: 18px; min-height: 18px; }}
.dnsfe-check-off {{ background-color: transparent; border: 1px solid #4E4E54; border-radius: 4px; min-width: 18px; min-height: 18px; }}
.dnsfe-name {{ color: {WHITE}; font-size: 11pt; font-weight: bold; background: transparent; }}
.dnsfe-ver {{ color: {MUTED}; font-size: 8pt; background: transparent; }}
.dnsfe-count {{ color: {MUTED}; font-size: 9pt; background: transparent; }}
.dnsfe-tag {{ color: {GREEN}; font-size: 9pt; background: transparent; }}
.dnsfe-tool-btn {{ background: transparent; border: 1px solid #333336; border-radius: 4px; color: {MUTED}; font-size: 16px; padding: 0; min-width: 28px; min-height: 28px; }}
.dnsfe-tool-btn:hover {{ color: {WHITE}; background-color: #28282C; }}
.dnsfe-tool-btn-del {{ background: transparent; border: 1px solid #333336; border-radius: 4px; color: {MUTED}; font-size: 16px; padding: 0; min-width: 28px; min-height: 28px; }}
.dnsfe-tool-btn-del:hover {{ color: #EF5A3B; background-color: #28282C; }}
.dnsfe-list {{ background-color: #18181B; border: 1px solid #28282C; border-radius: 6px; color: {WHITE}; font-family: monospace; font-size: 12px; padding: 4px; }}
.dnsfe-list row {{ padding: 6px 10px; border-bottom: 1px solid #222225; background: transparent; }}
.dnsfe-list row:hover {{ background-color: #242427; }}
.dnsfe-list row:selected {{ background-color: #2E2E33; color: {GREEN}; }}
.dnsfe-rule-row {{ color: {WHITE}; font-family: monospace; font-size: 12px; }}
.dnsfe-dialog-entry {{ background-color: #2C2C2F; color: {WHITE}; border: 1px solid #3E3E44; border-radius: 6px; padding: 0 12px; font-family: monospace; font-size: 12px; }}
.dnsfe-dialog-entry:focus {{ border-color: {GREEN}; }}

/* tabs/user_rules_view.py */
.user-rule-row {{ background-color: #242427; border: 1px solid #343438; border-radius: 8px; }}
.user-rule-row:hover {{ background-color: {SIDEBAR_HOVER}; border: 1px solid #44444A; }}
.user-rule-delete-btn {{ color: {MUTED}; background: transparent; border: none; font-size: 13px; font-weight: bold; border-radius: 4px; padding: 0; min-width: 24px; min-height: 24px; }}
.user-rule-delete-btn:hover {{ color: #EF5A3B; background-color: #343438; }}
.ur-editor-card {{ background-color: #242427; border: 1px solid #333336; border-radius: 10px; }}
.ur-card-title {{ color: {WHITE}; font-size: 12pt; font-weight: bold; }}
.ur-card-desc {{ color: {MUTED}; font-size: 9pt; }}
.ur-go-btn {{ background-color: {GREEN}; color: {WHITE}; border: none; border-radius: 6px; font-weight: bold; font-size: 12px; min-width: 160px; min-height: 38px; }}
.ur-go-btn:hover {{ background-color: {GREEN_HOVER}; }}

/* tabs/create_rule_view.py */
.combo-no-wheel {{ background-color: #242427; color: {WHITE}; border: 1px solid #3E3E42; border-radius: 8px; padding: 0 14px; font-size: 13px; min-height: 44px; }}
.combo-no-wheel:hover {{ border-color: #55555A; }}
/* GTK's DropDown is a button wrapping a label; the inner button has its own
   background, which rendered as a smaller grey block inside the frame.
   Python styles the QComboBox as one flat box, so flatten the children. */
.combo-no-wheel > button {{ background-color: transparent; background-image: none; border: none; box-shadow: none; padding: 0 4px; min-height: 40px; }}
.combo-no-wheel > button:hover {{ background-color: transparent; }}
.combo-no-wheel button > * {{ background-color: transparent; }}
.combo-no-wheel popover > contents {{ background-color: #28282B; border: 1px solid #3E3E44; border-radius: 8px; padding: 4px; }}
.combo-no-wheel popover listview row:selected {{ background-color: #38383E; color: {WHITE}; }}
.cr-entry {{ background-color: #242427; color: {WHITE}; border: 1px solid #3E3E42; border-radius: 8px; padding: 0 14px; font-size: 13px; min-height: 44px; }}
.cr-entry:focus {{ border-color: {GREEN}; }}
.cr-create-btn {{ background-color: {GREEN}; color: {WHITE}; border: none; border-radius: 6px; font-size: 13px; font-weight: bold; min-width: 130px; min-height: 40px; }}
.cr-create-btn:hover {{ background-color: {GREEN_HOVER}; }}
.cr-cancel-btn {{ background-color: transparent; color: {WHITE}; border: 1px solid #55555A; border-radius: 6px; font-size: 13px; font-weight: 500; min-width: 130px; min-height: 40px; }}
.cr-cancel-btn:hover {{ background-color: #242427; border-color: #8E8E93; }}

/* tabs/tracking_protection_view.py */
.tp-back-btn {{ color: {MUTED}; background: transparent; border: none; font-size: 13px; font-weight: 500; padding: 4px 8px; }}
.tp-back-btn:hover {{ color: {WHITE}; }}

/* tabs/custom_stealth_view.py */
.stealth-setting-row {{ background-color: transparent; border-radius: 8px; }}
.stealth-row-clickable:hover {{ background-color: {SIDEBAR_HOVER}; }}
.lbl-9-bold-white {{ color: {WHITE}; font-size: 9pt; font-weight: bold; }}

/* tabs/stealth_param_view.py */
.sp-warning {{ color: #E5983B; font-size: 10pt; }}
.sp-field-title {{ color: #A0A0A6; font-size: 10pt; }}
.sp-input-box {{ background-color: #242428; border: 1px solid #36363B; border-radius: 8px; min-height: 44px; }}
.sp-input-box-dim {{ background-color: #1C1C20; border: 1px solid #28282D; border-radius: 8px; min-height: 44px; opacity: 0.5; }}
.sp-entry {{ background: transparent; border: none; color: {WHITE}; font-size: 13px; }}
.sp-clear-btn {{ color: #8E8E93; background: transparent; border: none; font-size: 12px; padding: 0; min-width: 22px; min-height: 22px; }}
.sp-clear-btn:hover {{ color: {WHITE}; }}

/* tabs/anti_dpi_view.py */
.adp-warn {{ color: #E5983B; font-size: 9pt; }}

/* tabs/dns_protection_view.py */
.dns-protection-view {{ background-color: {BG}; }}
.dns-server-card {{ background-color: transparent; border-radius: 8px; }}
.dns-server-card:hover {{ background-color: {SIDEBAR_HOVER}; }}
.dns-toggle-row {{ background-color: transparent; border-radius: 8px; }}
.dns-toggle-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.dns-current {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.dns-chevron {{ color: #8E8E93; font-size: 18px; font-weight: 300; }}
.dns-divider {{ background-color: #4E4E54; min-width: 1px; min-height: 30px; }}

/* tabs/dns_server_select_view.py */
.lbl-14-bold-white {{ color: {WHITE}; font-size: 14pt; font-weight: bold; }}
.dns-green-action {{ color: {GREEN}; background: transparent; border: none; font-size: 13px; font-weight: bold; padding: 6px 12px; border-radius: 6px; }}
.dns-green-action:hover {{ background-color: #242427; color: {GREEN_HOVER}; }}

/* tabs/dns_server_detail_view.py */
.dns-proto-label {{ color: {WHITE}; font-size: 10pt; font-weight: 500; }}
.dns-proto-combo {{ background-color: #242427; color: {WHITE}; border: 1px solid #38383C; border-radius: 6px; padding: 0 12px; font-size: 11px; min-height: 38px; }}
.dns-proto-combo:hover {{ background-color: #2C2C30; border-color: #4E4E54; }}
.dns-select-btn {{ background-color: {GREEN}; color: {WHITE}; border: none; border-radius: 8px; font-size: 12px; font-weight: bold; min-width: 190px; min-height: 40px; }}
.dns-select-btn:hover {{ background-color: {GREEN_HOVER}; }}
.dns-select-btn:disabled {{ background-color: #2D3D32; color: #6E8E75; }}

/* tabs/dns_filters_view.py */
.dns-filter-row {{ background-color: transparent; border-radius: 8px; }}
.dns-filter-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.lbl-8-muted {{ color: {MUTED}; font-size: 8pt; }}
.dns-search-box {{ background-color: #242427; border: 1px solid #343438; border-radius: 8px; min-height: 38px; }}

/* tabs/browsing_security_view.py */
.bs-option-row {{ background-color: transparent; border-radius: 8px; }}
.bs-option-row:hover {{ background-color: rgba(255, 255, 255, 0.02); }}
.bs-link {{ color: {GREEN}; background: transparent; border: none; font-size: 10px; padding: 2px 0 0 0; }}
.bs-link:hover {{ color: {GREEN_HOVER}; text-decoration: underline; }}

/* tabs/app_item_row.py */
.app-item-row {{ background-color: transparent; border-radius: 8px; }}
.app-item-row:hover {{ background-color: {SIDEBAR_HOVER}; }}
.app-badge-btn {{ background: transparent; border: none; padding: 1px; border-radius: 3px; min-width: 18px; min-height: 18px; }}
.app-badge-btn:hover {{ background-color: rgba(255, 255, 255, 0.1); }}
.app-warn-8 {{ color: #E67E22; font-size: 8pt; }}

/* tabs/apps_group.py */
.app-group-header {{ background: transparent; border: none; padding: 6px 0; }}
.app-group-arrow {{ color: {MUTED}; font-size: 15px; font-weight: bold; }}

/* tabs/extensions_view.py */
.ext-back-btn {{ color: {MUTED}; background: transparent; border: none; font-size: 13px; font-weight: 500; padding: 4px 0; }}
.ext-back-btn:hover {{ color: {WHITE}; }}
.ext-header-menu-btn {{ background: transparent; color: {MUTED}; font-size: 18px; font-weight: bold; border: none; border-radius: 6px; min-width: 28px; min-height: 28px; padding: 0; }}
.ext-header-menu-btn:hover {{ background-color: rgba(255, 255, 255, 0.08); color: {WHITE}; }}
.ext-card {{ background-color: #242427; border-radius: 10px; border: 1px solid rgba(255, 255, 255, 0.04); padding: 14px 16px; }}
.ext-card:hover {{ background-color: #2A2A2E; }}
.ext-check-btn {{ background-color: {GREEN}; color: #1E1E21; font-size: 13px; font-weight: bold; border: none; border-radius: 5px; min-width: 22px; min-height: 22px; padding: 0; }}
.ext-check-btn-off {{ background-color: transparent; border: 2px solid #6E6E73; border-radius: 5px; min-width: 22px; min-height: 22px; padding: 0; }}
.ext-title-14 {{ color: {WHITE}; font-size: 14px; font-weight: bold; }}
.ext-title-13 {{ color: {WHITE}; font-size: 13px; font-weight: bold; }}
.ext-type-12 {{ color: {MUTED}; font-size: 12px; }}
.ext-detail-12 {{ color: {MUTED}; font-size: 12px; }}
.lbl-13-muted {{ color: {MUTED}; font-size: 13px; }}
.ext-section-16 {{ color: {WHITE}; font-size: 16px; font-weight: bold; }}
.ext-link {{ color: {GREEN}; background: transparent; border: none; font-size: 13px; font-weight: 500; padding: 0; }}
.ext-link:hover {{ color: {GREEN_HOVER}; text-decoration: underline; }}
.ext-add-btn {{ color: {GREEN}; background: transparent; border: none; font-size: 14px; font-weight: 500; padding: 4px 0; }}
.ext-add-btn:hover {{ color: {GREEN_HOVER}; }}
.ext-browse-btn {{ background-color: #2D2D31; color: {WHITE}; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 6px; padding: 6px 12px; font-size: 12px; }}
.ext-browse-btn:hover {{ background-color: #36363B; }}
.ext-storage-view {{ background-color: #242427; color: #A0FFA0; font-family: monospace; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 6px; }}
.ext-code-view {{ background-color: #141416; color: #D4D4D4; font-family: monospace; border: 1px solid rgba(255, 255, 255, 0.1); border-radius: 8px; }}
.apps-tab-root {{ background-color: {BG}; }}
.apps-header-22 {{ color: {WHITE}; font-size: 22px; font-weight: bold; }}
.apps-more-btn {{ background: transparent; border: none; border-radius: 6px; }}
.apps-more-btn:hover {{ background-color: {SIDEBAR_HOVER}; }}
.apps-banner {{ background-color: #2D221F; border: 1px solid #7A3D2C; border-radius: 8px; padding: 4px; }}
.apps-banner-text {{ color: #EF826B; font-size: 10pt; font-weight: 500; background: transparent; }}
.apps-subtitle-10 {{ color: {MUTED}; font-size: 10pt; background: transparent; }}
.apps-scroll {{ background-color: transparent; border: none; }}
.apps-scroll scrollbar {{ background: transparent; min-width: 6px; }}
.apps-scroll scrollbar.vertical slider {{ background: #3A3A3D; min-width: 6px; min-height: 25px; border-radius: 3px; }}
.apps-scroll scrollbar.horizontal slider {{ background: #3A3A3D; min-width: 25px; min-height: 6px; border-radius: 3px; }}
.apps-scroll scrollbar slider:hover {{ background: #55555A; }}
.apps-fu-title {{ color: {WHITE}; font-size: 11pt; font-weight: bold; }}
.apps-fu-desc-9 {{ color: {MUTED}; font-size: 9pt; }}
.apps-search-box {{ background-color: #262629; border: 1px solid #333338; border-radius: 8px; }}
.apps-search-box:focus-within {{ border: 1px solid {GREEN}; }}
.apps-search-entry {{ background: transparent; border: none; color: {WHITE}; font-size: 10pt; }}
.apps-add-app-btn {{ background: transparent; border: none; color: {GREEN}; padding: 6px 10px; font-size: 11pt; font-weight: bold; }}
.apps-add-app-btn:hover {{ color: #79D485; }}

/* tabs/adguard_assistant_dialog.py */
.assistant-dialog-root {{ background-color: transparent; }}
.assistant-dialog-card {{ background-color: #242427; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; }}
.assistant-title-22 {{ color: {WHITE}; font-size: 16.5pt; font-weight: bold; }}
.assistant-close-btn {{ color: #8E8E93; background: transparent; border: none; font-size: 12pt; font-weight: bold; border-radius: 4px; }}
.assistant-close-btn:hover {{ color: {WHITE}; background-color: rgba(255, 255, 255, 0.08); }}
.assistant-desc-13 {{ color: #8E8E93; font-size: 10pt; }}
.assistant-more-btn {{ color: {GREEN}; background: transparent; border: none; font-size: 10pt; font-weight: 500; padding: 0; }}
.assistant-more-btn:hover {{ color: {GREEN_HOVER}; text-decoration: underline; }}
.assistant-other-toggle {{ color: {WHITE}; background: transparent; border: none; font-size: 10pt; font-weight: bold; padding: 4px 0; }}
.assistant-other-toggle:hover {{ color: {GREEN}; }}
.assistant-other-container {{ background: transparent; }}
.assistant-browser-row {{ background: transparent; }}
.assistant-browser-name {{ color: {WHITE}; font-size: 10.5pt; font-weight: bold; }}
.assistant-browser-sub {{ color: #8E8E93; font-size: 9pt; }}
.assistant-browser-status {{ color: #8E8E93; font-size: 10pt; }}
.assistant-legacy-card {{ background-color: #1E1E21; border: 1px solid rgba(255, 255, 255, 0.04); border-radius: 10px; }}
.assistant-legacy-title {{ color: {WHITE}; font-size: 10pt; font-weight: bold; }}
.assistant-legacy-sub {{ color: #8E8E93; font-size: 9pt; }}
"#,
        BG = c::COLOR_MAIN_BG,
        SIDEBAR_BG = c::COLOR_SIDEBAR_BG,
        SIDEBAR_HOVER = c::COLOR_SIDEBAR_HOVER,
        SIDEBAR_ACTIVE = c::COLOR_SIDEBAR_ACTIVE,
        BORDER = c::COLOR_BORDER,
        WHITE = c::COLOR_TEXT_WHITE,
        MUTED = c::COLOR_TEXT_MUTED,
        GREEN = c::COLOR_ADGUARD_GREEN,
        GREEN_HOVER = c::COLOR_ADGUARD_GREEN_HOVER,
        SUBTITLE = c::COLOR_TEXT_SUBTITLE,
    )
}
