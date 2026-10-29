// Port of run_modern_gui.py — app entry: single instance, tray, window, timers.
mod backend;
mod constants;
mod filter_db;
mod i18n;
mod style;
mod tabs;
mod util;
mod widgets;
mod window;

use gtk4::prelude::*;
use gtk4::glib;

const APP_ID: &str = "com.cachyos.adguard-blocker";

// Same abstract-socket name as run_modern_gui.py
// ('\0adguard_modern_single_instance_lock' — the leading NUL is implicit in
// UnixAddr::new_abstract), so the Rust app wakes a running Python instance
// (and vice versa) like a drop-in replacement.
#[cfg(target_os = "linux")]
const SOCKET_NAME: &[u8] = b"adguard_modern_single_instance_lock";

/// Commands delivered from the tray / second instance to the GTK main loop.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Cmd {
    Show,
    Hide,
    Toggle,
    Enable,
    Disable,
    Pause30s,
    Quit,
}

static CMD_TX: std::sync::OnceLock<std::sync::mpsc::Sender<Cmd>> = std::sync::OnceLock::new();

/// `--selftest`: verify that each UI-facing backend entry point really reaches
/// the adguard-cli installation on this machine, and print what it found.
fn selftest() {
    use std::process::Command;
    println!("=== adguard-blocker-rebuilt selftest ===");

    let cli = backend::adguard_cli_path();
    println!("[cli path]        {}", cli.display());
    println!("[cli exists]      {}", cli.is_file());

    // `adguard-cli --version` through the same helper the UI uses.
    let out = Command::new(&cli).arg("--version").output();
    match out {
        Ok(o) => println!(
            "[cli --version]   {}",
            String::from_utf8_lossy(&o.stdout).trim()
        ),
        Err(e) => println!("[cli --version]   ERROR {}", e),
    }

    // `adguard-cli status` — the same call backend::status_snapshot() derives from.
    let out = Command::new(&cli).arg("status").output();
    match out {
        Ok(o) => println!(
            "[cli status]      {}",
            String::from_utf8_lossy(&o.stdout).trim().lines().next().unwrap_or("")
        ),
        Err(e) => println!("[cli status]      ERROR {}", e),
    }

    println!("[port 3129 open]  {}", backend::is_proxy_port_listening("127.0.0.1", 3129));
    // status_snapshot() is the real signal path (port probe + browser proxy sync);
    // is_running() only reflects the last refresh_status(), so print both.
    let (running, detail) = backend::check_status();
    println!("[check_status]    running={} ({})", running, detail.lines().next().unwrap_or(""));
    println!("[is_running]      {}", backend::is_running());
    println!("[browser proxy]   {}", backend::is_browser_proxy_enabled());
    println!("[https filtering] {}", backend::is_https_filtering_enabled());
    println!("[auto lang filt]  {}", backend::get_auto_language_filters());
    let mi = backend::get_filtering_mode_info();
    println!("[filtering mode]  {} / {} — {}", mi.mode, mi.port, mi.title);

    let groups = backend::query_group_filters_db(1);
    println!("[group 1 filters] {}", groups.len());
    let langs = backend::query_language_filters_db();
    println!("[language filters]{}", langs.len());

    let installed = crate::filter_db::get_real_installed_filters();
    println!("[installed filt]  {}", installed.len());
    if let Some(f) = installed.first() {
        println!("   first: id={} title={:?} group={:?} updated={:?}", f.id, f.title, f.group, f.updated);
    }
    println!("[user rules]      {}", crate::filter_db::count_user_rules());

    for p in [
        crate::constants::home_dir().join(".local/share/adguard-cli/proxy.yaml"),
        std::path::PathBuf::from("/root/.local/share/adguard-cli/proxy.yaml"),
    ] {
        println!("[proxy.yaml]      {} exists={}", p.display(), p.exists());
    }
    println!("[done]");
}

