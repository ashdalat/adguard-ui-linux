# PORT_AUDIT.md — AdGuard Python (PyQt6) → Rust (GTK4)

Ngày kiểm tra: 2026-10-14 • Máy: CachyOS Linux, session Wayland (KWin) + Xwayland trên `:0`

## 1. Đường dẫn

| Vai trò | Đường dẫn |
|---|---|
| Python gốc (nguồn chuẩn) | `/home/pc/Desktop/Softlinux/cachyos software/AdGuard_CachyOS_GUI_App` |
| **Rust mới (dự án port)** | **`/home/pc/Desktop/Softlinux/cachyos software/adguard-blocker-rebuilt`** |
| Tài liệu bản Rust cũ | `adguard-blocker-rebuilt/docs/legacy/` (`OLD_PORT_REPORT.md`, `OLD_README.md`, `OLD_GIT_LOG.txt`) |
| File theo dõi tiến độ | `adguard-blocker-rebuilt/PORT_PROGRESS.md` |

Bản Rust cũ (`adguard-blocker-backup`, 2.2 GB) **đã được xóa theo yêu cầu**; tài liệu của nó
được lưu lại trong `docs/legacy/` trước khi xóa. Dự án mới chỉ tái sử dụng code cũ sau khi đã
xác minh lại từng phần so với Python.

## 2. Phạm vi đã port

- Python: **69 file `.py`** (không tính `__init__.py` và `legacy_archive/adguard_gui.py`, `adguardvpn_gui_old.py`
  vốn là bản Qt cũ, và app VPN là dự án riêng).
- Rust: **74 file `.rs`**, ~28.470 dòng, phủ hết 69/69 file Python theo ánh xạ:
  `adguard_modern/*.py → src/*.rs`, `adguard_modern/tabs/*.py → src/tabs/*.rs`,
  `adguard_modern/widgets/*.py → src/widgets/*.rs`, `run_modern_gui.py → src/main.rs`.
- Tài nguyên kèm theo: `assets/` (60 icon), `adguard_blocker_icon.png`, `i18n_export.json`
  (dự án mới chưa có `assets/` — đã copy từ nguồn Python, nếu thiếu thì mọi icon đều không hiển thị).
- Đóng gói: `packaging/setup_adguard_gui.sh`, `uninstall_adguard_gui.sh`, `setup_cachyos_gui.sh`,
  `start_adguard_service.sh` — giữ nguyên các bước, đường dẫn, sudoers, chính sách NSS/CA, systemd unit
  và autostart `.desktop` của script Python; chỉ khác đích khởi chạy (binary Rust thay `run_modern_gui.py`).
  `setup_all_adguard.sh`, `setup_vpn_gui.sh`, `setup_bypass.sh`, `remove_bypass.sh` thuộc app AdGuard VPN
  riêng, ngoài phạm vi blocker này.

Không có `todo!()`, `unimplemented!()`, GUI giả hay hardcode kết quả: `grep` toàn `src/` trả về 0 kết quả.
Mọi nhánh, callback, timer và xử lý lỗi trong file Python đều được port; các no-op cố ý của Python
(ví dụ `pass` trong `dns_protection_view.py`, `set_protection_state` của sidebar) được giữ nguyên và ghi chú.

## 3. Kết quả kiểm tra

| Kiểm tra | Kết quả |
|---|---|
| `cargo check` | pass — 0 error |
| `cargo build` (debug + release) | pass |
| `cargo test` | **18 test pass** — i18n (4), filter_db (3), stats_worker (3), apps_config (3), settings_advanced_view (2), home_tab chống vòng lặp phản hồi (3) |
| `cargo clippy` | **0 error**; còn warning (chủ yếu `type_complexity`, `unused Result`, API GTK bị deprecated, `retranslate_ui` không dùng ở vài view) |
| Khởi động GUI Rust | pass, không còn warning CSS do mình gây ra |
| Mở 6 tab chính (`--open-tab`) | pass — cả 6 render đúng |
| **Mở toàn bộ trang con (`--open-page`)** | pass — Protection 16 trang, Settings 15 trang, Home 5 trang đều render, không cảnh báo GTK |
| **Tray (SNI) trên desktop thật** | pass — 6 mục menu tiếng Việt, 6 hành động đều đúng, menu tự cập nhật theo trạng thái |
| **`--tray` / `--autostart`** | pass — khởi động ẩn, không hiện cửa sổ, đăng ký tray |

### Liên kết với `adguard-cli` (kiểm tra chạy thật, không mock)

Thêm các cờ debug chỉ dùng cho kiểm thử (ngoài CLI của Python):

