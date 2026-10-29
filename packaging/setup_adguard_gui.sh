#!/bin/bash
# Port of AdGuard_CachyOS_GUI_App/setup_adguard_gui.sh for the re-ported Rust build.
# Same steps, same paths and same system integration as the Python installer; the
# only difference is where the GUI itself comes from (a compiled Rust binary
# instead of run_modern_gui.py).
set -e

echo "================================================================="
echo "   TỰ ĐỘNG CÀI ĐẶT ADGUARD CLI & GIAO DIỆN RUST (GTK4)            "
echo "================================================================="
echo

CURRENT_USER="${SUDO_USER:-$USER}"
USER_HOME=$(eval echo "~$CURRENT_USER")
SOURCE_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"
INSTALL_DIR="$USER_HOME/.local/share/adguard_modern_gui"
BIN_SRC="$SOURCE_DIR/target/release/adguard-blocker-rebuilt"

# 1. Cấu hình Sudoers (không hỏi mật khẩu khi bật/tắt dịch vụ)
echo "[1/6] Kiểm tra và cấp quyền sudoers cho dịch vụ AdGuard..."
SUDOERS_FILE="/etc/sudoers.d/adguard_cachyos"
if [ ! -f "$SUDOERS_FILE" ] || ! sudo grep -q "NOPASSWD: ALL" "$SUDOERS_FILE" 2>/dev/null; then
    sudo bash -c "cat << EOF > $SUDOERS_FILE
$CURRENT_USER ALL=(ALL) NOPASSWD: ALL
EOF
chmod 0440 $SUDOERS_FILE" || true
fi

# 2. Kiểm tra các gói phụ thuộc hệ thống (GTK4, rsvg, NSS certutil)
#
# QUAN TRỌNG: KHÔNG cài gói ở đây. `pacman -Sy <pkg>` chỉ đồng bộ database rồi cài
# một gói riêng lẻ — đó là "partial upgrade" và có thể nâng gtk4 mà không nâng
# glib2, khiến mọi app GTK4 trên máy hỏng ABI (đã từng xảy ra: gtk4 4.24.1 cần
# g_timeout_source_new_ns chỉ có ở glib2 2.90.1). Việc đồng bộ gói là của người
# dùng qua `pacman -Syu`; ở đây chỉ báo thiếu và hướng dẫn.
echo "[2/6] Kiểm tra các gói phụ thuộc hệ thống..."
MISSING_DEPS=""
if command -v pacman >/dev/null 2>&1; then
    for p in gtk4 librsvg nss curl ca-certificates; do
        pacman -Q "$p" >/dev/null 2>&1 || MISSING_DEPS="$MISSING_DEPS $p"
    done
    if [ -n "$MISSING_DEPS" ]; then
        echo "  Thiếu gói:$MISSING_DEPS"
        echo "  Cài bằng (KHÔNG dùng -Sy, để tránh lệch phiên bản):"
        echo "      sudo pacman -S --needed$MISSING_DEPS"
        echo "  Nếu máy đã lâu không cập nhật, nên chạy 'sudo pacman -Syu' trước."
    else
        echo "  Đã có đủ: gtk4 librsvg nss curl ca-certificates"
    fi
elif command -v apt-get >/dev/null 2>&1; then
    for p in libgtk-4-1 librsvg2-bin libnss3-tools curl ca-certificates; do
        dpkg -s "$p" >/dev/null 2>&1 || MISSING_DEPS="$MISSING_DEPS $p"
    done
    [ -n "$MISSING_DEPS" ] && echo "  Thiếu gói (cài bằng: sudo apt install$MISSING_DEPS)" || echo "  Đã có đủ gói phụ thuộc"
elif command -v dnf >/dev/null 2>&1; then
    echo "  (dnf) hãy đảm bảo đã cài: gtk4 librsvg2 nss-tools curl ca-certificates"
fi

# 3. Cài đặt AdGuard CLI nếu chưa có
echo "[3/6] Kiểm tra AdGuard CLI..."
if ! command -v adguard-cli >/dev/null 2>&1 && [ ! -f "$USER_HOME/.local/bin/adguard-cli" ] && [ ! -f "/usr/local/bin/adguard-cli" ]; then
    echo "Đang tải và cài đặt AdGuard CLI chính thức từ GitHub..."
    curl -fsSL https://raw.githubusercontent.com/AdguardTeam/AdGuardCLI/release/install.sh | sh -s -- -v || true
fi

CLI_BIN="$USER_HOME/.local/bin/adguard-cli"
[ -f "$CLI_BIN" ] || CLI_BIN="/usr/local/bin/adguard-cli"
[ -f "$CLI_BIN" ] || CLI_BIN="$(which adguard-cli 2>/dev/null || echo 'adguard-cli')"

