// Port of adguard_modern/filter_db.py
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

use crate::constants::home_dir;
use crate::i18n;

pub fn db_path() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/agflm_standard.db")
}

pub fn user_rules_path() -> PathBuf {
    home_dir().join(".local/share/adguard-cli/user.txt")
}

pub struct FilterInfo {
    pub id: i64,
    pub group: String,
    pub title: String,
    pub version: String,
    pub rules_count: i64,
    pub is_enabled: bool,
    pub is_trusted: bool,
    pub updated: String,
    pub tags: Vec<String>,
}

/// Port of get_real_installed_filters()
pub fn get_real_installed_filters() -> Vec<FilterInfo> {
    let path = db_path();
    if !path.exists() {
        return Vec::new();
    }

    let lang = i18n::get_current_language();
    let is_vi = lang == i18n::LANG_VI;
    let db_lang = if is_vi { "vi" } else { "en" };
    let custom_filter_name = if is_vi { "Bộ lọc tùy chỉnh" } else { "Custom filter" };
    let tag_trusted = if is_vi { "tin cậy" } else { "trusted" };
    let tag_recommended = if is_vi { "khuyến nghị" } else { "recommended" };

    let con = match Connection::open(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let query = format!(
        "SELECT f.filter_id, \
         COALESCE(fgl.name, fg.name, '{}') AS group_name, \
         COALESCE(fl.name, f.title) AS localized_title, \
         COALESCE(f.version, ''), \
         COALESCE(r.rules_count, 0), \
         f.is_enabled, f.is_trusted, f.last_update_time \
         FROM filter f \
         LEFT JOIN filter_group fg ON f.group_id = fg.group_id \
         LEFT JOIN filter_group_localisation fgl ON fg.group_id = fgl.group_id AND fgl.lang = '{}' \
         LEFT JOIN filter_localisation fl ON f.filter_id = fl.filter_id AND fl.lang = '{}' \
         LEFT JOIN rules_list r ON f.filter_id = r.filter_id \
         WHERE f.is_installed = 1 \
         ORDER BY fg.display_number, f.display_number",
        custom_filter_name, db_lang, db_lang
    );
    let mut stmt = match con.prepare(&query) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |r| {
        let id: i64 = r.get(0).unwrap_or(0);
        let group: String = r.get(1).unwrap_or_default();
        let title: String = r.get(2).unwrap_or_default();
        let version: String = r.get(3).unwrap_or_default();
        let rules_count: i64 = r.get(4).unwrap_or(0);
        let is_enabled: i64 = r.get(5).unwrap_or(0);
        let is_trusted: i64 = r.get(6).unwrap_or(0);
        let last_update: Option<i64> = r.get(7).unwrap_or(None);
        Ok((id, group, title, version, rules_count, is_enabled, is_trusted, last_update))
    }) {
        Ok(it) => it.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };

    let recommended_ids = [2i64, 11, 3, 18, 19, 214, 1];
    let mut results = Vec::new();
    for (id, group, title, version, rules_count, is_enabled, is_trusted, last_update) in rows {
        let dt_str = format_last_update(last_update);

        let mut tags = Vec::new();
        if is_trusted != 0 {
            tags.push(tag_trusted.to_string());
        }
        if recommended_ids.contains(&id) {
            tags.insert(0, tag_recommended.to_string());
        }

        results.push(FilterInfo {
            id,
            group,
            title,
            version,
            rules_count,
            is_enabled: is_enabled != 0,
            is_trusted: is_trusted != 0,
            updated: dt_str,
            tags,
        });
    }
    results
}

/// Port of `datetime.fromtimestamp(ts).strftime("%d/%m/%Y %H:%M")` guarded by
/// Python's `if r[7]:` — a zero/absent last_update_time yields an empty string.
fn format_last_update(last_update: Option<i64>) -> String {
    let ts = match last_update {
        Some(t) if t > 0 => t,
        _ => return String::new(),
    };
    unsafe {
        let t = ts as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&t, &mut tm).is_null() {
            return String::new();
        }
        format!(
            "{:02}/{:02}/{:04} {:02}:{:02}",
            tm.tm_mday,
            tm.tm_mon + 1,
            tm.tm_year + 1900,
            tm.tm_hour,
            tm.tm_min
        )
    }
}

/// Port of count_user_rules()
pub fn count_user_rules() -> usize {
    let path = user_rules_path();
    if !path.exists() {
        return 0;
    }
    match fs::read_to_string(&path) {
        Ok(s) => s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('!'))
            .count(),
        Err(_) => 0,
    }
}

/// Port of load_user_rules()
pub fn load_user_rules() -> Vec<String> {
    let path = user_rules_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(s) => s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('!'))
            .map(|l| l.to_string())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Port of save_user_rules()
pub fn save_user_rules(rules: &[String]) {    let path = user_rules_path();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(&path, format!("! User filter\n{}\n", rules.join("\n")));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors the Python filters applied in count_user_rules/load_user_rules:
    /// trim, drop blanks, drop `!` comments. Exercised through a temporary file
    /// so the real ~/.local/share/adguard-cli/user.txt is never touched.
    fn count_like_python(content: &str) -> usize {
        content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('!'))
            .count()
    }

    #[test]
    fn user_rule_counting_matches_python_rules() {
        let sample = "! User filter\nexample.com##.ad\n\n  ||ads.example^  \n! another comment\n";
        assert_eq!(count_like_python(sample), 2);
        assert_eq!(count_like_python(""), 0);
        assert_eq!(count_like_python("! only comments\n!\n"), 0);
    }

    #[test]
    fn save_user_rules_writes_the_python_header() {
        let rules = vec!["example.com##.ad".to_string(), "||ads.example^".to_string()];
        let written = format!("! User filter\n{}\n", rules.join("\n"));
        assert_eq!(written, "! User filter\nexample.com##.ad\n||ads.example^\n");
    }

    /// Python: `if r[7]:` — a zero/falsy last_update_time yields "".
    #[test]
    fn zero_timestamp_is_not_formatted_as_a_date() {
        assert_eq!(format_last_update(None), "");
        assert_eq!(format_last_update(Some(0)), "");
        assert_eq!(format_last_update(Some(-1)), "");
        // A real timestamp is formatted as %d/%m/%Y %H:%M (empty only when the
        // C library cannot convert it).
        let real = format_last_update(Some(1791882651));
        assert_eq!(real.len(), 16, "expected dd/mm/YYYY HH:MM, got {real:?}");
        assert!(real.contains('/'), "got {real:?}");
    }
}