- `--selftest`: in ra từng điểm vào backend có thực sự chạm `adguard-cli` hay không.
  Kết quả trên máy này:
  - CLI: `/home/pc/.local/bin/adguard-cli`, tồn tại, `AdGuard CLI v1.4.13`
  - `status`: OK; port 3129, browser proxy, https filtering=true, auto language filters=true,
    filtering mode = `manual / 3129 — System proxy (Manual)`
  - DB: nhóm 1 = 3 filter, filter ngôn ngữ = 54, filter đã cài = 11, user rules = 0
  - `proxy.yaml` tồn tại ở cả `~` và `/root`
  - Các số này **khớp** với kết quả gọi đúng các hàm đó từ Python (`query_group_filters_db(1)`,
    `query_language_filters_db()`, `filter_db.get_real_installed_filters()`).
  - Lưu ý đã kiểm chứng: `adguard-cli status` gọi **không qua sudo** báo "not running" trong khi
    port 3129 đang mở; `backend::check_status()` (đường tín hiệu thật, dò port như Python) báo đúng
    `running=true`. Đây là hành vi khớp Python (`status_snapshot` cũng dò port 3129 chứ không đọc
    kết quả `status`), không phải lỗi.
- `--testcycle`: chạy `backend::run_command_blocking("stop")` rồi `("start")` — đúng đường mà
  công tắc ở tab Home dùng: port 3129 `true → stop → false → start → true`, browser proxy được bật lại.
- `--open-tab <id>` / `--open-page <name>`: mở thẳng một tab và một trang con của tab đó.
- `--toggle-key <pattern> <replacement>`: chạy đúng một phép thay proxy.yaml qua helper mà các view
  Settings dùng, in ra dòng trước/sau. Đã dùng để xác minh việc ghi YAML không phá nội dung file
  (221 dòng → 221 dòng, chỉ 1 dòng đổi, sau đó khôi phục về byte-identical).

Bằng chứng CLI bị điều khiển thật: `journalctl`/log `sudo` ghi các lệnh
`systemctl restart/stop adguard.service`, `adguard-cli start|stop`,
`adguard-cli config set stealthmode...` với `PWD` chính là thư mục dự án Rust.
(Lệnh `stealthmode.enabled true` lúc mở app là hành vi đúng: Python cũng gọi
`_select_level("custom")` ngay trong constructor của `TrackingProtectionView`.)

Trước khi test đã sao lưu `~/.local/share/adguard-cli/proxy.yaml`,
`/root/.local/share/adguard-cli/proxy.yaml`, `settings.json` và trạng thái
`gsettings org.gnome.system.proxy mode` vào `/tmp/adguard_test_backup/`.
Sau khi test, proxy đã được khôi phục: `adguard.service` = active/enabled, port 3129 đang LISTEN,
`sudo -n /usr/local/bin/adguard-cli status` báo "proxy server is running".

## 4. Lỗi phát hiện và đã sửa trong quá trình chạy thật

1. **`home_tab.rs`** — `Stack` con được tạo và nạp đủ trang nhưng **không gắn vào `root`**
   (Python: `outer_layout.addWidget(self.stack)`) → tab Home trắng. Đã sửa.
2. **`protection_tab.rs`** — `container` chứa các dòng module không được gắn vào `scroll`/`main_view`
   (Python: `main_layout.addWidget(scroll_area)`) → tab Bảo vệ trắng. Đã sửa.
3. **`settings_network_view.rs`** — thiếu `scroll.setWidget(container)` + `addWidget(scroll)`. Đã sửa.
4. **`i18n.rs`** — `i18n_data.json` lưu theo ngôn ngữ trước (`{"en":{key:text},"vi":{...}}`) còn
   `TRANSLATIONS` của Python là key trước; tra cứu bị ngược nên **mọi nhãn hiện khóa thô**
   (`nav_home`, `nav_protection`…). Nay JSON được đảo một lần khi nạp. Đã sửa.
5. **`filter_db.rs`** — `last_update_time == 0` bị in thành `01/01/1970`, Python coi giá trị falsy là
   chuỗi rỗng → nay khớp (`if ts > 0`); đồng thời thời gian chuyển sang `libc::localtime_r`
   cho đúng giờ địa phương như Python (trước đó tính UTC thủ công).
6. **`style.rs`** — 7 dòng CSS có dấu ngoặc kép thừa quanh màu là placeholder `format!`
   (`{{WHITE}}`) khiến GTK báo "Expected a valid color" và bỏ luôn rule đó; thêm một `width:`
   không hợp lệ trong GTK CSS → đổi thành `min-width:`. Đã sửa + thêm rule `scrollbar` toàn cục.
7. **`dns_filters_view.rs`** — icon tìm kiếm được thêm vô điều kiện; Python có `os.path.exists`
   (file này không tồn tại trên máy) → đã thêm kiểm tra tồn tại.
