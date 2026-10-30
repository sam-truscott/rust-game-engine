# Local patch to miniquad 0.4.11

Upstream: https://github.com/not-fl3/miniquad (crates.io release 0.4.11, unmodified apart from below).

`src/native/windows.rs`, four `cfg` guards (two near line 274 for the window style, two near line 1241
for the window state pointer) changed from `target_arch = "x86_64"` / `"x86"` to
`target_pointer_width = "64"` / `"32"`.

Why: with the original guards neither branch compiles on `aarch64-pc-windows-msvc`, so
`SetWindowLongPtrA(wnd, GWLP_USERDATA, ...)` is never called. The window procedure reads that pointer, gets 0,
and hands every message to `DefWindowProc`: no keyboard or mouse input reaches the game, and closing the
window destroys it without ending the process (the main loop keeps running).

Drop this directory and the `[patch.crates-io]` entry in `Cargo.toml` once a miniquad release fixes it.

Also: `#![allow(unused_assignments, dead_code)]` added at the top of `src/lib.rs`, to silence two upstream
warnings that would otherwise print on every build.