# 4. Khởi tạo chứng chỉ & tích hợp bảo mật cho tất cả trình duyệt
echo "[4/6] Khởi tạo chứng chỉ HTTPS và tích hợp trình duyệt..."
mkdir -p "$USER_HOME/.local/share/adguard-cli"
echo yes | "$CLI_BIN" cert >/dev/null 2>&1 || true

CA_CERT="$USER_HOME/.local/share/adguard-cli/AdGuard CLI CA.pem"
if [ -f "$CA_CERT" ]; then
    mkdir -p "$USER_HOME/.pki/nssdb"
    certutil -d sql:"$USER_HOME/.pki/nssdb" -A -t "CT,C,C" -n "AdGuard CLI CA" -i "$CA_CERT" 2>/dev/null || true
    certutil -d sql:"$USER_HOME/.pki/nssdb" -M -t "CT,C,C" -n "AdGuard CLI CA" 2>/dev/null || true

    for db in $(find "$USER_HOME/.mullvad" "$USER_HOME/.mozilla" -name "cert9.db" 2>/dev/null); do
        pdir=$(dirname "$db")
        certutil -d sql:"$pdir" -A -t "CT,C,C" -n "AdGuard CLI CA" -i "$CA_CERT" 2>/dev/null || true
        certutil -d sql:"$pdir" -M -t "CT,C,C" -n "AdGuard CLI CA" 2>/dev/null || true
    done

    sudo rm -rf /etc/brave/policies/managed /etc/chromium/policies/managed /etc/opt/chrome/policies/managed 2>/dev/null || true
    sudo mkdir -p /etc/ca-certificates/trust-source/anchors /usr/local/share/ca-certificates /etc/firefox/policies /etc/mullvad-browser/policies /usr/lib/mullvad-browser/distribution/ /opt/mullvad-browser/distribution/
    sudo cp -f "$CA_CERT" /etc/ca-certificates/trust-source/anchors/AdGuard_CLI_CA.crt 2>/dev/null || true
    sudo cp -f "$CA_CERT" /usr/local/share/ca-certificates/AdGuard_CLI_CA.crt 2>/dev/null || true
    sudo update-ca-trust 2>/dev/null || sudo update-ca-certificates 2>/dev/null || true

    sudo bash -c "cat << 'EOF' > /etc/firefox/policies/policies.json
{
  \"policies\": {
    \"Certificates\": {
      \"ImportEnterpriseRoots\": true,
      \"Install\": [\"/etc/ca-certificates/trust-source/anchors/AdGuard_CLI_CA.crt\", \"$CA_CERT\"]
    },
    \"Preferences\": {
      \"security.enterprise_roots.enabled\": true,
      \"security.certerrors.mitm.auto_enable_enterprise_roots\": true,
      \"security.cert_pinning.enforcement_level\": 0
    }
  }
}
EOF" || true
    sudo cp -f /etc/firefox/policies/policies.json /etc/mullvad-browser/policies/ 2>/dev/null || true
    sudo cp -f /etc/firefox/policies/policies.json /usr/lib/mullvad-browser/distribution/ 2>/dev/null || true
    sudo cp -f /etc/firefox/policies/policies.json /opt/mullvad-browser/distribution/ 2>/dev/null || true

    for d in /opt/mullvad-browser/browser/defaults/preferences /usr/lib/mullvad-browser/browser/defaults/preferences /usr/lib/firefox/browser/defaults/preferences /usr/lib64/firefox/browser/defaults/preferences; do
        if [ -d "$d" ]; then
            sudo bash -c "cat << 'PREF_EOF' > \"$d/99-adguard.js\"
