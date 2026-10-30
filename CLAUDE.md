# Game

2D point-and-click adventure engine and game in the style of Thimbleweed Park (own engine, story, art and puzzles). Rust (edition 2024), macroquad for window/input/drawing, mlua (vendored Lua 5.4) for scripting. Crate name `game`.

Everything is on `master` (remote `origin`, git@github.com:sam-truscott/game.git). The engine was ported from Java/LibGDX/LuaJ (Maven, Eclipse); the last Java commit is `aca1a71`, so `git show aca1a71:src/main/java/com/wumpus/game/<File>.java` shows any original file.

## Environment (Windows 11 on ARM64)

- Rust is installed in `C:\Users\Sam\.cargo\bin`, default host `aarch64-pc-windows-msvc`, so builds and the game are **native ARM64**. This needs the "MSVC Build Tools for ARM64/ARM64EC" component of the Visual Studio Build Tools; without it linking fails with a confusing `link: extra operand` error (cargo falls back to Git Bash's own `link` command).
- Shell state does not persist between tool calls and `cargo` is not on `PATH` in Git Bash, so start commands with `export PATH="$PATH:/c/Users/Sam/.cargo/bin"`. The PowerShell tool may be unavailable; Git Bash can still run `powershell.exe` (fine for generating art with System.Drawing), but see "Verifying" before using it to drive the game.
- Mlua compiles Lua from C source, which needs the MSVC `cl.exe`; that already works here.
- **Python is not installed.** `python` is the Windows Store stub and hangs waiting on stdin; use Git Bash tools (`sed`, `diff`, `cmp`, `git`) or `powershell.exe` for scripting.
- Windows will not let cargo replace a running `game.exe`: close the game window first (`taskkill //F //IM game.exe`; it is this project's own process). Never kill processes by image name for anything else (the permission classifier refuses it, rightly); to stop a background command you started, use the TaskStop tool.
- To launch the game for the user without blocking the shell: `(cmd.exe //c start "" "C:\dev\Game\target\debug\game.exe" >/dev/null 2>&1)`, then check `tasklist //FI "IMAGENAME eq game.exe" //V`. If you need to close it from a script, post `WM_CLOSE` only to its own window after checking the window class is `MINIQUADAPP` and it belongs to the process you started; never send keystrokes.

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

- Controls: there are no verbs. A **single click** on something (a hotspot, or a carried item in the panel) looks at it; a **double click** (second click within 0.4 s and 4 units of the first) interacts with it, cutting short the look the first click started. A click where nothing is walks there. A double click on a carried item picks it up to combine: its slot is highlighted and the next click uses it on a hotspot or another item (clicking it again interacts with it alone; clicking empty room puts it back). Clicks that would start something are ignored while a script runs (walking to something, talking), except the second click of a double click; a held item's use is never cut short. The panel along the bottom holds only the inventory slots for now (26 high). There is no hover text. The system cursor is hidden; `assets/cursor.png` (top-left pixel is the click point) is drawn by `render::CursorSprite` at the session's cursor position, last so it is on top. While an item is held (`session.holding`) `draw_held_item` in `main.rs` draws a half-size icon of it at the cursor's bottom right. **R** restarts, **F1** toggles an overlay of the authored (green) and margin-adjusted (yellow) walk areas.
- Debug builds read `assets/` from the path baked in at compile time, release builds embed it, so a release exe is self-contained.

## The miniquad patch (do not remove casually)

macroquad sits on miniquad 0.4.11 (the newest release), which only stores its window state pointer on x86/x86_64. On aarch64 that call is compiled out, so **no keyboard or mouse input reaches the game and closing the window never ends the process**.

The fix is a small patch, not a copy of the crate: `patches/miniquad-0.4.11-aarch64.patch` changes four `cfg` guards to `target_pointer_width` (plus an `#![allow]` for two upstream warnings). Cargo can only override a dependency with a source folder, so `Cargo.toml` has `[patch.crates-io] miniquad = { path = "patched/miniquad" }` and **`bash tools/patch-miniquad.sh` builds that folder** (gitignored) from the published crate plus the patch. Run it once after cloning, or `cargo build` fails with "patch location ... does not contain packages matching `miniquad`"; it is a no-op when the folder is current. It checks the crate against the checksum crates.io publishes, which is pinned in the script because Cargo.lock has none for a patched crate. See `patches/README.md`. Drop all of it once upstream fixes it. Never run `cargo fmt` or clippy over `patched/`.

## Architecture

- `src/lib.rs` (crate `game`, no rendering, fully testable): `session` (the game logic: what a click and a double click do, the held item), `room` (walk outline, obstacles, props, hotspots, Lua room loading, polygon offset), `pathfinder` (visibility graph + Dijkstra), `actor` (path following, facing, distance walked, speech), `script` (Lua coroutines), `world` (inventory and hidden things, shared with scripts), `items` (item names and colours), `ui` (verbs, the bottom panel's layout and hit testing, screen/room point conversion), `direction`, `assets` (rust-embed).
- `src/main.rs`: macroquad app, a thin shell around `Session` (input, drawing, window). `src/render.rs`: `CharacterSprite` and `RoomRenderer` (back-to-front depth sort, skips hidden props). `src/automation.rs`: scripted play (below).
- `Room` stores the precomputed corner-to-corner visibility; `pathfinder::find` only borrows the room. Only start and goal are tested per click. Targets that are not walkable move to the nearest boundary point.
- Scripts: `walkTo(x, y)`, `say(text)`, `wait(seconds)` are small Lua wrappers (in `script.rs`) that call a Rust function, which only records a `Command`, and then `coroutine.yield()`. `ScriptRunner::update` applies the command to the actor and resumes the script once the action has finished. A suspended coroutine is simply dropped, so there are no threads to clean up. To add a blocking call: a Rust function that records a `Command`, a wrapper in `PRELUDE`, and a match arm in `update`. Calls that do not wait (`give`, `take`, `has`, `hide`) are plain Rust functions acting on the shared `World`.
- Rendering: the room draws into a 320x180 render target, shifted up by `ui::PANEL_HEIGHT` (a camera offset) so its bottom edge sits on the panel and the top 26 pixels of wall are cropped; the panel is then drawn over the bottom of the same target. The target is scaled with nearest filtering and letterboxed. Speech text, the F1 overlay and the mouse cursor are drawn afterwards at window resolution so they stay crisp (room points go through `View::room_to_window`, panel points through `View::screen_to_window`). Screenshots render the same frame at 1280x720 into an offscreen target.

## Conventions and data formats

- Coordinates are in the 320x180 virtual space, **y up**, origin bottom-left. An actor's position is its feet. Only `render.rs` and `main.rs` flip to the y-down screen, using `VIRTUAL_HEIGHT - y`.
- There are two spaces: **room** points (everything in room files, scripts, actors, hotspots) and **screen** points (mouse clicks and the cursor, the panel). The panel fills the bottom `ui::PANEL_HEIGHT` (26) of the screen and the room is drawn above it, so a room point is a screen point minus 26 in y (`ui::to_room`, `ui::to_screen`). `Session::click` and `move_cursor` take screen points; `Session::hotspot_at` and the actor take room points. The visible room is y 0 to 154; nothing should be placed above that.
- Depth: larger y is further back and drawn first. A prop's `depth` defaults to its `y`; ties draw the actor last.
- Room file (`assets/rooms/*.lua`) returns `{ background, margin, walkArea = {{x,y},...}, obstacles = {{{x,y},...},...}, props = {{id?, image, x, y, depth?},...}, hotspots = {{id, name, rect = {x,y,w,h}, walkTo = {x,y}?},...} }`. Obstacles are walking footprints only; their visuals are props. Image paths are relative to `assets/`. A hotspot's `rect` is in room coordinates; `walkTo` is where the player stands to use it with a verb other than "Walk to" (default: where you clicked). Later hotspots are on top. `hide("key")` in a script hides the prop and the hotspot that both have the id `key`. **Animated props**: give a prop `frames = N` (and optionally `fps`, default 8): its image is then a horizontal strip of N equal frames, and the prop shows the frame `World` holds for its id (0 at first). Scripts call `animate(id, first, last [, fps])` (blocking; 0-based, `first > last` plays backwards, then it stays on `last`) or `setFrame(id, n)`. `ScriptRunner::update` advances the animations. The door in the hall (`props/door.png`, 4 frames) is the demo; `play/closet.txt` plays it.
- Items are in `assets/items.lua`: `{ brass_key = { name = "brass key", color = { r, g, b } }, ... }`. There is no icon art yet; the inventory draws each item as a square of its colour.
- Behaviour lives in the room's script (`assets/scripts/hall.lua`, loaded with `ScriptRunner::load_definitions`, which runs it immediately so it may only define things): `hotspot.<id> = { look = function() say("...") end, interact = ..., use_with = { <item id> = function() ... end } }` and the same for `item.<id>` (for carried items). The two actions are `look` (single click) and `interact` (double click). Doing one runs `__interaction` (in `script.rs`): walk to the spot, then call the handler. With no handler "look" says "I see nothing special.", `interact` "I can't do that.", and a combination nobody handles "That doesn't work.". Script API: `give(item)`, `take(item)` (returns whether it was carried), `has(item)`, `hide(id)`. Clicks on the panel (verb buttons, inventory slots, the gaps) never walk the player.
- Character sheet: 16x32 frames, rows front / back / side (facing right), columns idle then 6 walk frames. Left is the side view mirrored. Walk frames advance every 5 units walked.
- Geometry limits: obstacle rings, once grown by the margin, must not overlap each other or the shrunk outline's edge; very narrow or sharp features can make the offset polygons fold over themselves. The margin is per room, not per actor.
- Comments are kept sparse: only where the code is not obvious from names.

## Verifying changes

- `cargo test` covers pathfinding, actor, scripting, interactions and inventory, room/hotspot/item loading, the panel slots, the click flows in `tests/session.rs` (headless, against the real hall data), and the automation parser. Drawing has no unit test.
- **Use scripted play, never OS-level input or screen capture.** `target/debug/game.exe --auto play/hall.txt --out screenshots` (also `play/hotspots.txt`, `play/inventory.txt` and `play/closet.txt`) feeds clicks, hovers and keys straight into the game (positions are screen points: add 26 to a room y, slots are at y 13; two `click` lines in a row are a double click), uses a fixed time step, prints `[state]` lines for `where` and renders `shot` images offscreen. Single clicks are ignored while a script runs, so leave enough `wait` for walks and speech to finish before the next click (about 1.7 s per 100 units walked, 1 s + 0.05 s per character spoken). It is deterministic (debug and release give byte-identical PNGs) and works whatever has focus. Commands are listed at the top of `src/automation.rs`. Look at the PNGs with the Read tool. Re-run all the play scripts after changing click behaviour; their comments say what each `where` should print.
- Earlier in this port a PowerShell harness (`SetCursorPos`, `mouse_event`, `keybd_event`, `CopyFromScreen`) sent F1 and clicks to the user's Brave browser because the game window was not in front, and captured part of their screen. Do not use that approach again.

## Art

All art is placeholder, generated by throwaway scripts (not in the repo): `assets/characters/player.png`, `assets/rooms/hall.png`, `assets/props/bar.png`, `assets/props/table.png`, `assets/props/key.png`, `assets/props/cupboard.png`, `assets/props/door.png`, `assets/props/switch.png`, `assets/rooms/closet.png`, `assets/cursor.png`. Replace the files in place, keeping the layouts above.

## Not done yet

- Dialogue UI, save/load, audio (macroquad's `audio` feature is off), item icon art.
- Hotspots are rectangles only. The inventory holds 11 items (`ui::SLOT_COUNT`) with no scrolling.
- Scripts can only drive the player; no other actors. The demo puzzle (key on the floor, corner cupboard) lives in `assets/scripts/hall.lua`. The hall is a pub: the bar fills the right half and is only reachable from the strip in front of it, because the walk area is notched rather than the bar being an obstacle (a margin-grown obstacle may not touch the outline edge); table and chairs, a window onto the street, a corner cupboard and an animated door are on the left.
- Four facings only, no diagonal views.

## Git

- Commit only when asked, and **push only when asked**: the user has said "commit, don't push" before, so a branch can legitimately be ahead of `origin`. Check `git status -sb` before assuming.
- The repo-local git identity is already set. Commit messages end with the `Co-Authored-By` trailer from the session's attribution reminder.
- Push with `GIT_SSH_COMMAND="ssh -o BatchMode=yes -o ConnectTimeout=15" git push` (fails fast instead of prompting). GitHub's ed25519 host key is already in `~/.ssh/known_hosts`. The old vendored `vendor/miniquad` is still in history (commit `6b5f8b3`); do not rewrite or force-push history unless asked.

## Working with the user

- They test in the real window and report what they see; offer to launch it (see Environment) before committing UI changes, and say plainly what you could not verify (the real keyboard keys R and F1 have never been confirmed by them; everything else was seen working).
- They want a small repo: they objected to a copied dependency and asked for a patch instead. Ask before adding large third-party code, and before downloads, installs or changes outside the repo (known_hosts, PATH, the Visual Studio installer).
- They give short instructions ("Run it", "Commit and push", "Go"); "continue" has meant the next planned step. Say how you interpreted anything ambiguous (for "remove the hover text" I removed the sentence line and said so).
- Ideas not yet asked for: a click that skips the current speech line; hotspot names as a tooltip (removed on purpose to save space); a dialogue UI; other actors and room switching; real art and item icons.

## Rooms

- A room id such as `closet` is `assets/rooms/closet.lua` plus its handlers in `assets/scripts/closet.lua`; the ids are listed in `session::ROOMS` and `hall` starts. Every room's script is loaded at start, each into its own `hotspot` table (`ScriptRunner::set_room`), so hotspot ids only need to be unique within a room. **Prop ids are global** (the animation frame, hidden state and lights live in `World` by id), so give props distinct ids across rooms (`door`, `closet_door`).
- `enterRoom("closet", x, y)` in a script switches room, putting the player at the nearest walkable point to x, y; the script carries on in the new room. `Session` loads the room, sets `room_id` and `take_room_changed()` tells `main.rs` to rebuild the `RoomRenderer`. `[state]` lines print the room.
- A room file may say `dark = true`: it is covered in darkness (`render::DARKNESS`) until `setLight("closet", true)`; `isLit(id)` asks. Props with `glow = true` are drawn over the darkness (the light switch outline). `Session::is_dark` combines the two. The closet is only 80x90 on a black background image, `margin = 2`.

## Safety and tests

- Lua runs sandboxed (`script::sandboxed_lua`): only table, string, math, coroutine and utf8; `load`, `loadfile`, `dofile` and `require` are removed, and rooms/items use a data-only state (`data_lua`). Scripts have no instruction or memory limit yet. Numbers passed to `walkTo`, `wait`, `animate` and `enterRoom` must be finite; room outlines need 3+ finite points; `assets::bytes` refuses paths that leave `assets/`; `shot` names in play scripts are letters, digits, `_` and `-`.
- Integration tests share `tests/common/mod.rs`. `tests/session.rs` finds hotspots through the room files (`point`, `stand`) instead of hard-coded coordinates, so layout changes should not break it; slot positions and the `FLOOR` point are still literal. `assets.rs` checks every id in `session::ROOMS` and that prop ids are unique across rooms. The play scripts are only run by hand (the automation lives in the binary). No coverage tool is installed (`cargo-llvm-cov` would need `llvm-tools-preview`).
