use std::borrow::Cow;
use std::cell::RefCell;
use std::path::PathBuf;

use rust_embed::RustEmbed;

/// The bundled game. Debug builds read it from disk, so edits show up on restart; release builds
/// embed it. `use_dir` reads a game from any folder instead.
#[derive(RustEmbed)]
#[folder = "games/pub/"]
#[exclude = "play/*"]
struct Embedded;

thread_local! {
    static DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Reads the game from this folder (the one holding `game.lua`) instead of the bundled one. It
/// applies to the calling thread only, which is the game's thread and, in tests, the test's own.
pub fn use_dir(dir: impl Into<PathBuf>) {
    DIR.with(|d| *d.borrow_mut() = Some(dir.into()));
}

/// Goes back to the bundled game.
pub fn use_bundled() {
    DIR.with(|d| *d.borrow_mut() = None);
}

/// Paths come from room and script files, so ones that leave the game folder are refused.
pub fn bytes(path: &str) -> Option<Cow<'static, [u8]>> {
    if path.starts_with('/') || path.contains(['\\', ':']) || path.contains("..") {
        return None;
    }
    match DIR.with(|d| d.borrow().clone()) {
        Some(dir) => std::fs::read(dir.join(path)).ok().map(Cow::Owned),
        None => Embedded::get(path).map(|file| file.data),
    }
}

pub fn text(path: &str) -> Option<String> {
    bytes(path).and_then(|data| String::from_utf8(data.into_owned()).ok())
}