pref(\"security.enterprise_roots.enabled\", true);
pref(\"security.certerrors.mitm.auto_enable_enterprise_roots\", true);
pref(\"security.cert_pinning.enforcement_level\", 0);
PREF_EOF" || true
        fi
    done
fi

EXCL_FILE="$USER_HOME/.local/share/adguard-cli/https_exclusions.txt"
if [ -f "$EXCL_FILE" ]; then
    grep -q "^adguard.com$" "$EXCL_FILE" || echo -e "\nadguard.com\nwww.adguard.com" >> "$EXCL_FILE"
fi

# 5. Cài đặt ứng dụng GUI (binary + assets) vào ~/.local/share/adguard_modern_gui
echo "[5/6] Cài đặt giao diện Rust vào $INSTALL_DIR..."
if [ ! -x "$BIN_SRC" ]; then
    echo "Chưa có binary release: $BIN_SRC"
    echo "Hãy build trước:  cd \"$SOURCE_DIR\" && cargo build --release"
    exit 1
fi
mkdir -p "$INSTALL_DIR/bin"
cp -f "$BIN_SRC" "$INSTALL_DIR/bin/adguard-blocker-rebuilt"
chmod +x "$INSTALL_DIR/bin/adguard-blocker-rebuilt"
cp -rf "$SOURCE_DIR/assets" "$INSTALL_DIR/" 2>/dev/null || true
cp -f "$SOURCE_DIR/adguard_blocker_icon.png" "$INSTALL_DIR/" 2>/dev/null || true
cp -f "$SOURCE_DIR/packaging/start_adguard_service.sh" "$INSTALL_DIR/" 2>/dev/null || true
chmod +x "$INSTALL_DIR/start_adguard_service.sh" 2>/dev/null || true

# The whole script runs under sudo, so everything copied above landed as root.
# Give the install directory back to the desktop user, otherwise the launcher
# cannot rewrite its cached data and dconf/theming break for the GUI.
chown -R "$CURRENT_USER" "$INSTALL_DIR" 2>/dev/null || true

mkdir -p "$USER_HOME/.local/bin"
cat << EOF > "$USER_HOME/.local/bin/adguard-gui"
#!/bin/bash
exec "$INSTALL_DIR/bin/adguard-blocker-rebuilt" "\$@"
EOF
chmod +x "$USER_HOME/.local/bin/adguard-gui"

DESKTOP_DIR="$USER_HOME/Desktop"
[ ! -d "$DESKTOP_DIR" ] && DESKTOP_DIR="$USER_HOME/Bàn làm việc"
mkdir -p "$DESKTOP_DIR"

rm -f "$DESKTOP_DIR/AdGuard_UI.desktop" "$DESKTOP_DIR/AdGuard_Modern.desktop"

cat << EOF > "$DESKTOP_DIR/AdGuard.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=AdGuard
Comment=Trình chặn quảng cáo AdGuard cho Linux
Exec=$USER_HOME/.local/bin/adguard-gui
Icon=$INSTALL_DIR/adguard_blocker_icon.png
Terminal=false
Categories=Network;Security;Utility;
StartupNotify=true
StartupWMClass=AdGuard
EOF
chmod +x "$DESKTOP_DIR/AdGuard.desktop"
if command -v gio >/dev/null 2>&1; then
    gio trust "$DESKTOP_DIR/AdGuard.desktop" 2>/dev/null || true
fi

mkdir -p "$USER_HOME/.local/share/applications"
cp "$DESKTOP_DIR/AdGuard.desktop" "$USER_HOME/.local/share/applications/adguard-gui.desktop"

mkdir -p "$USER_HOME/.config/autostart"
rm -f "$USER_HOME/.config/autostart/adguard_gui.desktop"
cat << EOF > "$USER_HOME/.config/autostart/adguard-gui.desktop"
[Desktop Entry]
Name=AdGuard (Autostart)
Comment=AdGuard AdBlocker Modern GUI
Exec=$USER_HOME/.local/bin/adguard-gui --autostart
Icon=$INSTALL_DIR/adguard_blocker_icon.png
Terminal=false
Type=Application
Categories=Network;Security;
X-GNOME-Autostart-enabled=true
EOF

systemctl --user stop adguard.service 2>/dev/null || true
systemctl --user disable adguard.service 2>/dev/null || true
rm -f "$USER_HOME/.config/systemd/user/adguard.service" 2>/dev/null || true
systemctl --user daemon-reload 2>/dev/null || true

sudo bash -c "cat << SVCEOF > /etc/systemd/system/adguard.service
[Unit]
Description=AdGuard CLI Blocker Service
After=network.target network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=$INSTALL_DIR/start_adguard_service.sh
ExecStop=/usr/local/bin/adguard-cli stop
TimeoutStartSec=60
Environment=HOME=$USER_HOME

[Install]
WantedBy=multi-user.target
SVCEOF"

sudo systemctl daemon-reload 2>/dev/null || true
sudo systemctl enable adguard.service 2>/dev/null || true
sudo systemctl start adguard.service 2>/dev/null || true

# 6. Kích hoạt AdGuard và khởi chạy giao diện
echo "[6/6] Khởi chạy AdGuard Blocker và mở giao diện..."
sudo "$CLI_BIN" start >/dev/null 2>&1 || "$CLI_BIN" start >/dev/null 2>&1 || true

# Do NOT start the GUI from here: this script runs as root, so a GUI started now
# would belong to root (wrong HOME, no session bus, no tray, no theme). Launch it
# as the desktop user, detached from this shell, and only when a session exists.
if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
    echo "  (GUI sẽ do người dùng $SUDO_USER tự mở: lệnh 'adguard-gui')"
    if [ -S "/run/user/$(id -u "$SUDO_USER")/bus" ]; then
        su -l "$SUDO_USER" -c 'nohup "$HOME/.local/bin/adguard-gui" >/dev/null 2>&1 &' 2>/dev/null || true
    fi
fi

echo
echo "================================================================="
echo "              CÀI ĐẶT ADGUARD GUI THÀNH CÔNG!                   "
echo "================================================================="
echo " • Lệnh chạy nhanh: adguard-gui"
echo " • Biểu tượng ứng dụng: Đã thêm vào Menu ứng dụng và Khay hệ thống"
echo " • Khởi động cùng máy: Tự động chạy ngầm ở chế độ System Tray"
echo "================================================================="
