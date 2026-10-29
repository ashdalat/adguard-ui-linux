# adguard-blocker

GTK4 + Rust port của `AdGuard_CachyOS_GUI_App` (PyQt6). Engine **không đổi** — vẫn dùng
`adguard-cli` chính thức của AdGuard; app này chỉ là GUI điều khiển (start/stop, filters,
proxy browsers, cert, statistics từ access.log).

## Build

```bash
cd adguard-blocker
cargo build --release
./target/release/adguard-blocker
```

Cần: gtk4 >= 4.10, libadwaita không bắt buộc, rust toolchain. `rusqlite` bundle sqlite luôn.

## Trạng thái port (WIP)

**Đã port xong code:**
- `src/backend.rs` — toàn bộ backend.py + protection_data.py + filter_db.py: start/stop protection,
  configure/disable browser proxy (gsettings, brave-flags.conf, prefs.js Firefox/Mullvad),
  install_certificate (giữ nguyên mega bash command gốc), đọc/ghi proxy.yaml (regex),
  SQLite agflm_standard.db (language filters, group filters, installed filters), user.txt rules,
  read_machine_protection_status (parse proxy.yaml theo indent), app actions (browsers.yaml/proxy.yaml).
- `src/stats.rs` — StatsWorker: parse access.log*, COMPANY_MAP, format count/bytes, time filter.
- `src/apps.rs` — apps_scanner: scan .desktop, phân loại browsers/other/system.
- `src/i18n.rs` — 535 keys VI/EN export từ i18n.py → `src/i18n_data.json` (include_str),
  cùng file settings `~/.config/adguard-cachyos-gui/settings.json` với app cũ.
- `src/widgets.rs` — ToggleSwitch vẽ tay bằng cairo (y chang bản Qt: animation, checkmark,
  orange ring khi OFF), FeatureIconsRow, Spinner, ProtectionItemRow, SubFilterRow, StatListRow...
- `src/ui/` — sidebar (collapse 236↔56), home (toggle lớn 96x50 + network cards + subviews
  HTTPS/mode), protection (8 modules + detail + language filters + group filters + user rules),
  statistics (requests/dns/companies/data usage + popover menus), apps, settings, updates.
- `src/main.rs` — single-instance (abstract unix socket), tray (ksni/SNI), hide-on-close,
  status timer 4s, auto-start protection sau 1.5s.

**Chưa xong / cần sửa khi build tiếp (TODO):**
> **Xem `PORT_REPORT.md`** — báo cáo đợt sửa 2026-10-12: `cargo check`/`cargo build` đã pass
> (0 error/0 warning, kiểm chứng trên macOS), `FilterToggleRow::connect_toggled` đã viết lại đúng,
> đã thêm debug flag `--open-menu <name>`, và danh sách bug UI đã sửa + việc còn lại.

1. ~~`cargo build` CHƯA pass~~ **DONE** — đã pass; chi tiết các lỗi đã sửa trong `PORT_REPORT.md`.
2. ~~`src/ui/protection.rs`: `FilterToggleRow::connect_toggled` viết dở~~ **DONE** — đã rewrite
   truyền filter_id vào closure lúc construct (pattern `ProtectionItemRow`).
3. Home tab: các nút back của https/mode view đang lấy `first_child()` hơi fragile — nên giữ
   reference trực tiếp.
4. Chưa port: tracking/dns/stealth detail views đầy đủ (đang dùng generic), extensions_view,
   filtering log dialog (đang không có), allowlist/EV certs subviews, settings advanced/network,
   license live từ CLI, live retranslate một số view con.
5. Icon tray đang dùng `security-high` theme icon — muốn icon riêng thì copy svg vào
   `~/.local/share/icons/hicolor/scalable/apps/adguard-blocker.svg`.
6. Language switch trong Settings chỉ retranslate các view đã đăng ký; một số label cần rebuild.

## Cấu trúc cũ → mới

| PyQt6 (AdGuard_CachyOS_GUI_App) | Rust (adguard-blocker) |
|---|---|
| backend.py, protection_data.py, filter_db.py | src/backend.rs |
| stats_worker.py | src/stats.rs |
| apps_scanner.py, apps_config.py | src/apps.rs + backend::save_app_action |
| i18n.py | src/i18n.rs + src/i18n_data.json |
| constants.py | src/constants.rs |
| widgets/*.py | src/widgets.rs |
| tabs/home_tab.py, home_network_section.py | src/ui/home.rs |
| tabs/protection_tab.py + các sub view | src/ui/protection.rs |
| tabs/statistics_tab.py, companies_view.py | src/ui/statistics.rs |
| tabs/apps_tab.py | src/ui/apps_tab.rs |
| tabs/settings_tab.py | src/ui/settings.rs |
| tabs/updates_tab.py | src/ui/updates.rs |
| run_modern_gui.py, main_window.py | src/main.rs |
