// Port of adguard_modern/constants.py
use std::path::PathBuf;
use std::sync::OnceLock;

// Exact color palette sampled from AdGuard Windows App
pub const COLOR_MAIN_BG: &str = "#1E1E1E";
pub const COLOR_SIDEBAR_BG: &str = "#19191A";
pub const COLOR_SIDEBAR_HOVER: &str = "#242427";
pub const COLOR_SIDEBAR_ACTIVE: &str = "#2C2C30";
pub const COLOR_BORDER: &str = "#55555A";

pub const COLOR_TEXT_WHITE: &str = "#FFFFFF";
pub const COLOR_TEXT_MUTED: &str = "#8E8E93";
pub const COLOR_TEXT_SUBTITLE: &str = "#9E9EA3";

pub const COLOR_ADGUARD_GREEN: &str = "#5EBB6A";
pub const COLOR_ADGUARD_GREEN_HOVER: &str = "#68C974";

// Toggle Switch Colors
pub const COLOR_SWITCH_TRACK_ON: &str = "#5EBB6A";
pub const COLOR_SWITCH_TRACK_OFF: &str = "#EF5A3B"; // Coral orange/red when disabled
pub const COLOR_SWITCH_ORANGE: &str = "#EF5A3B";
pub const COLOR_SWITCH_KNOB_ON: &str = "#FFFFFF";
pub const COLOR_SWITCH_KNOB_OFF: &str = "#FFFFFF";

pub const APP_NAME: &str = "adguard";
pub const APP_VERSION: &str = "8.0 (Modern Linux)";

// Python: BASE_DIR = dirname(dirname(abspath(__file__))) — the app root that
// contains assets/ and adguard_blocker_icon.png. For the Rust binary this is
// resolved from the executable location, with the source tree as dev fallback.
static BASE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn base_dir() -> &'static PathBuf {
    BASE_DIR.get_or_init(|| {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                // installed layout: <base_dir>/bin/<exe> or <base_dir>/assets
                for cand in [parent.to_path_buf(), parent.join("..")] {
                    if cand.join("assets").join("icons").is_dir() || cand.join("adguard_blocker_icon.png").is_file() {
                        return cand;
                    }
                }
            }
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    })
}

pub fn assets_dir() -> PathBuf {
    base_dir().join("assets")
}

pub fn icons_dir() -> PathBuf {
    assets_dir().join("icons")
}

pub fn icon_path(name: &str) -> PathBuf {
    icons_dir().join(name)
}

pub fn logo_path() -> PathBuf {
    base_dir().join("adguard_blocker_icon.png")
}

pub fn home_dir() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/tmp"))
}
