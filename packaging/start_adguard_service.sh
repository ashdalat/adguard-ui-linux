#!/bin/bash
# ==========================================================
# AdGuard Blocker Service Startup Script
# ==========================================================

USER_HOME="${HOME:-/home/pc}"
# Ưu tiên binary từ official installer (/usr/local/bin hoặc /opt) vì có root helper tích hợp
ADGUARD_BIN="/usr/local/bin/adguard-cli"
[ -f "$ADGUARD_BIN" ] || ADGUARD_BIN="$USER_HOME/.local/bin/adguard-cli"
[ -f "$ADGUARD_BIN" ] || ADGUARD_BIN="$(which adguard-cli 2>/dev/null || echo 'adguard-cli')"

if "$ADGUARD_BIN" status 2>&1 | grep -q "is running" || sudo -n "$ADGUARD_BIN" status 2>&1 | grep -q "is running"; then
    echo "AdGuard proxy server is already running."
    exit 0
fi

# Chờ 3 giây để hệ thống mạng sẵn sàng sau khi mở máy
sleep 3

MAX_RETRIES=4
RETRY_DELAY=5
COUNT=0

echo "Starting AdGuard Blocker Service..."

while [ $COUNT -lt $MAX_RETRIES ]; do
    if "$ADGUARD_BIN" status 2>&1 | grep -q "is running" || sudo -n "$ADGUARD_BIN" status 2>&1 | grep -q "is running"; then
        echo "AdGuard proxy server is running!"
        exit 0
    fi

    # Dọn dẹp process hoặc port bị treo nếu có
    sudo -n fuser -k 3129/tcp 2>/dev/null || true
    sudo -n rm -f "$USER_HOME/.local/share/adguard-cli/"*.pid 2>/dev/null || true

    # Khởi động adguard-cli (cần sudo để RootHelper hoạt động đúng)
    sudo -n "$ADGUARD_BIN" start >/tmp/adguard_start.log 2>&1 || "$ADGUARD_BIN" start >>/tmp/adguard_start.log 2>&1
    RET=$?

    sleep 1
    if [ $RET -eq 0 ] || "$ADGUARD_BIN" status 2>&1 | grep -q "is running" || sudo -n "$ADGUARD_BIN" status 2>&1 | grep -q "is running"; then
        echo "AdGuard proxy server started successfully!"
        # Áp dụng cấu hình proxy cho GNOME nếu có
        if command -v gsettings >/dev/null 2>&1; then
            gsettings set org.gnome.system.proxy mode manual 2>/dev/null || true
            gsettings set org.gnome.system.proxy.http host '127.0.0.1' 2>/dev/null || true
            gsettings set org.gnome.system.proxy.http port 3129 2>/dev/null || true
            gsettings set org.gnome.system.proxy.https host '127.0.0.1' 2>/dev/null || true
            gsettings set org.gnome.system.proxy.https port 3129 2>/dev/null || true
        fi
        exit 0
    fi

    echo "Attempt $((COUNT+1))/$MAX_RETRIES failed. Retrying in ${RETRY_DELAY}s..."
    sleep $RETRY_DELAY
    COUNT=$((COUNT+1))
done

echo "Failed to start AdGuard proxy after $MAX_RETRIES attempts."
cat /tmp/adguard_start.log 2>/dev/null
exit 1
