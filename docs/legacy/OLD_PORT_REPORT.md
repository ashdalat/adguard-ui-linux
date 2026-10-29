# Báo cáo port AdGuard Blocker (GTK4 + Rust) — đợt sửa compile & test UI

Ngày: 2026-10-12 • Môi trường sửa: macOS (darwin 24.6.0, arm64) • gtk4-rs 0.9.7 / gtk4 4.24.1

## Trạng thái cuối

- `cargo check`: **pass — 0 error, 0 warning**
- `cargo build --release`: **pass** → `target/release/adguard-blocker`
- Đã chạy thử app và **test cả 7 view** (home, protection, protection:ad_blocking, statistics,
  apps, settings, updates) bằng debug flag `--open-menu` + chụp màn hình kiểm tra: tất cả render
  đúng, không còn panic.
- Mọi thay đổi ảnh hưởng riêng macOS đều được bọc `#[cfg(...)]`, **hành vi trên Linux giữ nguyên
  như thiết kế ban đầu** (abstract socket, ksni tray vẫn bật trên Linux).

## Cài đặt hệ thống (macOS)

`brew install gtk4 dbus` — máy chưa có gtk4 và dbus-1 (pkg-config không thấy). Trên Linux cần
đủ `gtk4 >= 4.10` + `pkg-config` như README đã ghi; **rusqlite bundle sqlite nên không cần cài thêm**.

## Các lỗi compile đã sửa

### 1. Lỗi dây chuyền từ raw string — `src/backend.rs` (get_dns_upstream)
`r"servers:\s*\n\s*-\s*['\"]?..."` chứa dấu `"` trong raw string thường → string đứt sớm, đẻ ra
44 lỗi token phía sau. Sửa thành `r#"..."#`. Đây là lỗi gốc lớn nhất.

### 2. Viết lại `FilterToggleRow` — `src/ui/protection.rs` (TODO số 2 của README)
- Struct thêm field `filter_id: i64`; `connect_toggled` capture id từ row lúc connect (cùng pattern
  với `ProtectionItemRow`), callback nhận đúng `(filter_id, checked)`.
- Bỏ handler nội bộ trong `new()` (trước đây toggle bị bắn **2 lần**, một lần với id sai = 0).
- Xoá `thread_local FILTER_ID_STORE` không dùng.

### 3. Pattern thread → UI sai với GTK (main.rs, statistics.rs, updates.rs)
Code cũ `std::thread::spawn` + `glib::idle_add_local_once` từ thread con: không biên dịch được vì
closure chứa `Rc<Widget>` không `Send` (về runtime cũng sai vì `idle_add_local_*` chỉ được gọi ở
main thread). Thay bằng:
- `main.rs`: helper `run_backend_async(work, done)` — chạy backend ở worker thread, marshal kết
  quả về main loop qua `std::sync::mpsc` + `timeout_add_local` poll. Áp dụng cho 4 chỗ: toggle
  protection, timer 4s, check ban đầu, auto-start 1.5s.
- `statistics.rs::refresh_stats` / `updates.rs::start_check`: cùng pattern mpsc + poll
  (glib 0.20 đã **bỏ** `MainContext::channel`, không dùng lại được API cũ).
- Receiver đổi thành `self: &Rc<Self>` ở `refresh_stats`, `on_show`, `start_check`.

### 4. API khác giữa các crate version
- gtk4 0.9: `add_controller` nhận **by value** (`area.add_controller(gesture)`); `Image::set_cursor`
  cần `.as_ref()` thay `.ok()`; `CssProvider::load_from_string` thay `load_from_data`;
  `Cell::new(0.0f64)` (E0689 ambiguous float); `dict.lookup_value(..., Some(VariantTy::STRING))`.
- glib 0.20: `MainContext::channel` không tồn tại (xem mục 3).
- nix: `UnixAddr::new_abstract` chỉ có trên Linux (xem mục 6); `Uid` cần feature "user" → bỏ hẳn.
- ksni 0.2: menu dùng `MenuItem::Standard(ksni::menu::StandardItem { ..Default::default() })` +
  `MenuItem::Separator`; spawn qua `ksni::TrayService::new(tray).spawn()`.
