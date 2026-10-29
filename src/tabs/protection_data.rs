// Port of adguard_modern/tabs/protection_data.py — protection module data,
// translated data, active filter ids and machine protection status.
use regex::Regex;
use rusqlite::Connection;
use std::collections::HashSet;
use std::process::Command;

use crate::constants as c;
use crate::i18n;

#[derive(Clone)]
pub struct SubItem {
    pub key: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub active: bool,
    pub warning: &'static str,
    pub has_page: bool,
}

#[derive(Clone)]
pub struct ModuleData {
    pub key: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    pub active: bool,
    pub has_switch: bool,
    pub sub_items: Vec<SubItem>,
}

fn sub(key: &'static str, title: &'static str, desc: &'static str, active: bool, warning: &'static str, has_page: bool) -> SubItem {
    SubItem { key, title, desc, active, warning, has_page }
}

/// Port of PROTECTION_MODULES (insertion order preserved).
pub fn protection_modules() -> Vec<ModuleData> {
    vec![
        ModuleData {
            key: "ad_blocking",
            icon: "feat_adblock.svg",
            title: "Ad blocking",
            desc: "Blocks unwanted ads like popups, banners, and video ads",
            active: true,
            has_switch: false,
            sub_items: vec![
                sub("basic", "Basic protection", "Removes most ads from websites", true, "Without Basic protection, most ads won’t be blocked", true),
                sub("search_ads", "Block search ads and self-promotion", "Blocks ads in search results and self-promotion of websites", true, "", false),
                sub("language_ads", "Language-specific ad blocking", "Blocks ads on websites in specified languages", true, "", true),
                sub("user_rules", "User rules", "Contains manually added rules and rules created via filtering log", true, "", true),
            ],
        },
        ModuleData {
            key: "tracking",
            icon: "feat_tracking.svg",
            title: "Tracking protection",
            desc: "Protects your online activity and personal data from web trackers",
            active: true,
            has_switch: false,
            sub_items: vec![
                sub("stealth", "Stealth Mode", "Protects against online surveillance by blocking tracking cookies and web analytics", true, "Without Stealth Mode, web trackers can profile your online activity", false),
                sub("hide_search", "Hide search queries", "Prevents search engines from saving and tracking your queries", true, "", false),
                sub("dnt", "Send Do-Not-Track headers", "Sends Global Privacy Control and Do-Not-Track signals to websites", true, "", false),
                sub("third_party_cookies", "Self-destruction of third-party cookies", "Destroys third-party cookies after a specified time limit", true, "", false),
            ],
        },
        ModuleData {
            key: "annoyance",
            icon: "feat_annoyance.svg",
            title: "Annoyance blocking",
            desc: "Blocks annoying web elements such as cookie notices, social widgets, or in-page popups",
            active: false,
            has_switch: false,
            sub_items: vec![
                sub("cookie_notices", "Block cookie notices", "Blocks cookie consent forms on websites", false, "", false),
                sub("popups", "Block popups", "Blocks annoying popups that are unnecessary for website functionality", false, "", false),
                sub("mobile_banners", "Block mobile app banners", "Blocks irritating banners that promote websites’ mobile apps", false, "", false),
                sub("widgets", "Block widgets", "Blocks third-party widgets including online assistants and live support chats", false, "", false),
                sub("other_annoyances", "Block other annoyances", "Blocks unusual annoying web elements", false, "", false),
            ],
        },
        ModuleData {
            key: "dns",
            icon: "feat_dns.svg",
            title: "DNS protection",
            desc: "Protects you from ads, trackers, and threats at the DNS level",
            active: true,
            has_switch: false,
            sub_items: vec![
                sub("dns_server", "DNS server", "Allows to select a DNS server to resolve DNS requests, block ads and trackers, and encrypt DNS traffic", true, "", false),
                sub("dns_filters", "DNS filters", "Allows to add DNS filters and user rules to block ads and trackers at the domain level", true, "", false),
                sub("dns_user_rules", "DNS user rules", "Block, unblock, or redirect domains with your user rules", true, "", false),
            ],
        },
        ModuleData {
            key: "browsing",
            icon: "feat_globe.svg",
            title: "Browsing security",
            desc: "Blocks requests to malicious and phishing websites",
            active: true,
            has_switch: false,
            sub_items: vec![
                sub("malware", "Malicious websites filter", "Blocks requests to known malware distribution domains", true, "Without Malicious websites filter, browsing safety is reduced", false),
                sub("phishing", "Phishing protection", "Protects against fake login forms and credential harvesting sites", true, "", false),
                sub("telemetry", "Help improve browsing security", "Anonymously share malicious domain reports with AdGuard", false, "", false),
            ],
        },
        ModuleData {
            key: "extensions",
            icon: "feat_extensions.svg",
            title: "Extensions",
            desc: "Allows adding userscripts and userstyles to extend AdGuard Ad Blocker's functionality and improve your web experience",
            active: true,
            has_switch: true,
            sub_items: vec![
                sub("adguard_extra", "AdGuard Extra", "Blocks sophisticated ad insertion techniques on challenging websites", true, "", false),
                sub("popup_blocker", "AdGuard Popup Blocker", "Blocks unwanted popup windows before they load", true, "", false),
                sub("userscripts", "Userscripts manager", "Enables custom userscript scripts from GreasyFork / GitHub", true, "", false),
            ],
        },
        ModuleData {
            key: "vpn",
            icon: "feat_vpn.svg",
            title: "VPN",
            desc: "AdGuard VPN installed",
            active: true,
            has_switch: false,
            sub_items: vec![],
        },
        ModuleData {
            key: "assistant",
            icon: "feat_assistant.svg",
            title: "AdGuard Assistant",
            desc: "Allows managing protection from your browser",
            active: true,
            has_switch: false,
            sub_items: vec![],
        },
    ]
}

