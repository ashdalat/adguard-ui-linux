#!/bin/bash
# Debug sweep: open each sub-page and screenshot it. Test tooling only.
export XDG_RUNTIME_DIR=/run/user/1000
export DISPLAY=:0
export XAUTHORITY=/run/user/1000/xauth_EWDVRy
export GDK_BACKEND=x11
BIN="/home/pc/Desktop/Softlinux/cachyos software/adguard-blocker-rebuilt/target/debug/adguard-blocker-rebuilt"
OUT=/tmp/sweep
mkdir -p "$OUT"

kill_app() {
  for p in $(pgrep -f "target/debug/adguard-blocker-rebuilt" 2>/dev/null); do
    [ "$p" = "$$" ] && continue
    kill "$p" 2>/dev/null
  done
  sleep 2
}

shoot() {
  tab="$1"; page="$2"; tag="$3"
  kill_app
  rm -f "$OUT/$tag.pid"
  if [ -n "$page" ]; then
    ( setsid "$BIN" --open-tab "$tab" --open-page "$page" > "$OUT/$tag.log" 2>&1 & echo $! > "$OUT/$tag.pid" )
  else
    ( setsid "$BIN" --open-tab "$tab" > "$OUT/$tag.log" 2>&1 & echo $! > "$OUT/$tag.pid" )
  fi
  sleep 12
  pid=$(cat "$OUT/$tag.pid" 2>/dev/null)
  win=""
  for w in $(xdotool search --pid "$pid" 2>/dev/null); do
    g=$(xdotool getwindowgeometry "$w" 2>/dev/null | grep Geometry | awk '{print $2}')
    [ "$g" != "1x1" ] && win="$w"
  done
  if [ -n "$win" ]; then import -window "$win" "$OUT/$tag.png" 2>/dev/null; fi
  sz=$(stat -c%s "$OUT/$tag.png" 2>/dev/null || echo 0)
  nf=$(grep -c "NOT FOUND" "$OUT/$tag.log" 2>/dev/null | head -1)
  warn=$(grep -c "Gtk-WARNING" "$OUT/$tag.log" 2>/dev/null | head -1)
  echo "$tag png=$sz notfound=$nf gtkwarn=$warn"
}

kill_app