- rusqlite: `row.get::<_, Option<String>>(n).unwrap_or_default()` trả lại `Option<String>`
  (unwrap trên Result) → sửa thành `.ok().flatten().unwrap_or_default()`.
- regex: `m[1] != "x"` cần `&m[1]` (Index của Captures có `Output = str`).
- gtk4 0.9: loop `while let Some(w) = c` phải lấy `c = w.next_sibling()` **trước** khi
  `w.downcast::<Button>()` (w bị move).

### 5. Thiếu import / khai báo
- `use gtk4::glib;` (main, statistics, updates) • `use std::rc::Rc;` (sidebar) •
  `use crate::i18n::tr;` (settings) • `svg_image` thêm vào import widgets (protection) •
  `serde_json = "1"` thêm vào Cargo.toml • constants mới: `COLOR_SWITCH_KNOB_ON/OFF` (constants.rs) •
  `home.rs`: struct init `lbl_mode_sub: mode_sub_lbl`; `make_card` đổi sang
  `Option<&impl IsA<gtk4::Widget>>`; `make_nav_row` trả `(Button, Label)`.
- `src/i18n.rs`: registry retranslate đổi sang `thread_local` (`Box<dyn Fn()>` — closure chứa Rc
  không Send); leak `lang` một lần ngoài vòng lặp (E0382).

### 6. Riêng macOS (cfg-gated, Linux không đổi)
- Single-instance: `UnixAddr::new_abstract` (abstract socket) chỉ có trên Linux → giữ nguyên nhánh
  Linux, thêm nhánh `#[cfg(not(target_os = "linux"))]` dùng socket file trong `$TMPDIR`
  (`adguard-blocker-<USER>.sock`), có kiểm tra instance còn sống trước khi xoá socket stale.
- Tray ksni: `#[cfg(target_os = "linux")]` cho `setup_tray` — trên macOS không có D-Bus session /
  SNI host, thread tray chỉ panic vô ích. `SOCKET_NAME` cũng cfg linux.

## Bug runtime/UI bắt được khi test (đã sửa)

1. **Crash khi click row Protection** — panic `gio list_model: item_type is_a(...) assertion` ở
   main thread khi iterate `observe_children().iter::<Widget>()` (ListModel item type không khớp
   trên macOS). Thay 3 chỗ (protection detail, sidebar ×2) bằng walk `first_child()`/`next_sibling()`.
2. **Feature icons trên Home không hiện** — `FeatureIconsRow::new()` tạo button nhưng **quên
   `root.append(&btn)`**. Icons chiếm chỗ nhưng không vẽ. Đã append.
3. **Tiêu đề row mờ khắp app** — `lbl()` truyền chuỗi nhiều class (`"text-white bold-15"`) vào
   `add_css_class()` một lần → GTK không match class nào. Fix: `class.split_whitespace()` rồi add
   từng class.
4. **19 key i18n thiếu** hiển thị raw: đổi 5 key sang key gốc của app cũ trong code
   (`apps_group_browsers/other/system`, `fe_empty_rules`, `setting_language`, `fe_add_rule_btn`),
   thêm 13 key mới vào `src/i18n_data.json` (both en/vi): `all_up_to_date, checking,
   checking_updates, engine_label, installed_status, not_installed_status, license_free_text,
   no_rules_yet, settings_autostart, settings_language_note, stat_dns_requests, support_text,
   version_label`.
5. **Header các tab căn giữa** (Label:xalign mặc định 0.5) — căn trái + hexpand cho header
   Protection/Apps/Settings/Updates, tiêu đề nhóm Apps, status label, label "User filter is empty".
6. CSS `opacity: 0px` → `opacity: 0` (GTK parser không nhận px cho opacity, main.rs).
7. Markup Pango: strip ` style='...'` trong `http_proxy_desc` khi set label (GTK không cho phép
   attribute style trên `<a>`, home.rs).
