# Game

2D point-and-click adventure engine and game in the style of Thimbleweed Park (own engine, story, art and puzzles). Rust (edition 2024), macroquad for window/input/drawing, mlua (vendored Lua 5.4) for scripting. Crate name `game`.

Everything is on `master` (remote `origin`, git@github.com:sam-truscott/game.git). The engine was ported from Java/LibGDX/LuaJ (Maven, Eclipse); the last Java commit is `aca1a71`, so `git show aca1a71:src/main/java/com/wumpus/game/<File>.java` shows any original file.

## Environment (Windows 11 on ARM64)

- Rust is installed in `C:\Users\Sam\.cargo\bin`, default host `aarch64-pc-windows-msvc`, so builds and the game are **native ARM64**. This needs the "MSVC Build Tools for ARM64/ARM64EC" component of the Visual Studio Build Tools; without it linking fails with a confusing `link: extra operand` error (cargo falls back to Git Bash's own `link` command).
- Shell state does not persist between tool calls and `cargo` is not on `PATH` in Git Bash, so start commands with `export PATH="$PATH:/c/Users/Sam/.cargo/bin"`. The PowerShell tool may be unavailable; Git Bash can still run `powershell.exe` (fine for generating art with System.Drawing), but see "Verifying" before using it to drive the game.
- Mlua compiles Lua from C source, which needs the MSVC `cl.exe`; that already works here.

## Build, test, run

```
bash tools/patch-miniquad.sh   # once after cloning: builds patched/miniquad (see "The miniquad patch")
cargo build            # debug; assets are read from assets/ on disk, so edits show up after pressing R
cargo test             # unit and integration tests, no GL needed
cargo clippy --all-targets   # kept warning-free
cargo fmt              # default rustfmt settings
cargo build --release  # ~1.5 MB exe with assets/ embedded (rust-embed); runs from any directory
target/debug/game.exe
```

- Controls: pick a verb on the bar at the top, then click what to use it on. With "Walk to" (the default, and what every click resets to) a click just walks to where you clicked, even onto an object, so you can walk behind things. Carried items sit in the strip along the bottom: a plain click on one looks at it; pick "Use", click an item, then click a hotspot or another item to use it with that (clicking the same item again uses it on its own; clicking a verb, or somewhere with nothing there, puts it back). While a script runs (walking to something, talking) choosing still works, so no click is lost: verb buttons always do, and with "Use" selected so does picking a carried item; clicks that would start something (a hotspot, the room, an item with another verb) are ignored until it is over. **R** restarts, **F1** toggles an overlay of the authored (green) and margin-adjusted (yellow) walk areas.
- Debug builds read `assets/` from the path baked in at compile time, release builds embed it, so a release exe is self-contained.

## The miniquad patch (do not remove casually)

macroquad sits on miniquad 0.4.11 (the newest release), which only stores its window state pointer on x86/x86_64. On aarch64 that call is compiled out, so **no keyboard or mouse input reaches the game and closing the window never ends the process**.

The fix is a small patch, not a copy of the crate: `patches/miniquad-0.4.11-aarch64.patch` changes four `cfg` guards to `target_pointer_width` (plus an `#![allow]` for two upstream warnings). Cargo can only override a dependency with a source folder, so `Cargo.toml` has `[patch.crates-io] miniquad = { path = "patched/miniquad" }` and **`bash tools/patch-miniquad.sh` builds that folder** (gitignored) from the published crate plus the patch. Run it once after cloning, or `cargo build` fails with "patch location ... does not contain packages matching `miniquad`"; it is a no-op when the folder is current. It checks the crate against the checksum crates.io publishes, which is pinned in the script because Cargo.lock has none for a patched crate. See `patches/README.md`. Drop all of it once upstream fixes it. Never run `cargo fmt` or clippy over `patched/`.

## Architecture

- `src/lib.rs` (crate `game`, no rendering, fully testable): `session` (the game logic: what a click does, the verb, the held item, the sentence line), `room` (walk outline, obstacles, props, hotspots, Lua room loading, polygon offset), `pathfinder` (visibility graph + Dijkstra), `actor` (path following, facing, distance walked, speech), `script` (Lua coroutines), `world` (inventory and hidden things, shared with scripts), `items` (item names and colours), `ui` (verbs, the verb bar and inventory strip layout, hit testing, the sentence), `direction`, `assets` (rust-embed).
- `src/main.rs`: macroquad app, a thin shell around `Session` (input, drawing, window). `src/render.rs`: `CharacterSprite` and `RoomRenderer` (back-to-front depth sort, skips hidden props). `src/automation.rs`: scripted play (below).
- `Room` stores the precomputed corner-to-corner visibility; `pathfinder::find` only borrows the room. Only start and goal are tested per click. Targets that are not walkable move to the nearest boundary point.
- Scripts: `walkTo(x, y)`, `say(text)`, `wait(seconds)` are small Lua wrappers (in `script.rs`) that call a Rust function, which only records a `Command`, and then `coroutine.yield()`. `ScriptRunner::update` applies the command to the actor and resumes the script once the action has finished. A suspended coroutine is simply dropped, so there are no threads to clean up. To add a blocking call: a Rust function that records a `Command`, a wrapper in `PRELUDE`, and a match arm in `update`. Calls that do not wait (`give`, `take`, `has`, `hide`) are plain Rust functions acting on the shared `World`.
- Rendering: the room draws into a 320x180 render target, which is scaled with nearest filtering and letterboxed. Speech text, the verb labels and the F1 overlay are drawn afterwards at window resolution so they stay crisp. Screenshots render the same frame at 1280x720 into an offscreen target.

## Conventions and data formats

- Coordinates are in the 320x180 virtual space, **y up**, origin bottom-left. An actor's position is its feet. Only `render.rs` and `main.rs` flip to the y-down screen, using `VIRTUAL_HEIGHT - y`.
- Depth: larger y is further back and drawn first. A prop's `depth` defaults to its `y`; ties draw the actor last.
- Room file (`assets/rooms/*.lua`) returns `{ background, margin, walkArea = {{x,y},...}, obstacles = {{{x,y},...},...}, props = {{id?, image, x, y, depth?},...}, hotspots = {{id, name, rect = {x,y,w,h}, walkTo = {x,y}?},...} }`. Obstacles are walking footprints only; their visuals are props. Image paths are relative to `assets/`. A hotspot's `rect` is in game coordinates; `walkTo` is where the player stands to use it with a verb other than "Walk to" (default: where you clicked). Later hotspots are on top. `hide("key")` in a script hides the prop and the hotspot that both have the id `key`.
- Items are in `assets/items.lua`: `{ brass_key = { name = "brass key", color = { r, g, b } }, ... }`. There is no icon art yet; the inventory draws each item as a square of its colour.
- Behaviour lives in the room's script (`assets/scripts/hall.lua`, loaded with `ScriptRunner::load_definitions`, which runs it immediately so it may only define things): `hotspot.<id> = { look = function() say("...") end, open = ..., pickup = ..., use = ..., talk = ..., use_with = { <item id> = function() ... end } }` and the same for `item.<id>` (for carried items). The verb keys are `ui::Verb::key`. Using a verb runs `__interaction` (in `script.rs`): walk to the spot, then call the handler. With no handler "look" says "I see nothing special.", any other verb "I can't do that.", and a combination nobody handles "That doesn't work.". Script API: `give(item)`, `take(item)` (returns whether it was carried), `has(item)`, `hide(id)`. The verb bar sits over the wall (game y >= `ui::PANEL_BOTTOM`) and the inventory strip along the bottom (y < `ui::INVENTORY_TOP`); clicks on the bar never walk.
- Character sheet: 16x32 frames, rows front / back / side (facing right), columns idle then 6 walk frames. Left is the side view mirrored. Walk frames advance every 5 units walked.
- Geometry limits: obstacle rings, once grown by the margin, must not overlap each other or the shrunk outline's edge; very narrow or sharp features can make the offset polygons fold over themselves. The margin is per room, not per actor.
- Comments are kept sparse: only where the code is not obvious from names.

## Verifying changes

- `cargo test` covers pathfinding, actor, scripting, interactions and inventory, room/hotspot/item loading, the verb bar and slots, the click flows in `tests/session.rs` (headless, against the real hall data), and the automation parser (86 tests). Drawing has no unit test.
- **Use scripted play, never OS-level input or screen capture.** `target/debug/game.exe --auto play/hall.txt --out screenshots` (also `play/hotspots.txt` and `play/inventory.txt`) feeds clicks, hovers and keys straight into the game, uses a fixed time step, prints `[state]` lines for `where` and renders `shot` images offscreen. Clicks that start something are ignored while a script runs, so leave enough `wait` for walks and speech to finish before the next click (about 1.7 s per 100 units walked, 1 s + 0.05 s per character spoken). It is deterministic (debug and release give byte-identical PNGs) and works whatever has focus. Commands are listed at the top of `src/automation.rs`. Look at the PNGs with the Read tool. Re-run all three play scripts after changing click behaviour; their comments say what each `where` should print.
- Earlier in this port a PowerShell harness (`SetCursorPos`, `mouse_event`, `keybd_event`, `CopyFromScreen`) sent F1 and clicks to the user's Brave browser because the game window was not in front, and captured part of their screen. Do not use that approach again.

## Art

All art is placeholder, generated by throwaway scripts (not in the repo): `assets/characters/player.png`, `assets/rooms/hall.png`, `assets/props/counter.png`, `assets/props/window.png`, `assets/props/key.png`. Replace the files in place, keeping the layouts above.

## Not done yet

- Dialogue UI, save/load, audio (macroquad's `audio` feature is off), item icon art.
- Hotspots are rectangles only. The inventory holds 11 items (`ui::SLOT_COUNT`) with no scrolling.
- Scripts can only drive the player; no other actors, and no switching rooms. The demo puzzle (key on the floor, drawer in the counter) lives in `assets/scripts/hall.lua`.
- Four facings only, no diagonal views.

## Git

Commit only when asked. The repo-local git identity is already set.
