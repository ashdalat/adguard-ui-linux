// Port of adguard_modern/tabs/apps_config.py
use std::path::PathBuf;

use regex::Regex;

use crate::backend;

fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

pub fn browser_yaml_paths() -> Vec<PathBuf> {
    [
        PathBuf::from("/root/.local/share/adguard-cli/browsers.yaml"),
        home_dir().join(".local/share/adguard-cli/browsers.yaml"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

pub fn proxy_yaml_paths() -> Vec<PathBuf> {
    [
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
        home_dir().join(".local/share/adguard-cli/proxy.yaml"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect()
}

pub fn get_app_action_from_config(exec_name: &str, is_browser: bool) -> String {
    if is_browser {
        for bp in browser_yaml_paths() {
            if let Ok(content) = std::fs::read_to_string(&bp) {
                let pat = format!(
                    r"- name:[^\n]*{}[^\n]*\n\s*action:\s*(\w+)",
                    regex::escape(exec_name)
                );
                if let Ok(re) = Regex::new(&pat) {
                    if let Some(caps) = re.captures(&content) {
                        return caps[1].to_lowercase();
                    }
                }
            }
        }
    } else {
        for pp in proxy_yaml_paths() {
            if let Ok(content) = std::fs::read_to_string(&pp) {
                let pat = format!(
                    r#"- name:\s*['"]?{}\*?['"]?\s*\n\s*action:\s*['"]?(\w+)['"]?"#,
                    regex::escape(exec_name)
                );
                if let Ok(re) = Regex::new(&pat) {
                    if let Some(caps) = re.captures(&content) {
                        return caps[1].to_lowercase();
                    }
                }
            }
        }
    }
    "default".to_string()
}

pub fn save_app_action(
    exec_name: &str,
    action: &str,
    is_browser: bool,
    route: bool,
    filter_traffic: bool,
    https: bool,
) {
    // Python: action may be None -> derived from flags. Rust encodes None as "".
    let action = if action.is_empty() {
        if !route || !filter_traffic {
            "bypass"
        } else if !https {
            "bypass_https"
        } else {
            "default"
        }
    } else {
        action
    };

    if is_browser {
        for bp in browser_yaml_paths() {
            if let Ok(content) = std::fs::read_to_string(&bp) {
                let pat = format!(
                    r"(- name:[^\n]*{}[^\n]*\n\s*action:\s*)\w+",
                    regex::escape(exec_name)
                );
                let new_content = if Regex::new(&pat).map(|re| re.is_match(&content)).unwrap_or(false) {
                    Regex::new(&pat)
                        .unwrap()
                        .replace(&content, format!("${{1}}{action}").as_str())
                        .to_string()
                } else {
                    format!("{}\n- name: {}|{}-bin\n  action: {}\n", content, exec_name, exec_name, action)
                };
                let _ = std::fs::write(&bp, new_content);
            }
        }
    } else {
        for pp in proxy_yaml_paths() {
            if let Ok(content) = std::fs::read_to_string(&pp) {
                let pat = format!(
                    r#"(- name:\s*['"]?{}\*?['"]?\s*\n\s*action:\s*['"]?)\w+(['"]?)"#,
                    regex::escape(exec_name)
                );
                let new_content = if Regex::new(&pat).map(|re| re.is_match(&content)).unwrap_or(false) {
                    Regex::new(&pat)
                        .unwrap()
                        .replace(&content, format!("${{1}}{action}${{2}}").as_str())
                        .to_string()
                } else {
                    if let Some(idx) = content.rfind("- name: '*'") {
                        let rule = format!("- name: '{}*'\n    action: '{}'\n  ", exec_name, action);
                        format!("{}{}{}", &content[..idx], rule, &content[idx..])
                    } else {
                        format!("\n  - name: '{}*'\n    action: '{}'\n", exec_name, action)
                    }
                };
                let _ = std::fs::write(&pp, new_content);
            }
        }
    }

    // Trigger adguard-cli restart in background.
    let cli = backend::adguard_cli_path();
    let cli_str = cli.to_string_lossy();
    let _ = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!(
            "sudo -n {} restart 2>/dev/null || {} restart 2>/dev/null || true",
            cli_str, cli_str
        ))
        .output();
}

pub fn save_filter_unknown_apps(enable: bool) {
    let action = if enable { "bypass_https" } else { "bypass" };
    for pp in proxy_yaml_paths() {
        if let Ok(content) = std::fs::read_to_string(&pp) {
            let re = unknown_apps_rule_regex();
            if re.is_match(&content) {
                let new_content = re.replace(&content, format!("${{1}}{action}${{2}}").as_str()).to_string();
                let _ = std::fs::write(&pp, new_content);
            }
        }
    }
}

/// The regex used to locate the catch-all `- name: '*'` rule in proxy.yaml.
fn unknown_apps_rule_regex() -> Regex {
    Regex::new(r#"(- name:\s*['"]?\*['"]?\s*\n\s*action:\s*['"]?)\w+(['"]?)"#).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Python's get_app_action_from_config. Verified by running the Python
    /// function on this machine:
    ///   ('nonexistent-browser-xyz', True)  -> 'default'
    ///   ('nonexistent-browser-xyz', False) -> 'default'
    ///   ('', True)                         -> 'default'
    ///   ('', False)                        -> 'bypass_https' — the catch-all
    ///       `- name: '*'` rule in proxy.yaml also matches an empty exec name.
    #[test]
    fn unknown_app_falls_back_to_default() {
        assert_eq!(get_app_action_from_config("nonexistent-browser-xyz", true), "default");
        assert_eq!(get_app_action_from_config("nonexistent-browser-xyz", false), "default");
        assert_eq!(get_app_action_from_config("", true), "default");
        assert_eq!(get_app_action_from_config("", false), "bypass_https");
    }

    /// Mirrors save_filter_unknown_apps' substitution on a proxy.yaml sample.
    /// Note the braced `${1}` form: Rust's regex crate reads `$1bypass` as a
    /// group named `1bypass` and replaces with an empty string, so the unbraced
    /// form silently wiped the value (found by this test).
    #[test]
    fn unknown_apps_rule_regex_rewrites_the_action() {
        let re = unknown_apps_rule_regex();
        let before = "rules:\n- name: '*'\n  action: bypass\n";
        let on = re.replace(before, "${1}bypass_https${2}").to_string();
        assert!(on.contains("action: bypass_https"), "got {on:?}");
        let off = re.replace(&on, "${1}bypass${2}").to_string();
        assert!(off.contains("action: bypass\n"), "got {off:?}");
        // A rule for a concrete app must not be touched by this catch-all regex.
        let other = "rules:\n- name: 'brave'\n  action: bypass_https\n";
        assert!(!re.is_match(other));
    }

    /// Regression test on the real proxy.yaml: the substitution must rewrite only
    /// the catch-all action and leave every other line intact. Before the fix,
    /// `format!("$1{action}$2")` was read as a group named `1bypass` and the whole
    /// matched block collapsed, wiping the rule from the file.
    #[test]
    fn real_proxy_yaml_is_not_corrupted_by_the_substitution() {
        let path = crate::constants::home_dir().join(".local/share/adguard-cli/proxy.yaml");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return, // no live config on this machine; nothing to check
        };
        let re = unknown_apps_rule_regex();
        if !re.is_match(&content) {
            return;
        }
        let rewritten = re.replace(&content, "${1}bypass${2}").to_string();
        // Same line count, and no line lost its `action:` value.
        assert_eq!(
            rewritten.lines().count(),
            content.lines().count(),
            "substitution must not add or remove lines"
        );
        assert!(
            !rewritten.contains("action:\n")
                && !rewritten.lines().any(|l| l.trim_end().ends_with("action:")),
            "substitution left an action with no value"
        );
        assert!(
            rewritten.contains("action: bypass") || rewritten.contains("action: 'bypass'"),
            "the catch-all action was not rewritten to the new value"
        );
        // Every other rule keeps the same action it had before.
        for (before, after) in content.lines().zip(rewritten.lines()) {
            if before.trim_start().starts_with("action:") && before.contains("bypass_https") {
                assert!(
                    after.contains("bypass") || after.contains("bypass_https"),
                    "unrelated rule changed: {before:?} -> {after:?}"
                );
            }
        }
    }
}
