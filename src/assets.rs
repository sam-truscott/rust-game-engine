use std::borrow::Cow;

use rust_embed::RustEmbed;

/// Debug builds read `assets/` from disk, so edits show up on restart; release builds embed it.
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Embedded;

pub fn bytes(path: &str) -> Option<Cow<'static, [u8]>> {
    Embedded::get(path).map(|file| file.data)
}

pub fn text(path: &str) -> Option<String> {
    bytes(path).and_then(|data| String::from_utf8(data.into_owned()).ok())
}
