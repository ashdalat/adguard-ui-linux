// Port of adguard_modern/backend.py — system commands, proxy config, certificate
// install, status checks, filter DB, and the async worker/signals pattern.
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::constants::home_dir;
use crate::util;

pub static LAST_RUNNING: AtomicBool = AtomicBool::new(false);
static PROTECTION_BUSY: AtomicBool = AtomicBool::new(false);
static STATUS_BUSY: AtomicBool = AtomicBool::new(false);

type StatusCb = Box<dyn Fn(bool, &str) + Send + Sync>;
type ActionCb = Box<dyn Fn(bool, &str) + Send + Sync>;
type FilterCb = Box<dyn Fn(i64, bool, &str) + Send + Sync>;

static STATUS_CB: Mutex<Vec<StatusCb>> = Mutex::new(Vec::new());
static ACTION_CB: Mutex<Vec<ActionCb>> = Mutex::new(Vec::new());
static FILTER_CB: Mutex<Vec<FilterCb>> = Mutex::new(Vec::new());

pub fn connect_status_changed(cb: StatusCb) {
    STATUS_CB.lock().unwrap().push(cb);
}
pub fn connect_action_finished(cb: ActionCb) {
    ACTION_CB.lock().unwrap().push(cb);
}
pub fn connect_filter_toggled(cb: FilterCb) {
    FILTER_CB.lock().unwrap().push(cb);
}

pub fn is_running() -> bool {
    LAST_RUNNING.load(Ordering::SeqCst)
}

/// Port of the StatusWorker result used by the UI: returns the snapshot without
/// emitting the signal (the signal path is refresh_status()).
pub fn check_status() -> (bool, String) {
    status_snapshot()
}