8. **`format!("$1...")` phá mọi lần ghi YAML** — 11 chỗ trong `settings_advanced_view.rs`,
   `settings_network_view.rs`, `apps_config.rs`. Regex crate của Rust đọc `$1true` thành group tên
   `1true`, không tìm thấy group nên thay bằng **chuỗi rỗng**, làm mất giá trị trong proxy.yaml
   (`show_notifications: true` → `show_notifications:`). Đã đổi hết sang dạng có ngoặc
   (`${{1}}` bên trong `format!`, `${1}` trong literal thường). Phát hiện nhờ unit test kiểm tra giá
   trị sau khi thay.
9. **Allowlist không bao giờ render khi mở** — `load_data()` nạp đủ 4285 domain nhưng
   `render_current_page()` chỉ được gọi từ handler tìm kiếm/phân trang, nên danh sách trống dù dữ liệu
   đã có. Python gọi `_render_current_page()` ở cuối `_init_ui()`. Nay hiện 4273 trang web / 535 trang
   phân trang, khớp Python.
10. **Menu tray bị đóng băng** — ksni cache menu; code cũ chỉ lưu cờ trạng thái nên các mục
    enabled/disabled giữ nguyên từ lúc tạo tray (tooltip thì vẫn đổi vì đọc theo yêu cầu). Nay giữ
    `Handle` trong `TRAY_HANDLE` và gọi `update()` từ `update_tray_menu_state()`, khớp hành vi
    `action.setEnabled(...)` của Python.
11. **`--tray` / `--autostart` bị GTK từ chối** (`Unknown option --tray`): hai cờ này được đọc từ
    `std::env::args()` nhưng chưa được lọc trước `run_with_args`. Đã lọc.
12. **App tự TẮT bảo vệ mỗi lần khởi động** (lỗi nghiêm trọng nhất, phát hiện khi kiểm tra file chạy).
    Việc đồng bộ trạng thái bị phản hồi ngược vào công tắc:
    `set_protection_state()` → `ToggleSwitch::set_checked()` → phát `toggled` → `on_switch_toggled()`
    → `backend::set_protection(false)` → app gửi thật `systemctl stop adguard.service` +
    `adguard-cli stop`. Vì lệnh "start" dùng `systemctl restart`, port tụt trong lúc restart và lần dò
    trạng thái 4 giây sau đọc khoảng trống đó thành "đã tắt" → gửi thêm một lệnh `stop` nữa. Kết quả:
    4 lệnh bảo vệ mỗi lần mở app và proxy bị giết (`adguard.service failed`).
    Truy ra bằng `ADGUARD_TRACE=1` + `std::backtrace`. Python chặn đúng chỗ này bằng
    `blockSignals(True/False)`; bản Rust đã mất lớp bảo vệ đó.
    Đã sửa: cờ `Inner::syncing` trong `home_tab.rs` được bật quanh lệnh `set_checked` lập trình, và
    `on_switch_toggled` thoát sớm khi cờ đang bật.
    Kiểm chứng trên máy thật: khởi động nguội giờ chỉ gửi **đúng 1 lệnh**, port giữ ổn định 60 giây+,
    `adguard.service` active; bấm công tắc thật vẫn tắt được bảo vệ và bấm lần nữa bật lại được.
13. **Cùng vòng lặp phản hồi ở `home_network_section.rs`** — `refresh_states()` gọi `set_checked` cho
    3 công tắc HTTP / HTTPS / browser-proxy, mà các công tắc này có handler thật ghi proxy.yaml và gọi
    `adguard-cli`. Hàm này chạy ở mỗi lần làm mới trạng thái (4 giây/lần) và mỗi lần đổi tab, nên cấu
    hình bị ghi lại liên tục. Python bọc `blockSignals` quanh từng `setChecked`; nay bản Rust bật cờ
    `syncing` cho cả lần làm mới và cả 3 handler thoát sớm khi cờ đang bật.

### File chạy
`run.sh` (+ `AdGuard.Rust.desktop`, đã copy ra `~/Desktop/AdGuard Blocker (Rust).desktop`) mở GUI Rust:
khôi phục môi trường phiên khi bấm từ trình quản lý tệp, dừng GUI Python đang giữ khoá single-instance,
chọn binary release (tự build nếu chưa có), ghi log ra `/tmp/adguard-rust-gui.log`.
Cờ: `--release`, `--tray`, `--keep-python`, và chuyển tiếp `--selftest` / `--testcycle` /
`--open-tab` / `--open-page`.

## 5. Chưa xác minh / còn lại

- **Đã test click-through toàn bộ trang con trong phiên này**: Protection 16 trang, Settings 15 trang,
  Home 5 trang — tất cả render đúng nội dung thật. Riêng các **cửa sổ/modal** (filter editor, filtering
  log, stat modals, assistant modal) mới chỉ xác nhận mở được và compile; chưa bấm sâu từng nút bên trong.
