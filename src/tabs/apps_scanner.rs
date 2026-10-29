// Port of adguard_modern/tabs/apps_scanner.py
use std::collections::HashSet;
use std::path::PathBuf;

use crate::tabs::apps_config::get_app_action_from_config;

const BROWSER_KEYS: [&str; 12] = [
    "brave",
    "firefox",
    "google-chrome",
    "chromium",
    "mullvad",
    "vivaldi",
    "opera",
    "librewolf",
    "tor-browser",
    "waterfox",
    "epiphany",
    "falkon",
];

#[derive(Clone)]
pub struct AppEntry {
    pub name: String,
    pub icon: String,
    pub exec: String,
    pub file: PathBuf,
    pub is_browser: bool,
    pub route: bool,
    pub filter: bool,
    pub https: bool,
}

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// Scans real installed applications on this Linux system.
/// Returns (browsers, other_apps, system_apps).
pub fn scan_real_system_apps() -> (Vec<AppEntry>, Vec<AppEntry>, Vec<AppEntry>) {
    let search_dirs = [
        PathBuf::from("/usr/share/applications"),
        home_dir().join(".local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    ];

    let mut browsers: Vec<AppEntry> = Vec::new();
    let mut other_apps: Vec<AppEntry> = Vec::new();
    let mut system_apps: Vec<AppEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for app_dir in search_dirs {
        if !app_dir.exists() {
            continue;
        }

        // glob("*.desktop") sorted by path.
        let mut dpaths: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&app_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "desktop").unwrap_or(false) {
                    dpaths.push(p);
                }
            }
        }
        dpaths.sort();

        for dpath in dpaths {
            let content = match std::fs::read_to_string(&dpath) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut name: Option<String> = None;
            let mut icon: Option<String> = None;
            let mut exec_bin: Option<String> = None;
            let mut categories = String::new();
            let mut nodisplay = false;

            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("Name=") && name.is_none() {
                    name = Some(line.splitn(2, '=').nth(1).unwrap_or("").to_string());
                } else if line.starts_with("Icon=") && icon.is_none() {
                    icon = Some(line.splitn(2, '=').nth(1).unwrap_or("").to_string());
                } else if line.starts_with("Exec=") && exec_bin.is_none() {
                    let parts: Vec<&str> = line.splitn(2, '=').nth(1).unwrap_or("").split_whitespace().collect();
                    if let Some(first) = parts.first() {
                        exec_bin = Some(
                            PathBuf::from(first)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| first.to_string()),
                        );
                    }
                } else if line.starts_with("Categories=") {
                    categories = line.splitn(2, '=').nth(1).unwrap_or("").to_string();
                } else if line == "NoDisplay=true" {
                    nodisplay = true;
                }
            }

            let name = match name {
                Some(n) => n,
                None => continue,
            };
            if nodisplay || seen.contains(&name) {
                continue;
            }
            seen.insert(name.clone());

            // Check if pure browser (exclude web app shortcuts like PWA).
            let exec_lower = exec_bin.as_deref().unwrap_or("").to_lowercase();
            let check_str = format!(
                "{} {} {}",
                name.to_lowercase(),
                exec_lower,
                dpath.to_string_lossy().to_lowercase()
            );
            let mut is_browser = false;
            for b_key in BROWSER_KEYS {
                if (check_str.contains(b_key) || exec_lower.contains(b_key))
                    && !name.to_lowercase().contains("anime")
                    && !name.to_lowercase().contains("proton")
                {
                    is_browser = true;
                    break;
                }
            }

            // Read actual configured action from proxy.yaml or browsers.yaml.
            let action = get_app_action_from_config(exec_bin.as_deref().unwrap_or(""), is_browser);
            let route_st = action != "bypass";
            let filter_st = action != "bypass";
            let https_st = action == "default";

            let entry = AppEntry {
                name,
                icon: icon.unwrap_or_else(|| "apps".to_string()),
                exec: exec_bin.unwrap_or_default(),
                file: dpath,
                is_browser,
                route: route_st,
                filter: filter_st,
                https: https_st,
            };

            if entry.is_browser {
                browsers.push(entry);
            } else if ["Network", "AudioVideo", "Game", "Development", "Office"]
                .iter()
                .any(|cat| categories.contains(cat))
            {
                other_apps.push(entry);
            } else if ["System", "Settings", "Utility"]
                .iter()
                .any(|cat| categories.contains(cat))
            {
                system_apps.push(entry);
            }
        }
    }

    (browsers, other_apps, system_apps)
}