8. Dọn warnings: bọc `let _ =` cho các call cairo (`save/fill/stroke/restore`), `#[allow(dead_code)]
   ` có chú thích cho các hàm/field giữ cố ý để đối chiếu bản PyQt6.

## Feature mới: debug flag `--open-menu`

```bash
./target/release/adguard-blocker --open-menu home
# name ∈ {home | protection | statistics | apps | settings | updates}
./target/release/adguard-blocker --open-menu protection:ad_blocking   # mở detail của module
```
Đăng ký qua `app.add_main_option(...)` + `connect_handle_local_options` (parse tay sẽ bị GTK từ
chối "Unknown option"). `statistics`/`updates` tự refresh khi mở bằng flag.

## Việc còn lại khi làm tiếp trên Linux (theo README, chưa làm)

1. README TODO số 3: back button của https/mode view đang lấy `first_child()` hơi fragile — nên
   giữ reference trực tiếp.
2. TODO số 4: chưa port tracking/dns/stealth detail views đầy đủ, extensions_view, filtering log
   dialog, allowlist/EV certs subviews, settings advanced/network, license live từ CLI, live
   retranslate một số view con.
3. TODO số 5: icon tray đang dùng `security-high`; muốn icon riêng thì copy svg vào
   `~/.local/share/icons/hicolor/scalable/apps/adguard-blocker.svg`.
4. TODO số 6: language switch chỉ retranslate các view đã đăng ký; một số label cần rebuild.
5. Apps tab: danh sách đang quét `.desktop` theo đường dẫn Linux (`/usr/share/applications`,
   flatpak exports) — đúng ý thiết kế, chỉ chạy trên Linux.
6. Gợi ý test trên Linux: `cargo check && cargo build --release`, chạy không flag, rồi lần lượt
   `--open-menu` từng view; kiểm tra click từng row Protection (vùng từng panic), toggle filter
   ngôn ngữ (FilterToggleRow đã sửa — xác nhận đúng filter id được truyền), language switch
   VI/EN trong Settings.

## Lưu ý engine (không thuộc GUI)

App điều khiển `adguard-cli` và đọc `~/.local/share/adguard-cli/` (proxy.yaml, browsers.yaml,
agflm_standard.db, logs/access.log*). Trên máy Mac chưa có engine nên mọi trạng thái = fallback
(protection paused, số liệu 0). Trên Linux có adguard-cli cài sẵn thì GUI sẽ đọc đúng.

## File đã sửa (tổng hợp)

- `Cargo.toml` — thêm serde_json
- `src/main.rs` — glib import, single-instance cfg, tray cfg, ksni menu/spawn, run_backend_async,
  --open-menu, css fixes
- `src/backend.rs` — raw string fix, `&m[1]`, Option<String>, stmt mut, io::Read, dead_code allows
- `src/i18n.rs` — thread_local registry, RefCell import, leak lang
- `src/constants.rs` — 2 constants màu, allow dead_code
- `src/apps.rs` — allow dead_code cho AppEntry
- `src/stats.rs` — ép kiểu f64
- `src/widgets.rs` — lbl() split class, append feature buttons, add_controller by value,
  set_cursor, Cell f64, refresh_all_icons, cairo lets, dead_code allows
- `src/ui/protection.rs` — FilterToggleRow rewrite, import svg_image, SubFilterRow borrow,
  observe_children → first_child walk, set_text/String, xalign header, i18n key
- `src/ui/sidebar.rs` — Rc import, first_label() helper thay observe_children, toggled handler
- `src/ui/home.rs` — struct init, make_card/make_nav_row signature, add_controller, markup strip,
  pub feature_icons, xalign
- `src/ui/statistics.rs` — mpsc poll thay MainContext::channel, glib import, next_sibling trước
  downcast, refresh_stats receiver
- `src/ui/updates.rs` — mpsc poll, glib import, receiver Rc, xalign header
- `src/ui/settings.rs` — tr import, setting_language key, xalign header
- `src/ui/apps_tab.rs` — exec clone đúng, key apps_group_*/fe_empty_rules, xalign, fill()
- `src/i18n_data.json` — +13 key (en/vi)
