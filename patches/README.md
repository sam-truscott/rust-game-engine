# Patches to dependencies

## miniquad 0.4.11 on Windows ARM64

`miniquad-0.4.11-aarch64.patch` changes four `cfg` guards in `src/native/windows.rs` from
`target_arch = "x86_64"` / `"x86"` to `target_pointer_width = "64"` / `"32"`, and adds an `#![allow]`
at the top of `src/lib.rs` to silence two upstream warnings.

Why: with the original guards neither branch compiles on `aarch64-pc-windows-msvc`, so
`SetWindowLongPtrA(wnd, GWLP_USERDATA, ...)` is never called. The window procedure reads that pointer,
gets 0, and hands every message to `DefWindowProc`: no keyboard or mouse input reaches the game, and
closing the window destroys it without ending the process (the main loop keeps running).

How it is used: Cargo cannot apply a patch to a dependency, only point at a source folder. `Cargo.toml`
has `[patch.crates-io] miniquad = { path = "patched/miniquad" }`, and `tools/patch-miniquad.sh` builds that
folder (not committed, listed in `.gitignore`) from the published crate plus this patch. Run it once after
cloning; until you do, `cargo build` fails with "patch location ... does not contain packages matching
`miniquad`". The script takes the crate from cargo's download cache, or from crates.io if it is not there,
and refuses it unless it matches the checksum crates.io publishes (pinned in the script). Cargo.lock cannot
supply that checksum: a crate overridden by `[patch]` is recorded there without one.

To drop it: once a miniquad release fixes this, delete the `[patch.crates-io]` entry, `patched/`, this
patch and the script, and update macroquad/miniquad.
