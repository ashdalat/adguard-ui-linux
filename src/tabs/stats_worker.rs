// Port of adguard_modern/tabs/stats_worker.py — log parsing worker.
// Python runs a QThread and emits stats_ready(dict); here the same work runs on a
// worker thread and the result is delivered through util::spawn_async.
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::constants as c;
use crate::util;

pub fn company_map() -> HashMap<&'static str, (&'static str, &'static str)> {
    HashMap::from([
        ("google", ("Google", "company_google.svg")),
        ("googlevideo", ("Google", "company_google.svg")),
        ("youtube", ("YouTube", "company_google.svg")),
        ("ytimg", ("YouTube", "company_google.svg")),
        ("ggpht", ("Google", "company_google.svg")),
        ("doubleclick", ("Google", "company_google.svg")),
        ("googleusercontent", ("Google", "company_google.svg")),
        ("gstatic", ("Google", "company_google.svg")),
        ("googleapis", ("Google", "company_google.svg")),
        ("google-analytics", ("Google", "company_google.svg")),
        ("gvt1", ("Google", "company_google.svg")),
        ("1e100", ("Google", "company_google.svg")),
        ("dns.google", ("Google", "company_google.svg")),
        ("brave", ("Brave Software", "company_brave.svg")),
        ("microsoft", ("Microsoft", "company_microsoft.svg")),
        ("msftconnecttest", ("Microsoft", "company_microsoft.svg")),
        ("windowsupdate", ("Microsoft", "company_microsoft.svg")),
        ("live", ("Microsoft", "company_microsoft.svg")),
        ("bing", ("Microsoft", "company_microsoft.svg")),
        ("office", ("Microsoft", "company_microsoft.svg")),
        ("skype", ("Microsoft", "company_microsoft.svg")),
        ("azure", ("Microsoft", "company_microsoft.svg")),
        ("adguard", ("AdGuard", "company_adguard.svg")),
        ("adtidy", ("AdGuard", "company_adguard.svg")),
        ("cloudflare", ("Cloudflare, Inc.", "company_cloudflare.svg")),
        ("cloudflare-dns", ("Cloudflare, Inc.", "company_cloudflare.svg")),
        ("digicert", ("Digicert", "company_digicert.svg")),
        ("amazon", ("Amazon", "company_amazon.svg")),
        ("amazonaws", ("Amazon", "company_amazon.svg")),
        ("aws", ("Amazon", "company_amazon.svg")),
        ("sentry", ("Sentry", "company_sentry.svg")),
        ("facebook", ("Meta", "company_meta.svg")),
        ("fbcdn", ("Meta", "company_meta.svg")),
        ("instagram", ("Meta", "company_meta.svg")),
        ("meta", ("Meta", "company_meta.svg")),
        ("whatsapp", ("Meta", "company_meta.svg")),
        ("apple", ("Apple", "company_apple.svg")),
        ("icloud", ("Apple", "company_apple.svg")),
        ("aaplimg", ("Apple", "company_apple.svg")),
        ("tiktok", ("ByteDance", "")),
        ("byteoversea", ("ByteDance", "")),
        ("bytedance", ("ByteDance", "")),
        ("github", ("GitHub", "")),
        ("githubusercontent", ("GitHub", "")),
        ("reddit", ("Reddit", "")),
        ("redditmedia", ("Reddit", "")),
        ("discord", ("Discord", "")),
        ("discordapp", ("Discord", "")),
        ("telegram", ("Telegram", "")),
        ("spotify", ("Spotify", "")),
        ("steam", ("Valve Corporation", "")),
        ("steampowered", ("Valve Corporation", "")),
        ("steamcommunity", ("Valve Corporation", "")),
        ("quad9", ("Quad9", "")),
        ("cachyos", ("CachyOS", "")),
        ("archlinux", ("Arch Linux", "")),
    ])
}