pub fn annoyance_filter_ids() -> Vec<i64> {
    vec![18, 19, 20, 22, 21]
}

// Port of ANNOYANCE_FILTER_IDS mapping (sub_key -> filter_id).
pub fn annoyance_filter_id(sub_key: &str) -> Option<i64> {
    match sub_key {
        "cookie_notices" => Some(18),
        "popups" => Some(19),
        "mobile_banners" => Some(20),
        "widgets" => Some(22),
        "other_annoyances" => Some(21),
        _ => None,
    }
}

pub fn annoyance_filter_ids_map() -> Vec<(&'static str, i64)> {
    vec![
        ("cookie_notices", 18),
        ("popups", 19),
        ("mobile_banners", 20),
        ("widgets", 22),
        ("other_annoyances", 21),
    ]
}


fn mod_map(key: &str) -> (&'static str, &'static str) {
    match key {
        "ad_blocking" => ("feat_ad_blocking", "feat_ad_blocking_desc"),
        "tracking" => ("feat_tracking", "feat_tracking_desc"),
        "annoyance" => ("feat_annoyance", "feat_annoyance_desc"),
        "dns" => ("feat_dns", "feat_dns_desc"),
        "browsing" => ("feat_browsing", "feat_browsing_desc"),
        "extensions" => ("feat_extensions", "feat_extensions_desc"),
        "vpn" => ("feat_vpn", "feat_vpn_desc"),
        "assistant" => ("feat_assistant", "feat_assistant_desc"),
        _ => ("", ""),
    }
}

fn sub_map(key: &str) -> (&'static str, &'static str, &'static str) {
    match key {
        "basic" => ("sub_basic_t", "sub_basic_d", "sub_basic_w"),
        "search_ads" => ("sub_search_ads_t", "sub_search_ads_d", ""),
        "language_ads" => ("sub_lang_ads_t", "sub_lang_ads_d", ""),
        "user_rules" => ("sub_user_rules_t", "sub_user_rules_d", ""),
        "stealth" => ("sub_stealth_t", "sub_stealth_d", "sub_stealth_w"),
        "hide_search" => ("sub_hide_search_t", "sub_hide_search_d", ""),
        "dnt" => ("sub_dnt_t", "sub_dnt_d", ""),
        "third_party_cookies" => ("sub_tp_cookies_t", "sub_tp_cookies_d", ""),
        "cookie_notices" => ("sub_cookie_notices_t", "sub_cookie_notices_d", ""),
        "popups" => ("sub_popups_t", "sub_popups_d", ""),
        "mobile_banners" => ("sub_mobile_banners_t", "sub_mobile_banners_d", ""),
        "widgets" => ("sub_widgets_t", "sub_widgets_d", ""),
        "other_annoyances" => ("sub_other_annoyances_t", "sub_other_annoyances_d", ""),
        "dns_server" => ("sub_dns_server_t", "sub_dns_server_d", ""),
        "dns_filters" => ("sub_dns_filters_t", "sub_dns_filters_d", ""),
        "dns_user_rules" => ("sub_dns_user_rules_t", "sub_dns_user_rules_d", ""),
        "malware" => ("sub_malware_t", "sub_malware_d", "sub_malware_w"),
        "phishing" => ("sub_phishing_t", "sub_phishing_d", ""),
        "telemetry" => ("sub_telemetry_t", "sub_telemetry_d", ""),
        "adguard_extra" => ("sub_adguard_extra_t", "sub_adguard_extra_d", ""),
        "popup_blocker" => ("sub_popup_blocker_t", "sub_popup_blocker_d", ""),
        "userscripts" => ("sub_userscripts_t", "sub_userscripts_d", ""),
        _ => ("", "", ""),
    }
}