/// Port of get_adguard_cli_path()
pub fn adguard_cli_path() -> PathBuf {
    let home = home_dir();
    let paths = [
        home.join(".local/bin/adguard-cli"),
        PathBuf::from("/usr/local/bin/adguard-cli"),
        PathBuf::from("/usr/bin/adguard-cli"),
    ];
    for p in paths {
        if p.is_file() && is_executable(&p) {
            return p;
        }
    }
    if let Some(w) = which("adguard-cli") {
        return w;
    }
    PathBuf::from("adguard-cli")
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match fs::metadata(p) {
        Ok(m) => m.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var("PATH").ok()?;
    for dir in path.split(':') {
        let cand = PathBuf::from(dir).join(name);
        if cand.is_file() && is_executable(&cand) {
            return Some(cand);
        }
    }
    None
}

fn glob_recursive(root: &Path, filename: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() && p.file_name().map(|n| n == filename).unwrap_or(false) {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}

/// Port of configure_browser_proxy()
pub fn configure_browser_proxy() {
    let home = home_dir();
    // 1. Gnome / Cinnamon system proxy
    for args in [
        ["gsettings", "set", "org.gnome.system.proxy", "mode", "manual"],
        ["gsettings", "set", "org.gnome.system.proxy.http", "host", "127.0.0.1"],
        ["gsettings", "set", "org.gnome.system.proxy.http", "port", "3129"],
        ["gsettings", "set", "org.gnome.system.proxy.https", "host", "127.0.0.1"],
        ["gsettings", "set", "org.gnome.system.proxy.https", "port", "3129"],
    ] {
        let _ = Command::new(args[0]).args(&args[1..]).output();
    }

    // 2. Brave & Chromium flags
    let config_dir = home.join(".config");
    for flag_file in ["brave-flags.conf", "chromium-flags.conf"] {
        let fpath = config_dir.join(flag_file);
        if let Ok(mut f) = fs::File::create(&fpath) {
            let _ = f.write_all(b"--proxy-server=http://127.0.0.1:3129\n--password-store=basic\n");
        }
    }

    // 3. Firefox prefs.js (skips Mullvad-only handling; kept stable with NoVPN)
    let browser_dirs = [home.join(".mozilla/firefox"), home.join(".mullvad/mullvadbrowser")];
    for b_dir in browser_dirs {
        if !b_dir.exists() {
            continue;
        }
        for prefs_path in glob_recursive(&b_dir, "prefs.js") {
            let raw = match fs::read_to_string(&prefs_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut filtered: Vec<String> = raw
                .lines()
                .map(|l| l.to_string())
                .filter(|l| {
                    !l.contains("network.proxy.")
                        && !l.contains("security.enterprise_roots.")
                        && !l.contains("security.cert_pinning.")
                })
                .collect();
            for p in [
                "user_pref(\"network.proxy.type\", 1);",
                "user_pref(\"network.proxy.http\", \"127.0.0.1\");",
                "user_pref(\"network.proxy.http_port\", 3129);",
                "user_pref(\"network.proxy.ssl\", \"127.0.0.1\");",
                "user_pref(\"network.proxy.ssl_port\", 3129);",
                "user_pref(\"network.proxy.share_proxy_settings\", true);",
                "user_pref(\"security.enterprise_roots.enabled\", true);",
                "user_pref(\"security.certerrors.mitm.auto_enable_enterprise_roots\", true);",
                "user_pref(\"security.cert_pinning.enforcement_level\", 0);",
            ] {
                filtered.push(p.to_string());
            }
            let text = filtered.iter().map(|l| format!("{}\n", l)).collect::<String>();
            let _ = fs::write(&prefs_path, text);

            // user.js in the same profile directory (permanent)
            let user_js_path = prefs_path.parent().unwrap().join("user.js");
            let mut u_lines: Vec<String> = Vec::new();
            if user_js_path.exists() {
                if let Ok(s) = fs::read_to_string(&user_js_path) {
                    u_lines = s
                        .lines()
                        .map(|l| l.to_string())
                        .filter(|l| {
                            !l.contains("security.cert_pinning.")
                                && !l.contains("security.enterprise_roots.")
                                && !l.contains("security.certerrors.")
                        })
                        .collect();
                }
            }
            for p in [
                "user_pref(\"security.enterprise_roots.enabled\", true);",
                "user_pref(\"security.certerrors.mitm.auto_enable_enterprise_roots\", true);",
                "user_pref(\"security.cert_pinning.enforcement_level\", 0);",
            ] {
                u_lines.push(p.to_string());
            }
            let text = u_lines.iter().map(|l| format!("{}\n", l)).collect::<String>();
            let _ = fs::write(&user_js_path, text);
        }
    }
}

/// Port of disable_browser_proxy()
pub fn disable_browser_proxy() {
    let home = home_dir();
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.system.proxy", "mode", "none"])
        .output();

    let config_dir = home.join(".config");
    for flag_file in ["brave-flags.conf", "chromium-flags.conf"] {
        let fpath = config_dir.join(flag_file);
        if let Ok(mut f) = fs::File::create(&fpath) {
            let _ = f.write_all(b"--password-store=basic\n");
        }
    }

    let browser_dirs = [home.join(".mozilla/firefox"), home.join(".mullvad/mullvadbrowser")];
    for b_dir in browser_dirs {
        if !b_dir.exists() {
            continue;
        }
        for prefs_path in glob_recursive(&b_dir, "prefs.js") {
            let raw = match fs::read_to_string(&prefs_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut filtered: Vec<String> = raw
                .lines()
                .map(|l| l.to_string())
                .filter(|l| !l.contains("network.proxy."))
                .collect();
            filtered.push("user_pref(\"network.proxy.type\", 0);".to_string());
            let text = filtered.iter().map(|l| format!("{}\n", l)).collect::<String>();
            let _ = fs::write(&prefs_path, text);
        }
    }
}

/// Port of install_certificate() — the exact bash command from Python.
pub fn install_certificate() -> (bool, String) {
    let home_path = home_dir();
    let home = home_path.display().to_string();
    let ca = home_path.join(".local/share/adguard-cli/AdGuard CLI CA.pem").display().to_string();
    let cmd = format!(
        "bash -c '[ -f \"{ca}\" ] || echo yes | adguard-cli cert || true' && \
certutil -d sql:{home}/.pki/nssdb -A -t \"CT,C,C\" -n \"AdGuard CLI CA\" -i \"{ca}\" 2>/dev/null || true ; \
for db in $(find {home} -name \"cert9.db\" 2>/dev/null); do \
  dir=$(dirname \"$db\") ; \
  certutil -d sql:\"$dir\" -A -t \"CT,C,C\" -n \"AdGuard CLI CA\" -i \"{ca}\" 2>/dev/null || true ; \
  certutil -d sql:\"$dir\" -M -t \"CT,C,C\" -n \"AdGuard CLI CA\" 2>/dev/null || true ; \
done ; \
sudo -n bash -c '\
rm -rf /etc/brave/policies/managed /etc/chromium/policies/managed /etc/opt/chrome/policies/managed && \
mkdir -p /etc/ca-certificates/trust-source/anchors /usr/local/share/ca-certificates /etc/firefox/policies /etc/mullvad-browser/policies /usr/lib/mullvad-browser/distribution/ /opt/mullvad-browser/distribution/ && \
cp -f \"{ca}\" /etc/ca-certificates/trust-source/anchors/AdGuard_CLI_CA.crt 2>/dev/null || true ; \
cp -f \"{ca}\" /usr/local/share/ca-certificates/AdGuard_CLI_CA.crt 2>/dev/null || true ; \
update-ca-trust 2>/dev/null || update-ca-certificates 2>/dev/null || true ; \
cat << \"EOF\" > /etc/firefox/policies/policies.json\n\
{{\n\
  \"policies\": {{\n\
    \"Certificates\": {{\n\
      \"ImportEnterpriseRoots\": true,\n\
      \"Install\": [\"/etc/ca-certificates/trust-source/anchors/AdGuard_CLI_CA.crt\", \"{ca}\"]\n\
    }},\n\
    \"Preferences\": {{\n\
      \"security.enterprise_roots.enabled\": true,\n\
      \"security.certerrors.mitm.auto_enable_enterprise_roots\": true,\n\
      \"security.cert_pinning.enforcement_level\": 0\n\
    }}\n\
  }}\n\
}}\n\
EOF\n\
cp -f /etc/firefox/policies/policies.json /etc/mullvad-browser/policies/ 2>/dev/null || true ; \
cp -f /etc/firefox/policies/policies.json /usr/lib/mullvad-browser/distribution/ 2>/dev/null || true ; \
cp -f /etc/firefox/policies/policies.json /opt/mullvad-browser/distribution/ 2>/dev/null || true ; \
for d in /opt/mullvad-browser/browser/defaults/preferences /usr/lib/mullvad-browser/browser/defaults/preferences /usr/lib/firefox/browser/defaults/preferences /usr/lib64/firefox/browser/defaults/preferences; do \
  if [ -d \"$d\" ]; then \
    cat << \"PREF_EOF\" > \"$d/99-adguard.js\"\n\
pref(\"security.enterprise_roots.enabled\", true);\n\
pref(\"security.certerrors.mitm.auto_enable_enterprise_roots\", true);\n\
pref(\"security.cert_pinning.enforcement_level\", 0);\n\
PREF_EOF\n\
  fi ; \
done ; \
'"
    );
    let out = Command::new("bash").arg("-c").arg(cmd).output();
    match out {
        Ok(o) => {
            let s = format!(
                "{}\n{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            )
            .trim()
            .to_string();
            (o.status.success(), s)
        }
        Err(e) => (false, e.to_string()),
    }
}

/// Port of is_browser_proxy_enabled()
pub fn is_browser_proxy_enabled() -> bool {
    let home = home_dir();
    let brave_flag = home.join(".config/brave-flags.conf");
    if brave_flag.exists() {
        if let Ok(s) = fs::read_to_string(&brave_flag) {
            if s.contains("127.0.0.1:3129") {
                return true;
            }
        }
    }
    for b_dir in [home.join(".mullvad/mullvadbrowser"), home.join(".mozilla/firefox")] {
        if !b_dir.exists() {
            continue;
        }
        for p in glob_recursive(&b_dir, "prefs.js") {
            if let Ok(s) = fs::read_to_string(&p) {
                if s.contains("network.proxy.type\", 1") {
                    return true;
                }
            }
        }
    }
    false
}

/// Port of is_proxy_port_listening()
pub fn is_proxy_port_listening(host: &str, port: u16) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    let addr = (host, port).to_socket_addrs();
    match addr {
        Ok(addrs) => {
            for a in addrs {
                if TcpStream::connect_timeout(&a, Duration::from_millis(200)).is_ok() {
                    return true;
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Port of CommandWorker::run (blocking variant; the async wrapper dispatches
/// the result to the main loop like the Qt `finished` signal).
pub fn run_command_blocking(action: &str) -> (bool, String) {
    let bin = adguard_cli_path();
    let bin_str = bin.display().to_string();
    let cmd_str = if action == "start" {
        "bash -c '\nsudo -n systemctl restart adguard.service 2>/dev/null || true\n\
if ! timeout 2 bash -c \"</dev/tcp/127.0.0.1/3129\" 2>/dev/null; then\n    \
sudo -n fuser -k 3129/tcp 2>/dev/null || true\n    \
sudo -n /usr/local/bin/adguard-cli start 2>/dev/null || /home/pc/.local/bin/adguard-cli start 2>/dev/null || true\nfi\n'".to_string()
    } else if action == "stop" {
        "bash -c '\nsudo -n systemctl stop adguard.service 2>/dev/null || true\n\
sudo -n /usr/local/bin/adguard-cli stop 2>/dev/null || /home/pc/.local/bin/adguard-cli stop 2>/dev/null || true\n\
sudo -n pkill -f \"adguard-cli start\" 2>/dev/null || true\n\
sudo -n fuser -k 3129/tcp 2>/dev/null || true\n'".to_string()
    } else {
        format!("sudo -n '{bin_str}' {action} 2>&1 || '{bin_str}' {action} 2>&1")
    };

    let out = match run_shell(&cmd_str, Duration::from_secs(30)) {
        Some(r) => r,
        None => {
            if action == "stop" {
                disable_browser_proxy();
            }
            return (false, "timeout".to_string());
        }
    };
    let text = format!("{}\n{}", out.1, out.2).trim().to_string();

    if action == "start" {
        let mut port_active = false;
        for _ in 0..8 {
            std::thread::sleep(Duration::from_millis(250));
            if is_proxy_port_listening("127.0.0.1", 3129) {
                port_active = true;
                break;
            }
        }
        if port_active {
            configure_browser_proxy();
            (true, text)
        } else {
            disable_browser_proxy();
            (false, text)
        }
    } else if action == "stop" {
        disable_browser_proxy();
        (true, text)
    } else {
        (true, text)
    }
}

/// Run a bash command with a timeout; returns (exit_ok, stdout, stderr).
fn run_shell(cmd: &str, timeout: Duration) -> Option<(bool, String, String)> {
    let mut child = match Command::new("bash").arg("-c").arg(cmd).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(_) => return None,
    };
    let out_rx = pipe_reader(child.stdout.take());
    let err_rx = pipe_reader(child.stderr.take());
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    };
    let stdout = out_rx.recv().unwrap_or_default();
    let stderr = err_rx.recv().unwrap_or_default();
    Some((status.success(), stdout, stderr))
}

fn pipe_reader<R: std::io::Read + Send + 'static>(pipe: Option<R>) -> std::sync::mpsc::Receiver<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    if let Some(mut p) = pipe {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = std::io::Read::read_to_end(&mut p, &mut buf);
            let _ = tx.send(String::from_utf8_lossy(&buf).to_string());
        });
    } else {
        let _ = tx.send(String::new());
    }
    rx
}

/// Run a CLI command with optional stdin input and a timeout.
fn run_cli(args: &[String], input: Option<&str>, timeout: Duration, quiet: bool) -> (bool, String) {
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    if input.is_some() {
        cmd.stdin(Stdio::piped());
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (false, e.to_string()),
    };
    let out_rx = pipe_reader(child.stdout.take());
    let err_rx = pipe_reader(child.stderr.take());
    if let Some(i) = input {
        if let Some(mut s) = child.stdin.take() {
            let _ = s.write_all(i.as_bytes());
        }
    }
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return (false, "timeout".to_string());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return (false, "error".to_string()),
        }
    };
    let out = format!(
        "{}{}",
        out_rx.recv().unwrap_or_default(),
        err_rx.recv().unwrap_or_default()
    );
    (status.success(), out)
}

/// Port of StatusWorker::run
pub fn status_snapshot() -> (bool, String) {
    let trace = std::env::var("ADGUARD_TRACE").is_ok();
    let listening = is_proxy_port_listening("127.0.0.1", 3129);
    if trace {
        eprintln!("[trace] status_snapshot: port3129={listening}");
    }
    if listening {
        let out = "The AdGuard proxy server is running\nHTTP proxy is listening on 127.0.0.1:3129".to_string();
        if !is_browser_proxy_enabled() {
            configure_browser_proxy();
        }
        (true, out)
    } else {
        let out = "The AdGuard proxy server is not running".to_string();
        if is_browser_proxy_enabled() {
            disable_browser_proxy();
        }
        (false, out)
    }
}

pub fn refresh_status() {
    if STATUS_BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    util::spawn_async(status_snapshot, move |(running, out)| {
        STATUS_BUSY.store(false, Ordering::SeqCst);
        LAST_RUNNING.store(running, Ordering::SeqCst);
        let subs = std::mem::take(&mut *STATUS_CB.lock().unwrap());
        for cb in &subs {
            cb(running, &out);
        }
        *STATUS_CB.lock().unwrap() = subs;
    });
}

/// Port of BackendManager::set_protection (async, single worker guard)
pub fn set_protection(enable: bool) {
    if std::env::var("ADGUARD_TRACE").is_ok() {
        eprintln!("[trace] set_protection(enable={})", enable);
    }
    if PROTECTION_BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let action = if enable { "start".to_string() } else { "stop".to_string() };
    util::spawn_async(
        move || run_command_blocking(&action),
        move |(success, out)| {
            PROTECTION_BUSY.store(false, Ordering::SeqCst);
            on_action_result(success, &out);
        },
    );
}

/// Port of BackendManager::run_async_action. CommandWorker ignores `needs_root`
/// (kept for interface parity); the action name is command[0].
pub fn run_async_action(action: &str, _needs_root: bool) {
    let act = action.to_string();
    util::spawn_async(move || run_command_blocking(&act), move |(success, out)| {
        on_action_result(success, &out);
    });
}

fn on_action_result(success: bool, out: &str) {
    let subs = std::mem::take(&mut *ACTION_CB.lock().unwrap());
    for cb in &subs {
        cb(success, out);
    }
    *ACTION_CB.lock().unwrap() = subs;
    refresh_status();
}

/// Port of FilterToggleWorker::run
pub fn toggle_filter(filter_id: i64, enable: bool) {
    util::spawn_async(
        move || filter_toggle_blocking(filter_id, enable),
        move |(fid, success, out)| {
            let subs = std::mem::take(&mut *FILTER_CB.lock().unwrap());
            for cb in &subs {
                cb(fid, success, &out);
            }
            *FILTER_CB.lock().unwrap() = subs;
        },
    );
}

fn filter_toggle_blocking(filter_id: i64, enable: bool) -> (i64, bool, String) {
    let home = home_dir();
    let home_bin = home.join(".local/bin/adguard-cli");
    let bin = if home_bin.exists() { home_bin } else { PathBuf::from("adguard-cli") };
    let bin_str = bin.display().to_string();
    let id = filter_id.to_string();

    let mut last = (false, String::new());
    if enable {
        let _ = run_cli(&[bin_str.clone(), "filters".into(), "add".into(), id.clone()], Some("yes\n"), Duration::from_secs(15), false);
        last = run_cli(&[bin_str.clone(), "filters".into(), "enable".into(), id.clone()], Some("yes\n"), Duration::from_secs(15), false);
        // sync root daemon if running under root
        let _ = run_cli(&["sudo".into(), "-n".into(), bin_str.clone(), "filters".into(), "add".into(), id.clone()], Some("yes\n"), Duration::from_secs(5), true);
        let _ = run_cli(&["sudo".into(), "-n".into(), bin_str.clone(), "filters".into(), "enable".into(), id.clone()], Some("yes\n"), Duration::from_secs(5), true);
    } else {
        last = run_cli(&[bin_str.clone(), "filters".into(), "disable".into(), id.clone()], None, Duration::from_secs(15), false);
        let _ = run_cli(&["sudo".into(), "-n".into(), bin_str.clone(), "filters".into(), "disable".into(), id.clone()], None, Duration::from_secs(5), true);
    }
    (filter_id, last.0, last.1)
}

pub fn get_auto_language_filters() -> bool {
    let home = home_dir();
    let home_bin = home.join(".local/bin/adguard-cli");
    let bin = if home_bin.exists() { home_bin } else { PathBuf::from("adguard-cli") };
    let r = run_cli(&[bin.display().to_string(), "config".into(), "show".into()], None, Duration::from_secs(5), false);
    {
        let re = Regex::new(r"(?i)auto_enable_language_filters:\s*(true|false)").unwrap();
        if let Some(m) = re.captures(&r.1) {
            return m.get(1).map(|s| s.as_str().to_lowercase() == "true").unwrap_or(false);
        }
    }
    true
}

pub fn set_auto_language_filters(enable: bool) {
    let home = home_dir();
    let home_bin = home.join(".local/bin/adguard-cli");
    let bin = if home_bin.exists() { home_bin } else { PathBuf::from("adguard-cli") };
    let val = if enable { "true" } else { "false" };
    let _ = run_cli(&[bin.display().to_string(), "config".into(), "set".into(), "auto_enable_language_filters".into(), val.into()], None, Duration::from_secs(10), false);
}

/// Port of BackendManager::run_cmd_async
pub fn run_cmd_async(cmd: &str, on_complete: Box<dyn Fn(bool, String)>) {
    let cmd = cmd.to_string();
    util::spawn_async(
        move || {
            // If the command involves adguard-cli and is not already sudo'd, run the
            // root-daemon variant first, then the user one to keep proxy.yaml synced.
            if cmd.contains("adguard-cli") && !cmd.starts_with("sudo") {
                let root_cmd = cmd.replace("adguard-cli", "sudo -n adguard-cli");
                let _ = run_shell(&root_cmd, Duration::from_secs(15));
            }
            match run_shell(&cmd, Duration::from_secs(15)) {
                Some(r) => (r.0, if !r.1.is_empty() { r.1 } else { r.2 }),
                None => (false, "error".to_string()),
            }
        },
        move |(ok, out)| on_complete(ok, out),
    );
}

/// Port of BackendManager::set_https_filtering
pub fn set_https_filtering(enabled: bool, on_finish: Box<dyn Fn(bool, String)>) {
    util::spawn_async(
        move || {
            let val = if enabled { "true" } else { "false" };
            let paths = vec![
                home_dir().join(".local/share/adguard-cli/proxy.yaml"),
                PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
            ];
            let re = Regex::new(r"(https_filtering:\s*\n\s*enabled:\s*)(true|false)").unwrap();
            for p in paths {
                if p.exists() {
                    if let Ok(c) = fs::read_to_string(&p) {
                        let nc = re.replace(&c, |caps: &regex::Captures| format!("{}{}", &caps[1], val));
                        let _ = fs::write(&p, nc.as_ref());
                    }
                }
            }
            let home = home_dir();
            let home_bin = home.join(".local/bin/adguard-cli");
            let bin = if home_bin.exists() { home_bin } else { PathBuf::from("adguard-cli") };
            let bin_str = bin.display().to_string();
            let _ = run_cli(&["sudo".into(), "-n".into(), bin_str.clone(), "config".into(), "set".into(), "https_filtering.enabled".into(), val.into()], None, Duration::from_secs(5), true);
            let _ = run_cli(&[bin_str, "config".into(), "set".into(), "https_filtering.enabled".into(), val.into()], None, Duration::from_secs(5), true);

            if enabled {
                let _ = install_certificate();
            }
            (true, val.to_string())
        },
        move |(ok, out)| on_finish(ok, out),
    );
}

pub fn is_https_filtering_enabled() -> bool {
    let paths = vec![
        home_dir().join(".local/share/adguard-cli/proxy.yaml"),
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
    ];
    let re = Regex::new(r"(?i)https_filtering:\s*\n\s*enabled:\s*(true|false)").unwrap();
    for p in paths {
        if p.exists() {
            if let Ok(c) = fs::read_to_string(&p) {
                if let Some(m) = re.captures(&c) {
                    return m.get(1).map(|s| s.as_str().to_lowercase() == "true").unwrap_or(false);
                }
            }
        }
    }
    true
}

pub struct ModeInfo {
    pub mode: String,
    pub port: String,
    pub title: String,
    pub sub: String,
}

/// Port of get_filtering_mode_info()
pub fn get_filtering_mode_info() -> ModeInfo {
    let paths = vec![
        home_dir().join(".local/share/adguard-cli/proxy.yaml"),
        PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
    ];
    let mut port = "3129".to_string();
    let mut mode = "manual".to_string();
    let re_port = Regex::new(r"http_proxy:\s*([0-9]+)").unwrap();
    let re_mode = Regex::new(r"proxy_mode:\s*([a-zA-Z0-9_-]+)").unwrap();
    for p in paths {
        if p.exists() {
            if let Ok(c) = fs::read_to_string(&p) {
                if let Some(m) = re_port.captures(&c) {
                    let v = m.get(1).unwrap().as_str();
                    if v != "-1" {
                        port = v.to_string();
                    }
                }
                if let Some(m) = re_mode.captures(&c) {
                    if m.get(1).unwrap().as_str().to_lowercase().contains("auto") {
                        mode = "auto".to_string();
                    }
                }
            }
        }
    }
    let title = if mode == "manual" {
        "System proxy (Manual)".to_string()
    } else {
        "Automatic redirect (iptables)".to_string()
    };
    let sub = if mode == "manual" {
        format!("{} • Port {}", title, port)
    } else {
        title.clone()
    };
    ModeInfo { mode, port, title, sub }
}

pub struct FilterRow {
    pub id: i64,
    pub title: String,
    pub name: String,
    pub enabled: bool,
    pub installed: bool,
    pub updated: String,
}

/// Port of query_group_filters_db() — reads the AdGuard SQLite DB directly.
pub fn query_group_filters_db(group_id: i64) -> Vec<FilterRow> {
    let db_path = home_dir().join(".local/share/adguard-cli/agflm_standard.db");
    if !db_path.exists() {
        return Vec::new();
    }
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Error querying group {}: {}", group_id, e);
            return Vec::new();
        }
    };
    let mut stmt = match conn.prepare(
        "SELECT f.filter_id, f.title, l.name, f.is_enabled, f.is_installed, \
         datetime(f.last_update_time, 'unixepoch', 'localtime') \
         FROM filter f \
         LEFT JOIN filter_localisation l ON f.filter_id = l.filter_id AND l.lang = 'vi' \
         WHERE f.group_id = ? \
         ORDER BY f.is_enabled DESC, f.is_installed DESC, f.filter_id ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            println!("Error querying group {}: {}", group_id, e);
            return Vec::new();
        }
    };
    let rows = stmt.query_map([group_id], |r| {
        let fid: i64 = r.get(0)?;
        let title: String = r.get(1).unwrap_or_default();
        let loc_name: Option<String> = r.get(2).unwrap_or(None);
        let is_en: i64 = r.get(3).unwrap_or(0);
        let is_inst: i64 = r.get(4).unwrap_or(0);
        let last_upd: Option<String> = r.get(5).unwrap_or(None);
        let mut display_name = title.clone();
        if group_id == 7 {
            display_name = loc_name.unwrap_or(title.clone());
            if fid == 214 && !display_name.contains("Tiếng Việt") {
                display_name = "Tiếng Việt (ABPVN List)".to_string();
            }
        }
        Ok(FilterRow {
            id: fid,
            title,
            name: display_name,
            enabled: is_en != 0,
            installed: is_inst != 0,
            updated: last_upd.unwrap_or_else(|| "Recent".to_string()),
        })
    });
    match rows {
        Ok(it) => it.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            println!("Error querying group {}: {}", group_id, e);
            Vec::new()
        }
    }
}

pub fn query_language_filters_db() -> Vec<FilterRow> {
    query_group_filters_db(7)
}