pub fn get_log_paths() -> Vec<PathBuf> {
    let root_dir = PathBuf::from("/root/.local/share/adguard-cli/logs");
    let user_dir = c::home_dir().join(".local/share/adguard-cli/logs");
    let target_dir = if root_dir.exists() && is_readable(&root_dir) {
        root_dir
    } else {
        user_dir
    };
    if !target_dir.exists() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = match fs::read_dir(&target_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                name.starts_with("access.log")
            })
            .collect(),
        Err(_) => return Vec::new(),
    };
    files.sort();
    files.retain(|p| is_readable(p));
    files
}

fn is_readable(path: &PathBuf) -> bool {
    fs::File::open(path).is_ok()
}

pub fn format_count(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn format_bytes(b: i64) -> String {
    if b >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.0} MB", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.0} KB", b as f64 / 1024.0)
    } else {
        format!("{} B", b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values are the output of the Python functions
    // `format_count` / `format_bytes` in adguard_modern/tabs/stats_worker.py.
    #[test]
    fn format_count_matches_python() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(1), "1");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1000), "1.0K");
        assert_eq!(format_count(1500), "1.5K");
        assert_eq!(format_count(12345), "12.3K");
        assert_eq!(format_count(999999), "1000.0K");
        assert_eq!(format_count(1000000), "1.0M");
        assert_eq!(format_count(1500000), "1.5M");
        assert_eq!(format_count(28700), "28.7K");
    }

    #[test]
    fn format_bytes_matches_python() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1 KB");
        assert_eq!(format_bytes(1536), "2 KB");
        assert_eq!(format_bytes(1048576), "1 MB");
        assert_eq!(format_bytes(1500000), "1 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
        assert_eq!(format_bytes(2500000000), "2.3 GB");
    }

    #[test]
    fn company_map_has_the_same_entries_as_python() {
        let m = company_map();
        assert_eq!(m.len(), 56, "COMPANY_MAP size must match Python (56)");
        assert_eq!(m.get("google"), Some(&("Google", "company_google.svg")));
        assert_eq!(m.get("youtube"), Some(&("YouTube", "company_google.svg")));
        assert_eq!(m.get("facebook"), Some(&("Meta", "company_meta.svg")));
        assert_eq!(m.get("amazon"), Some(&("Amazon", "company_amazon.svg")));
    }
}

#[derive(Clone)]
pub struct CompanyStat {
    pub name: String,
    pub ads: String,
    pub trackers: String,
    pub total: String,
    pub raw_ads: i64,
    pub raw_trackers: i64,
    pub raw_total: i64,
    pub icon: String,
    pub initial: String,
}

pub struct StatsResult {
    pub total_req: String,
    pub blocked_ads: String,
    pub blocked_trackers: String,
    pub dns_ads: String,
    pub dns_trackers: String,
    pub dns_total: String,
    pub saved_str: String,
    pub uploaded_str: String,
    pub downloaded_str: String,
    pub companies: Vec<CompanyStat>,
    pub all_companies: Vec<CompanyStat>,
}

fn empty_stats() -> StatsResult {
    StatsResult {
        total_req: "0".to_string(),
        blocked_ads: "0".to_string(),
        blocked_trackers: "0".to_string(),
        dns_ads: "0".to_string(),
        dns_trackers: "0".to_string(),
        dns_total: "0".to_string(),
        saved_str: "0 KB".to_string(),
        uploaded_str: "0 KB".to_string(),
        downloaded_str: "0 MB".to_string(),
        companies: Vec::new(),
        all_companies: Vec::new(),
    }
}

fn parse_log_time(date: &str, time: &str) -> Option<i64> {
    let d: Vec<&str> = date.split('.').collect();
    let t: Vec<&str> = time.split(':').collect();
    if d.len() != 3 || t.len() != 3 {
        return None;
    }
    let (day, month, year) = (d[0].parse::<i64>().ok()?, d[1].parse::<i64>().ok()?, d[2].parse::<i64>().ok()?);
    let (hour, min, sec) = (t[0].parse::<i64>().ok()?, t[1].parse::<i64>().ok()?, t[2].parse::<i64>().ok()?);
    // days_from_civil (Howard Hinnant) → naive local seconds, matching Python's
    // naive datetime.strptime comparison
    let y = if month < 3 { year - 1 } else { year };
    let m = ((month + 9) % 12) as i64;
    let dy = day - 1;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 12) / 5 + dy;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