/// Port of get_module_translated_data(): (title, desc, sub_items) in the active language.
pub fn get_module_translated_data(key: &str) -> (String, String, Vec<(String, String, String, bool, String, bool)>) {
    let raw = match protection_modules().into_iter().find(|m| m.key == key) {
        Some(m) => m,
        None => ModuleData {
            key: "",
            icon: "",
            title: "",
            desc: "",
            active: false,
            has_switch: false,
            sub_items: vec![],
        },
    };
    let (t_key, d_key) = mod_map(key);
    let title = if !t_key.is_empty() { i18n::tr_default(t_key, raw.title) } else { raw.title.to_string() };
    let desc = if !d_key.is_empty() { i18n::tr_default(d_key, raw.desc) } else { raw.desc.to_string() };

    let mut sub_items = Vec::new();
    for item in &raw.sub_items {
        let (st_k, sd_k, sw_k) = sub_map(item.key);
        let s_title = if !st_k.is_empty() { i18n::tr_default(st_k, item.title) } else { item.title.to_string() };
        let s_desc = if !sd_k.is_empty() { i18n::tr_default(sd_k, item.desc) } else { item.desc.to_string() };
        let s_warn = if !sw_k.is_empty() { i18n::tr_default(sw_k, item.warning) } else { item.warning.to_string() };
        sub_items.push((item.key.to_string(), s_title, s_desc, item.active, s_warn, item.has_page));
    }

    (title, desc, sub_items)
}

/// Port of get_active_filter_ids(): direct SQLite read, CLI fallback.
pub fn get_active_filter_ids() -> HashSet<i64> {
    let db_path = c::home_dir().join(".local/share/adguard-cli/agflm_standard.db");
    if db_path.exists() {
        if let Ok(conn) = Connection::open(&db_path) {
            let mut stmt = match conn.prepare("SELECT filter_id FROM filter WHERE is_installed = 1 AND is_enabled = 1") {
                Ok(s) => s,
                Err(_) => return fallback_cli(),
            };
            let rows = stmt.query_map([], |r| r.get::<_, i64>(0));
            if let Ok(rows) = rows {
                let mut set = HashSet::new();
                for r in rows.flatten() {
                    set.insert(r);
                }
                return set;
            }
        }
    }
    fallback_cli()
}

fn fallback_cli() -> HashSet<i64> {
    let home_bin = c::home_dir().join(".local/bin/adguard-cli");
    let bin = if home_bin.exists() { home_bin } else { std::path::PathBuf::from("adguard-cli") };
    let out = match Command::new(&bin).arg("filters").arg("list").output() {
        Ok(o) => o,
        Err(_) => return HashSet::new(),
    };
    let re = Regex::new(r"\s*\[(x| )\]\s*\|\s*([0-9]+)\s*\|").unwrap();
    let mut active = HashSet::new();
    for line in String::from_utf8_lossy(&out.stdout).split('\n') {
        if let Some(m) = re.captures(line) {
            if m.get(1).map(|g| g.as_str() == "x").unwrap_or(false) {
                if let Ok(id) = m.get(2).unwrap().as_str().parse::<i64>() {
                    active.insert(id);
                }
            }
        }
    }
    active
}

pub struct AntiDpi {
    pub enabled: bool,
    pub client_hello: i64,
    pub http_split: i64,
    pub space_juggling: bool,
}

pub struct StealthParams {
    pub third_party_cookies: String,
    pub first_party_cookies: String,
    pub referer: String,
    pub user_agent: String,
    pub mask_ip: String,
}

