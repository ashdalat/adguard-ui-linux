#!/bin/bash
# =============================================================================
#  Chạy AdGuard Blocker (bản Rust/GTK4) — dự án adguard-blocker-rebuilt
#
#  Cách dùng:
#     ./run.sh                 mở giao diện bình thường
#     ./run.sh --release       dùng bản release (mặc định: tự chọn)
#     ./run.sh --tray          chạy ẩn xuống khay hệ thống
#     ./run.sh --keep-python   KHÔNG dừng bản Python cũ (nếu nó đang chạy)
#     ./run.sh --selftest      kiểm tra liên kết adguard-cli rồi thoát
#     ./run.sh --testcycle     test tắt/bật bảo vệ qua backend rồi thoát
#
#  Bất kỳ tham số nào khác được chuyển nguyên cho app.
# =============================================================================
set -u

PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$PROJECT_DIR" || exit 1

# --- 1. Môi trường phiên đồ họa --------------------------------------------
# Khi bấm đúp từ trình quản lý tệp, các biến này có thể chưa có; lấy lại từ
# tiến trình phiên đang chạy để cửa sổ mở đúng màn hình.
UID_NUM="$(id -u)"
if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
    [ -d "/run/user/$UID_NUM" ] && export XDG_RUNTIME_DIR="/run/user/$UID_NUM"
fi
if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ] && [ -S "/run/user/$UID_NUM/bus" ]; then
    export DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$UID_NUM/bus"
fi
if [ -z "${WAYLAND_DISPLAY:-}" ] && [ -e "/run/user/$UID_NUM/wayland-0" ]; then
    export WAYLAND_DISPLAY="wayland-0"
fi
if [ -z "${DISPLAY:-}" ] && [ -e "/tmp/.X11-unix/X0" ]; then
    export DISPLAY=":0"
fi
# XAUTHORITY: lấy từ một tiến trình phiên nếu chưa có (cần cho Xwayland)
if [ -z "${XAUTHORITY:-}" ]; then
    for p in $(pgrep -x plasmashell 2>/dev/null) $(pgrep -x kwin_wayland 2>/dev/null); do
        xa="$(tr '\0' '\n' < "/proc/$p/environ" 2>/dev/null | sed -n 's/^XAUTHORITY=//p' | head -1)"
        if [ -n "$xa" ] && [ -e "$xa" ]; then
            export XAUTHORITY="$xa"
            break
        fi
    done
fi

# --- 2. Tham số ------------------------------------------------------------
KEEP_PYTHON=0
WANT_RELEASE=0
APP_ARGS=()
for arg in "$@"; do
    case "$arg" in
        --keep-python) KEEP_PYTHON=1 ;;
        --release)     WANT_RELEASE=1 ;;
        *)             APP_ARGS+=("$arg") ;;
    esac
done

# --- 3. Tìm binary ---------------------------------------------------------
BIN=""
if [ "$WANT_RELEASE" -eq 1 ]; then
    BIN="$PROJECT_DIR/target/release/adguard-blocker-rebuilt"
else
    # Ưu tiên bản release (mượt, có sẵn), rồi tới debug
    for cand in "$PROJECT_DIR/target/release/adguard-blocker-rebuilt" \
                "$PROJECT_DIR/target/debug/adguard-blocker-rebuilt"; do
        [ -x "$cand" ] && BIN="$cand" && break
    done
fi

if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
    echo "Chưa có binary. Đang build bản release (lần đầu có thể mất vài phút)..."
    if ! cargo build --release; then
        echo "Build thất bại. Chạy 'cargo build --release' trong $PROJECT_DIR để xem lỗi." >&2
        exit 1
    fi
    BIN="$PROJECT_DIR/target/release/adguard-blocker-rebuilt"
fi

# --- 4. Giải phóng khoá single-instance ------------------------------------
# Bản Rust dùng CÙNG abstract socket với bản Python, nên nếu bản Python còn
# chạy thì bản Rust sẽ chỉ gửi tín hiệu "SHOW" và thoát ngay.
PY_RUNNING="$(pgrep -f "run_modern_gui\.py" 2>/dev/null | head -1)"
if [ -n "$PY_RUNNING" ]; then
    if [ "$KEEP_PYTHON" -eq 1 ]; then
        echo "Lưu ý: bản Python (pid $PY_RUNNING) đang chạy và đang giữ khoá."
        echo "       Bản Rust sẽ chỉ yêu cầu bản Python hiện cửa sổ rồi thoát."
    else
        echo "Đang dừng bản Python cũ (pid $PY_RUNNING) để bản Rust chạy được..."
        for p in $PY_RUNNING; do kill "$p" 2>/dev/null; done
        sleep 2
    fi
fi

# Bản Rust còn sót từ lần trước.
# Lưu ý: KHÔNG dùng `pgrep -f adguard-blocker-rebuilt` trực tiếp — tên thư mục dự
# án cũng chứa chuỗi đó nên nó sẽ khớp cả shell đang chạy script này. Chỉ kill
# tiến trình mà dòng lệnh thật sự trỏ tới file binary trong target/.
kill_stale_app() {
    local p cmd
    for p in $(pgrep -f "adguard-blocker-rebuilt" 2>/dev/null); do
        [ "$p" = "$$" ] && continue
        cmd="$(tr '\0' ' ' < "/proc/$p/cmdline" 2>/dev/null)"
        case "$cmd" in
            *target/*/adguard-blocker-rebuilt*) kill "$p" 2>/dev/null ;;
        esac
    done
}
kill_stale_app
sleep 1

# --- 5. Chạy ---------------------------------------------------------------
echo "Đang mở AdGuard Blocker (Rust): $BIN"
echo "Proxy adguard-cli: $(ss -ltn 2>/dev/null | grep -q ':3129' && echo 'đang chạy (3129)' || echo 'chưa chạy')"

case " ${APP_ARGS[*]:-} " in
    *" --selftest "*|*" --testcycle "*)
        # Chế độ kiểm tra: chạy trực tiếp và in kết quả
        "$BIN" "${APP_ARGS[@]}"
        exit $?
        ;;
esac

# GUI thường: chạy nền và gỡ khỏi terminal để đóng terminal không tắt app
nohup "$BIN" "${APP_ARGS[@]}" >/tmp/adguard-rust-gui.log 2>&1 &
APP_PID=$!
sleep 3

if kill -0 "$APP_PID" 2>/dev/null; then
    echo "Đã mở (pid $APP_PID). Nhật ký: /tmp/adguard-rust-gui.log"
else
    echo "App thoát ngay. Nhật ký dưới đây:" >&2
    cat /tmp/adguard-rust-gui.log >&2
    exit 1
fi