/// Port of StatsWorker::run().
pub fn run(time_filter: &str, sort_by: &str) -> StatsResult {
    let log_files = get_log_paths();
    if log_files.is_empty() {
        return empty_stats();
    }

    let now = naive_local_now();
    let cutoff: Option<i64> = match time_filter {
        "24h" => Some(now - 24 * 3600),
        "7d" => Some(now - 7 * 86400),
        "14d" => Some(now - 14 * 86400),
        _ => None,
    };

    let mut total_req: i64 = 0;
    let mut blocked_ads: i64 = 0;
    let mut blocked_trackers: i64 = 0;
    let mut dns_ads: i64 = 0;
    let mut dns_trackers: i64 = 0;
    let mut dns_total: i64 = 0;
    let mut bytes_down: i64 = 0;

    let map = company_map();
    let mut companies: HashMap<String, (i64, i64, i64, String)> = HashMap::new();

    for log_path in &log_files {
        let content = match fs::read_to_string(log_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() < 6 {
                continue;
            }
            if parts.len() > 4 && parts[3] == "HTTP1" && parts[4] == "CONNECT" {
                continue;
            }

            if let Some(c) = cutoff {
                if parts.len() >= 2 {
                    if let Some(dt) = parse_log_time(parts[0], &parts[1][..8.min(parts[1].len())]) {
                        if dt < c {
                            continue;
                        }
                    }
                }
            }

            total_req += 1;

            for p in &parts {
                if p.ends_with('b') && p[..p.len() - 1].chars().all(|ch| ch.is_ascii_digit()) && !p[..p.len() - 1].is_empty() {
                    if let Ok(v) = p[..p.len() - 1].parse::<i64>() {
                        bytes_down += v;
                    }
                }
            }

            let is_blocked = line.contains("BLOCKED") || line.contains("ID=") || line.contains(" 502 ") || line.contains(" 403 ");
            let is_dns = parts[3].contains("DNS") || line.to_lowercase().contains("dns");
            if is_dns {
                dns_total += 1;
            }

            let mut raw_target = String::new();
            for p in &parts[3..] {
                if matches!(*p, "TLS" | "HTTP1" | "TCP" | "CONNECT" | "GET" | "POST" | "any" | "NONE" | "--" | "-") {
                    continue;
                }
                if p.ends_with("ms") || p.ends_with('b') || (!p.is_empty() && p.chars().all(|ch| ch.is_ascii_digit())) {
                    continue;
                }
                if (p.contains('.') || p.contains("http")) && !p.starts_with('[') {
                    raw_target = p.to_string();
                    break;
                }
            }

            let mut comp_name = "Other".to_string();
            let mut comp_icon = "company_generic.svg".to_string();
            if !raw_target.is_empty() {
                let mut clean_t = raw_target.split(':').next().unwrap_or("").to_lowercase();
                if clean_t.starts_with("http://") {
                    clean_t = clean_t[7..].to_string();
                } else if clean_t.starts_with("https://") {
                    clean_t = clean_t[8..].to_string();
                }
                clean_t = clean_t.split('/').next().unwrap_or("").to_string();

                let parts_d: Vec<&str> = clean_t.split('.').collect();
                if parts_d.len() >= 2 && parts_d[parts_d.len() - 1].chars().all(|ch| ch.is_ascii_digit()) && !parts_d[parts_d.len() - 1].is_empty() {
                    comp_name = "Direct IP Connection".to_string();
                    comp_icon = "company_generic.svg".to_string();
                } else {
                    let root_domain = if parts_d.len() >= 2 {
                        format!("{}.{}", parts_d[parts_d.len() - 2], parts_d[parts_d.len() - 1])
                    } else {
                        clean_t.clone()
                    };
                    let main_part = if parts_d.len() >= 2 { parts_d[parts_d.len() - 2].to_string() } else { clean_t.clone() };

                    if let Some((name, icon)) = map.get(main_part.as_str()) {
                        comp_name = name.to_string();
                        comp_icon = icon.to_string();
                    } else if let Some((name, icon)) = map.get(root_domain.as_str()) {
                        comp_name = name.to_string();
                        comp_icon = icon.to_string();
                    } else {
                        comp_name = if parts_d.len() >= 2 { root_domain } else { clean_t };
                        comp_icon = "company_generic.svg".to_string();
                    }
                }
            }

            let e = companies.entry(comp_name.clone()).or_insert((0, 0, 0, String::new()));
            e.2 += 1;
            e.3 = comp_icon.clone();

            if is_blocked {
                if line.contains("ID=3") || line.to_lowercase().contains("track") {
                    blocked_trackers += 1;
                    let e = companies.get_mut(&comp_name).unwrap();
                    e.1 += 1;
                    if is_dns {
                        dns_trackers += 1;
                    }
                } else {
                    blocked_ads += 1;
                    let e = companies.get_mut(&comp_name).unwrap();
                    e.0 += 1;
                    if is_dns {
                        dns_ads += 1;
                    }
                }
            }
        }
    }

    let mut entries: Vec<(String, i64, i64, i64, String)> = companies
        .into_iter()
        .map(|(name, (ads, trackers, total, icon))| (name, ads, trackers, total, icon))
        .collect();
    match sort_by {
        "trackers" => entries.sort_by(|a, b| (b.2, b.3).cmp(&(a.2, a.3))),
        "requests" => entries.sort_by(|a, b| b.3.cmp(&a.3)),
        _ => entries.sort_by(|a, b| (b.1, b.3).cmp(&(a.1, a.3))),
    }

    let bytes_saved = blocked_ads * 120 * 1024 + blocked_trackers * 30 * 1024;
    let bytes_uploaded = if bytes_down > 0 { bytes_down as f64 * 0.12 } else { 0.0 };

    let all_companies: Vec<CompanyStat> = entries
        .into_iter()
        .map(|(name, ads, trackers, total, icon)| CompanyStat {
            initial: name.chars().next().map(|ch| ch.to_uppercase().to_string()).unwrap_or_else(|| "•".to_string()),
            name,
            ads: format_count(ads),
            trackers: format_count(trackers),
            total: format_count(total),
            raw_ads: ads,
            raw_trackers: trackers,
            raw_total: total,
            icon,
        })
        .collect();

    StatsResult {
        total_req: format_count(total_req),
        blocked_ads: format_count(blocked_ads),
        blocked_trackers: format_count(blocked_trackers),
        dns_ads: format_count(dns_ads),
        dns_trackers: format_count(dns_trackers),
        dns_total: format_count(dns_total),
        saved_str: format_bytes(bytes_saved),
        uploaded_str: format_bytes(bytes_uploaded as i64),
        downloaded_str: format_bytes(bytes_down),
        companies: all_companies[..all_companies.len().min(5)].to_vec(),
        all_companies,
    }
}

fn naive_local_now() -> i64 {
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let t = epoch as libc::time_t;
    unsafe { libc::localtime_r(&t, &mut tm) };
    epoch + tm.tm_gmtoff
}

/// Python: `StatsWorker(time_filter, sort_by).start()` then `stats_ready` signal.
pub fn start(
    time_filter: &str,
    sort_by: &str,
    on_ready: Box<dyn Fn(StatsResult)>,
) {
    let tf = time_filter.to_string();
    let sb = sort_by.to_string();
    util::spawn_async(
        move || run(&tf, &sb),
        move |res| on_ready(res),
    );
}
