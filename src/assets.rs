use std::borrow::Cow;

use rust_embed::RustEmbed;

/// Debug builds read `assets/` from disk, so edits show up on restart; release builds embed it.
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Embedded;

/// Paths come from room and script files, so ones that leave `assets/` are refused.
pub fn bytes(path: &str) -> Option<Cow<'static, [u8]>> {
    if path.starts_with('/') || path.contains(['\\', ':']) || path.contains("..") {
        return None;
    }
    Embedded::get(path).map(|file| file.data)
}

pub fn text(path: &str) -> Option<String> {
    bytes(path).and_then(|data| String::from_utf8(data.into_owned()).ok())
}
