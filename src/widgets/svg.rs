// SVG icon rendering. GTK4 renders SVG only through a gdk-pixbuf loader, which is
// not installed on this system, so icons are rasterized with `rsvg-convert`
// (present at /usr/sbin/rsvg-convert) and cached as textures. If the tool is
// unavailable, callers fall back to a symbolic icon name.
use gtk4::gdk;
use gtk4::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

static CACHE: std::sync::LazyLock<Mutex<HashMap<(PathBuf, u32), Option<gdk::Texture>>>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn svg_texture(path: &Path, size: u32) -> Option<gdk::Texture> {
    let key = (path.to_path_buf(), size);
    {
        let c = CACHE.lock().unwrap();
        if let Some(hit) = c.get(&key) {
            return hit.clone();
        }
    }
    let rendered = render_svg(path, size);
    let texture = rendered.and_then(|png| gdk::Texture::from_filename(&png).ok());
    CACHE.lock().unwrap().insert(key, texture.clone());
    texture
}

fn render_svg(path: &Path, size: u32) -> Option<PathBuf> {
    let out = cache_dir()?.join(format!(
        "{}-{}.png",
        path.file_stem()?.to_string_lossy(),
        size
    ));
    if out.exists() {
        return Some(out);
    }
    let ok = Command::new("rsvg-convert")
        .arg("-w")
        .arg(size.to_string())
        .arg("-h")
        .arg(size.to_string())
        .arg("-o")
        .arg(&out)
        .arg(path)
        .status()
        .ok()?
        .success();
    if ok {
        Some(out)
    } else {
        None
    }
}

fn cache_dir() -> Option<PathBuf> {
    let dir = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| crate::constants::home_dir().join(".cache"))
        .join("adguard-blocker-rebuilt/svg");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Port of QSvgWidget: an image widget showing an SVG icon at a fixed size.
pub fn svg_image(icon_filename: &str, size: u32) -> gtk4::Image {
    let img = gtk4::Image::new();
    let path = crate::constants::icons_dir().join(icon_filename);
    match svg_texture(&path, size) {
        Some(tex) => {
            img.set_paintable(Some(&tex));
        }
        None => {
            // Fallback: symbolic icon name so the layout still shows something.
            img.set_icon_name(Some("image-symbolic"));
        }
    }
    img.set_size_request(size as i32, size as i32);
    img.add_css_class("svg-icon");
    img
}
