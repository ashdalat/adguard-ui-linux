#!/bin/bash
# Port of AdGuard_CachyOS_GUI_App/uninstall_adguard_gui.sh for the Rust build.
# Same paths as the Python uninstaller; the process names differ because the
# GUI is a compiled binary instead of run_modern_gui.py.
set -e

echo "================================================================="
echo "            GỠ CÀI ĐẶT ADGUARD MODERN GUI (RUST)                 "
echo "================================================================="
echo

CURRENT_USER="${SUDO_USER:-$USER}"
USER_HOME=$(eval echo "~$CURRENT_USER")

echo "[1/4] Đang dừng tiến trình GUI đang chạy..."
pkill -9 -f "adguard-blocker-rebuilt" 2>/dev/null || true
pkill -9 -f "run_modern_gui.py" 2>/dev/null || true
pkill -9 -f "adguard_gui.py" 2>/dev/null || true

echo "[2/4] Đang xóa thư mục ứng dụng cài đặt..."
rm -rf "$USER_HOME/.local/share/adguard_modern_gui"
rm -rf "$USER_HOME/.local/share/adguard_gui"

echo "[3/4] Đang xóa lệnh chạy nhanh và file khởi động..."
rm -f "$USER_HOME/.local/bin/adguard-gui"
rm -f "$USER_HOME/.local/share/applications/adguard-gui.desktop"
rm -f "$USER_HOME/.config/autostart/adguard-gui.desktop"

echo "[4/4] Hoàn tất gỡ bỏ!"
echo "================================================================="
echo "       ĐÃ GỠ CÀI ĐẶT ADGUARD GUI HOÀN TOÀN KHỎI MÁY!            "
echo "================================================================="
