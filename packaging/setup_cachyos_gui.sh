#!/bin/bash
# Port of AdGuard_CachyOS_GUI_App/setup_cachyos_gui.sh for the Rust build.
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
chmod +x "$DIR/setup_adguard_gui.sh"
exec "$DIR/setup_adguard_gui.sh" "$@"
