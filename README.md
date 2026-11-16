# AdGuard Blocker UI for Linux

An unofficial, high-performance native **GTK4** desktop frontend (wrapper) for the official **AdGuard CLI** (`adguard-cli`) on Linux. Re-architected in **Rust** to replace legacy Electron/Python frontends with native speed, low memory usage, and direct IPC communication.

> [!NOTE]
> This application is an independent open-source GUI frontend. It connects to and manages the official `adguard-cli` service daemon.

---

## ✨ Features

- **Protection Controls:** One-click toggle for ad blocking, tracking protection, and browsing security.
- **Live Filtering Log:** Real-time stream of HTTP/HTTPS and DNS requests with filter rule matches and block/allow details.
- **HTTPS Filtering:** Manage AdGuard Root CA certificate generation, installation, and inspection.
- **DNS Protection:** Configure custom DNS servers (DoH, DoT, DNSCrypt) and DNS filtering rules.
- **Extensions & Userscripts:** Manage custom user scripts and browser extensions.
- **Per-App Filtering:** Granular control over which applications are filtered or bypassed.
- **System Tray:** Background tray icon with quick pause/resume toggles and status display.

---

## 🚀 Building & Running

### Prerequisites
- **Rust & Cargo** (1.75+)
- **GTK4** (`gtk4`, `cairo`, `pango`) development packages
- **SQLite3** library (`libsqlite3-dev` / `sqlite`)
- **adguard-cli** backend service installed on your system

### Build from Source
```bash
# Clone repository
git clone https://github.com/ashdalat/adguard-blocker.git
cd adguard-blocker

# Build release binary
cargo build --release

# Run
./target/release/adguard-blocker-rebuilt
```

### Run Script
```bash
./run.sh
```

---

## 📦 Packaging & Installation
Automated setup scripts are located in the `packaging/` directory:
- `packaging/setup_adguard_gui.sh`: Installs the binary, desktop entry, and icon.
- `packaging/start_adguard_service.sh`: Helper to ensure `adguard-cli` systemd service is active.
- `packaging/uninstall_adguard_gui.sh`: Cleanly removes the GUI and desktop entries.

---

## 📄 License

GPL-3.0 or MIT. Not affiliated with AdGuard Software Ltd.