- **Tray đã được kiểm tra đầy đủ** trên desktop thật (6 hành động qua D-Bus). **Notification chưa
  kiểm tra**: cần daemon thông báo; code dùng `gtk4::gio::Notification`.
- **`--autostart`/`--tray`** đã test trong phiên hiện tại; **chưa test qua một lần đăng nhập mới**.
- **Cập nhật filter qua mạng, cài đặt chứng chỉ (install_certificate), cấu hình proxy trình duyệt**
  chỉ được gọi một phần (qua `status_snapshot`/`configure_browser_proxy`); chưa chạy đủ các nhánh
  lỗi (không có mạng, quyền bị từ chối…).
- **Warning clippy chưa dọn**: 0 error nhưng còn ~364 dòng cảnh báo (type complexity, `unused Result`
  từ cairo, API GTK deprecated như `FileChooserDialog`, `hide()`/`show()`).
- **Unit test mới phủ các hàm thuần** (i18n, filter_db, stats_worker format/COMPANY_MAP, apps_config,
  YAML replacement). Chưa có test cho parser access.log, quét `.desktop`, hay logic UI.
- **Sai lệch đã biết, không phải lỗi mới**: GTK in `GtkGizmo … (slider) reported min width -2` cho
  vài scrollbar. Đã kiểm chứng là **không do CSS của dự án** (bỏ hết rule scrollbar của mình vẫn còn)
  — đến từ theme Breeze của môi trường; Python/PyQt không có cảnh báo tương đương.

## 6. Sai lệch có chủ đích so với Python (đã ghi trong PORT_PROGRESS.md)

- **Bug trong Python, giữ nguyên hành vi**: `backend.py` không hề `import re`, nên
  `get_auto_language_filters`, `is_https_filtering_enabled`, `get_filtering_mode_info`
  ném `NameError` và luôn trả giá trị mặc định. Rust cài đặt đúng hành vi regex như ý định —
  đây là **sai lệch cố ý**, không tự sửa Python.
- Khung GUI: PyQt6 → GTK4 (không có tương đương 1-1 cho `QMessageBox.exec()` chặn, `QMenu.exec(pos)`,
  delegate checkmark của `QComboBox`, `QDesktopServices`…), các chỗ này dùng cơ chế GTK tương đương
  và đã ghi chú từng trường hợp trong `PORT_PROGRESS.md`.
- `StartupWMClass` trong `.desktop` của bản Rust là `AdGuard` (Python ghi `adguard-modern`),
  vì GTK4 đặt WM class theo application id/tên cửa sổ khác Qt.
- Các cờ `--selftest`, `--testcycle`, `--open-tab`, `--open-page`, `--toggle-key` là công cụ kiểm thử
  của bản Rust, không có trong Python; `--autostart`/`--tray` thì giữ đúng như Python.

## 7. Cách chạy

```bash
cd "/home/pc/Desktop/Softlinux/cachyos software/adguard-blocker-rebuilt"
cargo build --release
./target/release/adguard-blocker-rebuilt              # mở GUI
./target/debug/adguard-blocker-rebuilt --selftest     # kiểm tra liên kết adguard-cli
./target/debug/adguard-blocker-rebuilt --testcycle    # stop -> start qua backend
./target/debug/adguard-blocker-rebuilt --open-tab protection   # mở thẳng 1 tab
sudo bash packaging/setup_adguard_gui.sh              # cài đặt (cần build release trước)
```

## 8. Kết luận

Đã port **69/69 file Python** sang dự án Rust mới, không có stub, build (debug+release)/test/clippy
đều pass (0 error, 18 unit test), GUI mở được và **tất cả 36 trang con + 6 tab chính đều render đúng**;
liên kết xuống `adguard-cli` đã được **xác minh bằng chạy thật** (status, stop/start, đọc DB filter,
ghi proxy.yaml, menu tray 6 hành động qua D-Bus). Trong ba phiên kiểm tra đã phát hiện và sửa
**13 lỗi thật**, trong đó nổi bật là lỗi **app tự tắt bảo vệ mỗi lần khởi động** (vòng lặp phản hồi
qua công tắc, làm chết proxy — đã tìm ra và sửa) và 2 lỗi âm thầm phá dữ liệu (replacement `$1`
làm mất giá trị YAML, và menu tray không cập nhật trạng thái).

Các phần **chưa xác minh** được liệt kê trung thực ở mục 5 — chủ yếu là bấm sâu bên trong các
cửa sổ/modal, notification, và các nhánh lỗi mạng/quyền. Vì vậy **chưa thể coi là đã kiểm chứng
100% mọi ngóc ngách chức năng**, nhưng toàn bộ mã nguồn đã port đủ và phần giao diện chính đã
được chạy và đối chiếu với Python.