pub struct ProtectionStatus {
    pub ad_blocking: bool,
    pub tracking: bool,
    pub annoyance: bool,
    pub dns: bool,
    pub browsing: bool,
    pub parental: bool,
    pub extensions: bool,
    pub anti_dpi: AntiDpi,
    pub params: StealthParams,
    pub active_filter_ids: HashSet<i64>,
}

/// Port of read_machine_protection_status(): proxy.yaml + filter db.
pub fn read_machine_protection_status() -> ProtectionStatus {
    let active_ids = get_active_filter_ids();
    let annoyance_active = annoyance_filter_ids().iter().any(|fid| active_ids.contains(fid));

    let mut status = ProtectionStatus {
        ad_blocking: true,
        tracking: true,
        annoyance: annoyance_active,
        dns: true,
        browsing: true,
        parental: false,
        extensions: true,
        anti_dpi: AntiDpi {
            enabled: true,
            client_hello: 1,
            http_split: 1,
            space_juggling: true,
        },
        params: StealthParams {
            third_party_cookies: "180".to_string(),
            first_party_cookies: "4320".to_string(),
            referer: String::new(),
            user_agent: String::new(),
            mask_ip: "208.54.104.184".to_string(),
        },
        active_filter_ids: active_ids,
    };

    let cfg_path = c::home_dir().join(".local/share/adguard-cli/proxy.yaml");
    if cfg_path.exists() {
        let content = match std::fs::read_to_string(&cfg_path) {
            Ok(s) => s,
            Err(_) => return status,
        };
        let re = Regex::new(r"^\s*([a-zA-Z0-9_-]+):\s*(.*)").unwrap();
        let mut curr_sec: Option<String> = None;
        let mut curr_sub: Option<String> = None;

        for line in content.lines() {
            let clean = line.split('#').next().unwrap_or("").trim_end();
            if clean.trim().is_empty() {
                continue;
            }
            let indent = clean.len() - clean.trim_start().len();
            let caps = match re.captures(clean) {
                Some(c) => c,
                None => continue,
            };
            let k = caps.get(1).unwrap().as_str().to_string();
            let v = caps.get(2).unwrap().as_str().trim().to_string();
            let val: Option<String> = if v.to_lowercase() == "true" {
                Some("true".to_string())
            } else if v.to_lowercase() == "false" {
                Some("false".to_string())
            } else if !v.is_empty() && v.chars().all(|ch| ch.is_ascii_digit()) {
                Some(v.clone())
            } else if !v.is_empty() {
                Some(v.trim_matches(|c| c == '\'' || c == '"').to_string())
            } else {
                None
            };

            let as_bool = |s: &str| s == "true";
            let as_int = |s: &str| s.parse::<i64>().unwrap_or(0);

            if indent == 0 {
                curr_sec = Some(k.clone());
                curr_sub = None;
                if k == "ad_blocking_enabled" {
                    if let Some(v) = &val {
                        status.ad_blocking = as_bool(v);
                    }
                }
            } else if indent == 2 {
                curr_sub = Some(k.clone());
                match curr_sec.as_deref() {
                    Some("stealthmode") => {
                        if let Some(v) = &val {
                            match k.as_str() {
                                "enabled" => status.tracking = as_bool(v),
                                "block_third_party_cookies_min" => status.params.third_party_cookies = v.clone(),
                                "block_first_party_cookies_min" => status.params.first_party_cookies = v.clone(),
                                "custom_referrer" => status.params.referer = v.clone(),
                                "custom_user_agent" => status.params.user_agent = v.clone(),
                                "custom_ip" => status.params.mask_ip = v.clone(),
                                _ => {}
                            }
                        }
                    }
                    Some("dns_filtering") => {
                        if k == "enabled" {
                            if let Some(v) = &val {
                                status.dns = as_bool(v);
                            }
                        }
                    }
                    Some("safebrowsing") => {
                        if k == "enabled" {
                            if let Some(v) = &val {
                                status.browsing = as_bool(v);
                            }
                        }
                    }
                    _ => {}
                }
            } else if indent == 4 {
                if curr_sec.as_deref() == Some("stealthmode") && curr_sub.as_deref() == Some("anti_dpi") {
                    if let Some(v) = &val {
                        match k.as_str() {
                            "enabled" => status.anti_dpi.enabled = as_bool(v),
                            "client_hello_split_fragment_size" => status.anti_dpi.client_hello = as_int(v),
                            "http_split_fragment_size" => status.anti_dpi.http_split = as_int(v),
                            "http_space_juggling" => status.anti_dpi.space_juggling = as_bool(v),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    status
}