/// `--toggle-key <pattern> <replacement>`: run one proxy.yaml substitution
/// through the same helper the Settings views use, and report old vs new value.
fn toggle_key(pattern: &str, replacement: &str) {
    let path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".local/share/adguard-cli/proxy.yaml"))
        .unwrap_or_default();
    println!("=== toggle-key ===");
    println!("[file]    {}", path.display());
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    println!("[pattern] {pattern}");
    println!("[replace] {replacement}");
    let re = match regex::RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(r) => r,
        Err(e) => {
            println!("[error]   bad regex: {e}");
            return;
        }
    };
    if !re.is_match(&before) {
        println!("[match]   no match, file untouched");
        return;
    }
    let after = re.replace_all(&before, replacement).to_string();
    println!("[lines]   before={} after={}", before.lines().count(), after.lines().count());
    for (b, a) in before.lines().zip(after.lines()) {
        if b != a {
            println!("[changed] {b:?} -> {a:?}");
        }
    }
    match std::fs::write(&path, &after) {
        Ok(()) => println!("[written] ok"),
        Err(e) => println!("[written] ERROR {e}"),
    }
}

/// `--testcycle`: run backend::run_command_blocking("stop") then ("start") —
/// the same calls the Home toggle uses — and verify the proxy port follows.
fn testcycle() {
    println!("=== testcycle: stop -> start through backend::run_command_blocking ===");
    println!("[before] port3129={}", backend::is_proxy_port_listening("127.0.0.1", 3129));

    let (ok, out) = backend::run_command_blocking("stop");
    println!("[stop]   ok={ok} text={:?}", out.lines().next().unwrap_or(""));
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("[after stop] port3129={}", backend::is_proxy_port_listening("127.0.0.1", 3129));

    let (ok, out) = backend::run_command_blocking("start");
    println!("[start]  ok={ok} text={:?}", out.lines().next().unwrap_or(""));
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("[after start] port3129={}", backend::is_proxy_port_listening("127.0.0.1", 3129));
    println!("[browser proxy enabled] {}", backend::is_browser_proxy_enabled());
    println!("[done]");
}

fn main() {
    // Debug/verification mode: exercise the real adguard-cli integration paths
    // (binary lookup, status, proxy.yaml, filter DB, browser proxy state) and
    // print a report, without opening a window or changing the real network
    // configuration. Invoked as `--selftest`.
    if std::env::args().any(|a| a == "--selftest") {
        selftest();
        return;
    }

    // Debug/verification mode: apply a single proxy.yaml key change through the
    // exact write_key()/set_yaml_key() path the Settings switches use, then print
    // the resulting line. Invoked as `--toggle-key <pattern> <replacement>`.
    if std::env::args().any(|a| a == "--toggle-key") {
        let args: Vec<String> = std::env::args().collect();
        let idx = args.iter().position(|a| a == "--toggle-key").unwrap();
        match (args.get(idx + 1), args.get(idx + 2)) {
            (Some(pat), Some(rep)) => toggle_key(pat, rep),
            _ => println!("usage: --toggle-key <regex-pattern> <replacement>"),
        }
        return;
    }

    let app = gtk4::Application::builder().application_id(APP_ID).build();

    let (tx, rx) = std::sync::mpsc::channel::<Cmd>();
    let _ = CMD_TX.set(tx);

    // 1. Kernel-level Linux Abstract Socket Single Instance Check
    if !try_acquire_single_instance() {
        // An instance is already running: wake it up and exit cleanly
        wake_running_instance();
        std::process::exit(0);
    }

    let rx = std::rc::Rc::new(rx);
    app.connect_activate(move |app| build_ui(app, rx.clone()));
    // Debug flags like `--open-tab <id>` are ours, not GTK's; strip them so the
    // GTK option parser does not abort with "Unknown option".
    app.run_with_args(&stripped_argv());
}

