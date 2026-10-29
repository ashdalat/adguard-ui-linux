// Port of adguard_modern/i18n.py — English (en) and Vietnamese (vi).
// Language selection persisted in ~/.config/adguard-cachyos-gui/settings.json.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub const LANG_VI: &str = "vi";
pub const LANG_EN: &str = "en";
pub const DEFAULT_LANG: &str = LANG_EN;

pub fn settings_file() -> PathBuf {
    crate::constants::home_dir().join(".config/adguard-cachyos-gui/settings.json")
}

// TRANSLATIONS table, embedded from the export generated out of i18n.py.
// The JSON is language-first: {"en": {key: text}, "vi": {key: text}}. Python's
// TRANSLATIONS is the other way round (key -> {lang: text}), so invert it once
// here to keep the lookup semantics of the original `tr(key, default)`.
const DATA_JSON: &str = include_str!("i18n_data.json");

/// Raw JSON text of the translation table (used by tests to validate every key).
pub fn raw_data() -> &'static str {
    DATA_JSON
}

fn translations() -> &'static HashMap<String, HashMap<String, String>> {
    static TABLE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let by_lang: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(DATA_JSON).expect("i18n_data.json is not valid JSON");
        let mut by_key: HashMap<String, HashMap<String, String>> = HashMap::new();
        for (lang, entries) in by_lang {
            for (key, text) in entries {
                by_key.entry(key).or_default().insert(lang.clone(), text);
            }
        }
        by_key
    })
}

static CURRENT_LANG: Mutex<String> = Mutex::new(String::new());

// Callbacks hold non-Send widget handles, so they live on the GTK main thread.
thread_local! {
    static SUBSCRIBERS: std::cell::RefCell<Vec<Box<dyn Fn(&str)>>> = std::cell::RefCell::new(Vec::new());
}

// Port of I18nManager::_load_saved_language
fn load_saved_language() -> String {
    let path = settings_file();
    if path.exists() {
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(lang) = v.get("language").and_then(|l| l.as_str()) {
                    if lang == LANG_VI || lang == LANG_EN {
                        return lang.to_string();
                    }
                }
            }
        }
    }
    DEFAULT_LANG.to_string()
}

fn current_lang_locked(guard: &mut String) -> String {
    if guard.is_empty() {
        *guard = load_saved_language();
    }
    guard.clone()
}

pub fn get_current_language() -> String {
    current_lang_locked(&mut CURRENT_LANG.lock().unwrap())
}

// Port of I18nManager::set_language: validates, persists, emits language_changed.
pub fn set_language(lang: &str) {
    if lang != LANG_VI && lang != LANG_EN {
        return;
    }
    let changed = {
        let mut g = CURRENT_LANG.lock().unwrap();
        let cur = current_lang_locked(&mut g);
        if cur == lang {
            false
        } else {
            *g = lang.to_string();
            true
        }
    };
    if !changed {
        return;
    }

    let path = settings_file();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let mut settings: serde_json::Value = serde_json::Value::Object(serde_json::Map::new());
    if path.exists() {
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                settings = v;
            }
        }
    }
    if settings.is_object() {
        settings
            .as_object_mut()
            .unwrap()
            .insert("language".to_string(), serde_json::Value::String(lang.to_string()));
        match fs::write(&path, serde_json::to_string_pretty(&settings).unwrap()) {
            Ok(()) => {}
            Err(e) => println!("Error saving language setting: {}", e),
        }
    }

    // Take the callbacks out of the cell before invoking them so a callback can
    // call i18n functions (or re-register) without deadlocking.
    let subs = SUBSCRIBERS.with(|s| std::mem::take(&mut *s.borrow_mut()));
    for cb in &subs {
        cb(lang);
    }
    SUBSCRIBERS.with(|s| *s.borrow_mut() = subs);
}

// Port of get_i18n_signal().connect(...). Callbacks run on the GTK main loop
// because set_language is only called from UI code, so they need not be Send.
pub fn connect_language_changed(cb: Box<dyn Fn(&str)>) {
    SUBSCRIBERS.with(|s| s.borrow_mut().push(cb));
}

// Port of tr(key, default="")
pub fn tr(key: &str) -> String {
    tr_default(key, "")
}

pub fn tr_default(key: &str, default: &str) -> String {
    lookup(&get_current_language(), key, default)
}

/// Pure lookup, mirrors Python `tr`: `item.get(lang) or item.get(LANG_EN) or default or key`.
/// Split out from `tr_default` so it can be tested without touching settings.json.
pub fn lookup(lang: &str, key: &str, default: &str) -> String {
    let table = translations();
    let item = match table.get(key) {
        Some(i) => i,
        None => {
            return if !default.is_empty() {
                default.to_string()
            } else {
                key.to_string()
            }
        }
    };
    if let Some(v) = item.get(lang).filter(|s| !s.is_empty()) {
        return v.clone();
    }
    if let Some(v) = item.get(LANG_EN).filter(|s| !s.is_empty()) {
        return v.clone();
    }
    if !default.is_empty() {
        default.to_string()
    } else {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the regression fixed during the real runs: i18n_data.json is
    /// language-first while Python's TRANSLATIONS is key-first. If the table is
    /// not inverted at load time every label comes back as its raw key.
    #[test]
    fn translations_are_keyed_by_translation_key_not_by_language() {
        let t = translations();
        assert!(t.contains_key("nav_home"), "expected key-first table");
        assert!(!t.contains_key(LANG_EN), "table must not be keyed by language");
    }

    /// Expected strings come from Python: tr("nav_home") etc. in both languages.
    #[test]
    fn lookup_matches_python_for_nav_keys() {
        for (lang, expected) in [
            (LANG_EN, ["Home", "Protection", "Statistics", "App management", "Settings", "Check for updates", "Hide"]),
            (LANG_VI, ["Trang chủ", "Bảo vệ", "Thống kê", "Quản lý ứng dụng", "Cài đặt", "Kiểm tra cập nhật", "Ẩn"]),
        ] {
            let keys = ["nav_home", "nav_protection", "nav_statistics", "nav_apps", "nav_settings", "nav_updates", "nav_hide"];
            for (k, exp) in keys.iter().zip(expected) {
                assert_eq!(lookup(lang, k, ""), exp, "{lang}/{k}");
            }
        }
    }

    /// Python: `item.get(lang) or item.get(LANG_EN) or default or key`.
    #[test]
    fn lookup_fallback_matches_python() {
        assert_eq!(lookup(LANG_VI, "nonexistent_key_xyz", "DEFAULT"), "DEFAULT");
        assert_eq!(lookup(LANG_VI, "nonexistent_key_xyz", ""), "nonexistent_key_xyz");
        // Unknown language falls back to English, like Python's LANG_EN branch.
        assert_eq!(lookup("zz", "nav_home", ""), "Home");
    }

    #[test]
    fn both_languages_cover_the_same_keys() {
        let loaded: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
            serde_json::from_str(DATA_JSON).expect("i18n_data.json is not valid JSON");
        let en = loaded.get(LANG_EN).expect("en table");
        let vi = loaded.get(LANG_VI).expect("vi table");
        assert_eq!(en.len(), 535, "Python TRANSLATIONS has 535 keys");
        assert_eq!(vi.len(), 535);
        for k in en.keys() {
            assert!(vi.contains_key(k), "vi is missing key {k}");
        }
    }
}
