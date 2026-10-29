#!/bin/bash
# One-off: remove the old Python GUI install and put the Rust build in its place.
# Run as the normal user (not root).
set -u

echo "=== [1/6] Dừng tiến trình cũ ==="
# Match on the executable path so this script's own command line cannot match.
for p in $(pgrep -f 'adguard_modern_gui/run_modern_gui\.py' 2>/dev/null); do
    kill "$p" 2>/dev/null
done
for p in $(pgrep -f 'release/adguard-blocker-rebuilt' 2>/dev/null); do
    kill "$p" 2>/dev/null
done
sleep 2
pgrep -f 'run_modern_gui\.py' >/dev/null 2>&1 && echo "  CÒN tiến trình Python" || echo "  Python GUI đã dừng"

echo
echo "=== [2/6] Xóa app Python cũ ==="
if [ -d "$HOME/.local/share/adguard_modern_gui" ]; then
    du -sh "$HOME/.local/share/adguard_modern_gui"
    rm -rf "$HOME/.local/share/adguard_modern_gui"
    echo "  đã xóa ~/.local/share/adguard_modern_gui"
else
    echo "  (không tồn tại)"
fi

echo
echo "=== [3/6] Xóa launcher + desktop entries cũ (GIỮ nguyên VPN) ==="
rm -f "$HOME/.local/bin/adguard-gui"
rm -f "$HOME/.local/share/applications/adguard-gui.desktop"
rm -f "$HOME/.local/share/applications/adguard-modern.desktop"
rm -f "$HOME/.local/share/applications/AdGuard_UI.desktop"
rm -f "$HOME/.config/autostart/adguard-gui.desktop"
rm -f "$HOME/Desktop/AdGuard.desktop"
rm -f "$HOME/Bàn làm việc/AdGuard.desktop"
echo "  đã xóa"

echo
echo "=== [4/6] Kiểm tra VPN còn nguyên ==="
for f in \
    "$HOME/.config/autostart/adguard-vpn.desktop" \
    "$HOME/.local/share/applications/adguard-vpn.desktop" \
    "$HOME/.local/share/applications/AdGuard_VPN_UI.desktop" \
    "$HOME/Desktop/AdGuard VPN.desktop" ; do
    [ -e "$f" ] && echo "  OK  $f" || echo "  !!  THIẾU $f"
done
echo "  VPN binary: $(command -v adguardvpn-cli || echo '/opt/adguardvpn_cli/adguardvpn-cli')"

echo
echo "=== [5/6] Kiểm tra service systemd ==="
systemctl is-enabled adguard.service 2>/dev/null || true
systemctl is-active adguard.service 2>/dev/null || true
grep -n ExecStart /etc/systemd/system/adguard.service 2>/dev/null || sudo -n grep -n ExecStart /etc/systemd/system/adguard.service 2>/dev/null || true

echo
echo "=== [6/6] Xong phần xóa ==="
echo "Bước cài đặt: chạy packaging/setup_adguard_gui.sh"