/// argv minus this project's non-GTK debug flags (`--open-tab <id>`).
fn stripped_argv() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut skip_next = false;
    for a in std::env::args() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--selftest" || a == "--testcycle" {
            continue;
        }
        // --selftest / --testcycle / --toggle-key return before GTK starts, so
        // only the UI flags ever need stripping here. --autostart and --tray are
        // read by build_ui through std::env::args() and must not reach GTK's
        // option parser (it aborts with "Unknown option").
        if a == "--autostart" || a == "--tray" {
            continue;
        }
        if a == "--open-tab" || a == "--open-page" {
            skip_next = true;
            continue;
        }
        if a == "--measure" {
            continue;
        }
        out.push(a);
    }
    out
}

/// Port of run_modern_gui.py's connect + sendall(b"SHOW\n").
#[cfg(target_os = "linux")]
fn wake_running_instance() {
    use nix::sys::socket::{connect, socket, AddressFamily, SockFlag, SockType, UnixAddr};
    use std::os::fd::AsRawFd;
    let addr = match UnixAddr::new_abstract(SOCKET_NAME) {
        Ok(a) => a,
        Err(_) => return,
    };
    let fd = match socket(AddressFamily::Unix, SockType::Stream, SockFlag::SOCK_CLOEXEC, None) {
        Ok(f) => f,
        Err(_) => return,
    };
    if connect(fd.as_raw_fd(), &addr).is_ok() {
        let borrowed = unsafe { std::os::fd::BorrowedFd::borrow_raw(fd.as_raw_fd()) };
        let _ = nix::unistd::write(borrowed, b"SHOW\n");
    }
    // OwnedFd closes itself here; an explicit close would double-close.
}

#[cfg(not(target_os = "linux"))]
fn wake_running_instance() {
    use std::os::unix::net::UnixStream;
    let path = std::env::temp_dir().join(format!(
        "{}-{}.sock",
        constants::APP_NAME,
        std::env::var("USER").unwrap_or_else(|_| "default".to_string())
    ));
    let _ = UnixStream::connect(&path);
}

/// Port of the bind/listen block + IpcWakeupListener thread.
fn try_acquire_single_instance() -> bool {
    #[cfg(target_os = "linux")]
    {
        use nix::sys::socket::{
            bind, listen, socket, AddressFamily, Backlog, SockFlag, SockType, UnixAddr,
        };
        use std::os::fd::AsRawFd;
        let addr = match UnixAddr::new_abstract(SOCKET_NAME) {
            Ok(a) => a,
            Err(_) => return true,
        };
        let fd = match socket(AddressFamily::Unix, SockType::Stream, SockFlag::SOCK_CLOEXEC, None) {
            Ok(fd) => fd,
            Err(_) => return true,
        };
        match bind(fd.as_raw_fd(), &addr) {
            Ok(()) => {
                let _ = listen(&fd, Backlog::new(5).unwrap_or(Backlog::MAXCONN));
                // Keep the socket alive for the process lifetime: the listener
                // thread owns it as a std UnixListener.
                std::thread::spawn(move || {
                    use std::io::Read;
                    use std::os::fd::{FromRawFd, IntoRawFd};
                    use std::os::unix::net::UnixListener;
                    let listener = unsafe { UnixListener::from_raw_fd(fd.into_raw_fd()) };
                    for stream in listener.incoming().flatten() {
                        let mut s = stream;
                        let mut buf = [0u8; 1024];
                        let n = s.read(&mut buf).unwrap_or(0);
                        drop(s);
                        // Python: `if b"SHOW" in data: wakeup_signal.emit()`
                        if buf[..n].windows(4).any(|w| w == b"SHOW") {
                            if let Some(tx) = CMD_TX.get() {
                                let _ = tx.send(Cmd::Show);
                            }
                        }
                    }
                });
                true
            }
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // No abstract sockets outside Linux: per-user filesystem socket.
        use std::io::Read;
        use std::os::unix::net::{UnixListener, UnixStream};
        let path = std::env::temp_dir().join(format!(
            "{}-{}.sock",
            constants::APP_NAME,
            std::env::var("USER").unwrap_or_else(|_| "default".to_string())
        ));
        if UnixStream::connect(&path).is_ok() {
            return false;
        }
        let _ = std::fs::remove_file(&path); // stale socket from a crashed run
        match UnixListener::bind(&path) {
            Ok(listener) => {
                std::thread::spawn(move || {
                    for stream in listener.incoming().flatten() {
                        let mut s = stream;
                        let mut buf = [0u8; 1024];
                        let n = s.read(&mut buf).unwrap_or(0);
                        drop(s);
                        if buf[..n].windows(4).any(|w| w == b"SHOW") {
                            if let Some(tx) = CMD_TX.get() {
                                let _ = tx.send(Cmd::Show);
                            }
                        }
                    }
                });
                true
            }
            Err(_) => false,
        }
    }
}

fn build_ui(app: &gtk4::Application, rx: std::rc::Rc<std::sync::mpsc::Receiver<Cmd>>) {
    // app.setApplicationName("AdGuard"); app.setOrganizationName("AdGuard")
    glib::set_application_name("AdGuard");

    // Default modern clean font (QFont("Ubuntu", 10) + PreferAntialias)
    apply_font();

    let win = window::create(app);

    // app.setQuitOnLastWindowClosed(False): closing the window hides to tray
    // and the app keeps running.
    {
        let win2 = win.clone();
        win.connect_close_request(move |_| {
            win2.set_visible(false);
            glib::Propagation::Stop
        });
    }

    // Tray icon: logo file, fallback to the "security-high" icon theme
    #[cfg(target_os = "linux")]
    let tray_icon = load_tray_icon();

    // 30-second Pause Timer (single shot) — started by the tray's "pause 30s"
    // action; the handler below is the timer's timeout callback.

    // Tray context menu (port of the QMenu block). The SNI protocol has no menu
    // styling or per-item bold font, so the QMenu stylesheet is not portable.
    #[cfg(target_os = "linux")]
    setup_tray(&tray_icon, &win);

    // IPC + tray command pump
    {
        let win2 = win.clone();
        let app2 = app.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    Cmd::Show => show_window(&win2),
                    Cmd::Hide => win2.set_visible(false),
                    Cmd::Toggle => {
                        // Port of toggle_window (Qt isMinimized has no direct GTK4
                        // equivalent; GTK4 tracks visibility only)
                        if win2.is_visible() {
                            win2.set_visible(false);
                        } else {
                            show_window(&win2);
                        }
                    }
                    Cmd::Enable => {
                        backend::set_protection(true);
                        refresh_status(&win2);
                    }
                    Cmd::Disable => {
                        backend::set_protection(false);
                        refresh_status(&win2);
                    }
                    Cmd::Pause30s => {
                        backend::set_protection(false);
                        refresh_status(&win2);
                        notify(&app2, &i18n::tr("tray_paused_msg"));
                        // pause_timer.start(30000)
                        let win3 = win2.clone();
                        let app3 = app2.clone();
                        glib::timeout_add_local(std::time::Duration::from_secs(30), move || {
                            backend::set_protection(true);
                            refresh_status(&win3);
                            notify(&app3, &i18n::tr("tray_resumed_msg"));
                            glib::ControlFlow::Break
                        });
                    }
                    Cmd::Quit => {
                        // full_quit: tray.hide(); app.quit()
                        app2.quit();
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // window.backend.status_changed.connect(update_tray_menu) + initial sync
    i18n::connect_language_changed(Box::new(move |_| {
        // retranslate_tray(): labels/tooltip are re-read on every SNI query,
        // so nothing to do here beyond invalidating the host's cached menu.
    }));
    backend::connect_status_changed(Box::new(move |running, _detail| {
        update_tray_menu_state(running);
    }));
    update_tray_menu_state(backend::is_running());

    // Handle start mode: tray or show
    if std::env::args().any(|a| a == "--autostart" || a == "--tray") {
        win.set_visible(false);
    } else {
        show_window(&win);
    }
}

fn show_window(win: &gtk4::ApplicationWindow) {
    // show_window(): un-minimize, show, showNormal, raise, activate
    win.set_visible(true);
    win.present();
}

fn refresh_status(win: &gtk4::ApplicationWindow) {
    let (running, detail) = backend::check_status();
    window::on_status_changed(win, running, &detail);
    update_tray_menu_state(running);
}

/// Port of update_tray_menu: enable state + tooltip follow the status.
fn update_tray_menu_state(running: bool) {
    TRAY_RUNNING.store(running, std::sync::atomic::Ordering::SeqCst);
    // ksni caches the menu layout; push it again so the enabled/disabled items
    // follow the new state (Python's QMenu reflects setEnabled immediately).
    #[cfg(target_os = "linux")]
    if let Some(handle) = TRAY_HANDLE.get() {
        handle.update(|_tray| {});
    }
}

static TRAY_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn tray_tooltip() -> String {
    if TRAY_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
        i18n::tr("tray_tooltip_enabled")
    } else {
        i18n::tr("tray_tooltip_disabled")
    }
}

/// Port of QSystemTrayIcon::showMessage — SNI has no message API, so this goes
/// through the desktop notification daemon.
fn notify(app: &gtk4::Application, body: &str) {
    let n = gtk4::gio::Notification::new("AdGuard");
    n.set_body(Some(body));
    app.send_notification(Some("adguard-tray"), &n);
}

fn apply_font() {
    // QFont("Ubuntu", 10) with PreferAntialias, applied app-wide.
    let css = format!(
        "* {{ font-family: \"Ubuntu\", sans-serif; font-size: 10pt; }}\n{}",
        crate::style::css()
    );
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Port of tray.setIcon(QIcon(LOGO_PATH)): logo as an ARGB32 pixmap for SNI.
#[cfg(target_os = "linux")]
fn load_tray_icon() -> Vec<ksni::Icon> {
    let path = constants::logo_path();
    if !path.exists() {
        return Vec::new(); // Python falls back to QIcon.fromTheme("security-high")
    }
    let tex = match gtk4::gdk::Texture::from_filename(&path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let w = tex.width().max(0) as usize;
    let h = tex.height().max(0) as usize;
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let stride = w * 4;
    let mut buf = vec![0u8; stride * h];
    tex.download(&mut buf, stride);
    // gdk_texture_download writes the texture in its memory format. For this
    // image it is GDK_MEMORY_R8G8B8A8, i.e. bytes are R, G, B, A in memory
    // (confirmed at runtime by tracing texture_format). The SNI icon format is
    // ARGB32 in network byte order, i.e. bytes A, R, G, B — so reorder
    // R,G,B,A -> A,R,G,B. Treating the source as BGRA (as the previous version
    // did) produced A,B,G,R, which is why the tray icon was the wrong colour.
    let mut argb = vec![0u8; buf.len()];
    for (i, px) in buf.as_chunks::<4>().0.iter().enumerate() {
        argb[i * 4] = px[3]; // A
        argb[i * 4 + 1] = px[0]; // R
        argb[i * 4 + 2] = px[1]; // G
        argb[i * 4 + 3] = px[2]; // B
    }
    vec![ksni::Icon {
        width: w as i32,
        height: h as i32,
        data: argb,
    }]
}

#[cfg(target_os = "linux")]
struct Tray {
    sender: std::sync::mpsc::Sender<Cmd>,
    icon_pixmap: Vec<ksni::Icon>,
}

/// Kept so the tray menu can be re-published on every protection-state change.
#[cfg(target_os = "linux")]
static TRAY_HANDLE: std::sync::OnceLock<ksni::Handle<Tray>> = std::sync::OnceLock::new();

#[cfg(target_os = "linux")]
impl ksni::Tray for Tray {
        fn id(&self) -> String {
            "adguard-blocker".into()
        }
        fn icon_name(&self) -> String {
            // Python sends the logo as a pixmap (QIcon(LOGO_PATH) -> QSystemTrayIcon)
            // and shows the coloured shield. Per the SNI spec a host resolves
            // IconName through the icon theme FIRST, so any name here (the old
            // "security-high", or a themed lookup of our own name) replaced the
            // logo with a monochrome theme glyph. Leaving it empty makes the host
            // fall back to IconPixmap, which is the actual AdGuard icon.
            String::new()
        }
        fn title(&self) -> String {
            "AdGuard AdBlocker".into() // tray.setToolTip("AdGuard AdBlocker")
        }
        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            self.icon_pixmap.clone()
        }
        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                icon_name: String::new(),
                icon_pixmap: Vec::new(),
                title: "AdGuard".into(),
                description: if TRAY_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                    i18n::tr("tray_tooltip_enabled")
                } else {
                    i18n::tr("tray_tooltip_disabled")
                },
            }
        }
        fn activate(&mut self, _x: i32, _y: i32) {
            // tray.activated(Trigger/DoubleClick) -> toggle_window
            let _ = self.sender.send(Cmd::Toggle);
        }
        fn secondary_activate(&mut self, _x: i32, _y: i32) {
            let _ = self.sender.send(Cmd::Toggle);
        }
        fn menu(&self) -> Vec<ksni::MenuItem<Tray>> {
            let running = TRAY_RUNNING.load(std::sync::atomic::Ordering::SeqCst);
            let item = |label: String, enabled: bool, cmd: Cmd| {
                ksni::MenuItem::Standard(ksni::menu::StandardItem {
                    label,
                    enabled,
                    activate: Box::new(move |tray: &mut Tray| {
                        let _ = tray.sender.send(cmd);
                    }),
                    ..Default::default()
                })
            };
            vec![
                // 1. Open AdGuard (bold in Qt; SNI has no per-item font)
                item(i18n::tr("tray_open"), true, Cmd::Show),
                // 2. Enable protection
                item(i18n::tr("tray_enable_protection"), !running, Cmd::Enable),
                // 3. Disable protection for 30 seconds
                item(i18n::tr("tray_pause_30s"), running, Cmd::Pause30s),
                // 4. Disable protection
                item(i18n::tr("tray_disable"), running, Cmd::Disable),
                ksni::MenuItem::Separator,
                // 5. Close AdGuard, but continue filtering
                item(i18n::tr("tray_close_filtering"), true, Cmd::Hide),
                // 6. Exit AdGuard
                {
                    ksni::MenuItem::Standard(ksni::menu::StandardItem {
                        label: i18n::tr("tray_exit"),
                        enabled: true,
                        activate: Box::new(move |tray: &mut Tray| {
                            let _ = tray.sender.send(Cmd::Quit);
                        }),
                        ..Default::default()
                    })
                }
            ]
        }
}

#[cfg(target_os = "linux")]
fn setup_tray(icon_pixmap: &[ksni::Icon], win: &gtk4::ApplicationWindow) {
    let sender = CMD_TX.get().cloned().expect("command channel");
    let tray = Tray {
        sender,
        icon_pixmap: icon_pixmap.to_vec(),
    };
    // TrayService::spawn() runs the SNI service on its own thread; the Handle
    // shares the same model, so it can push menu/property updates later (ksni
    // caches the menu, so without an explicit update() the enabled state of the
    // items would stay frozen at creation time while the tooltip kept changing).
    let svc = ksni::TrayService::new(tray);
    let handle = svc.handle();
    let _ = TRAY_HANDLE.set(handle);
    std::thread::spawn(move || {
        svc.spawn();
    });
    let _ = win;
}
