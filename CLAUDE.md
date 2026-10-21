# Game

2D point-and-click adventure engine and game in the style of Thimbleweed Park (own engine, story, art and puzzles). Java 21, LibGDX 1.14.2, LuaJ 3.0.1 for scripting, Maven, Eclipse. Package `com.wumpus.game`.

## Environment (Windows 11 on ARM64)

- LibGDX ships no Windows ARM64 natives (`gdxarm64.dll` is missing), so the game **must run on an x64 JDK** under emulation: `C:\dev\x64_jdk-25.0.4.1+1`. The ARM64 JDKs (`C:\dev\jdk-21.0.8+9`, `C:\dev\jdk-25.0.4.1+1`) build fine but fail at launch. In Eclipse, select the x64 JDK under Installed JREs for the project, or Run As fails with the same error.
- Maven is `C:\dev\apache-maven-3.10.0`. Shell state does not persist between tool calls, so reload `PATH` from the machine and user environment in the same command (`$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')`).
- `maven.compiler.release` is 21; the installed JDKs are newer, which is fine.

## Build and run

```
mvn package        # compiles, runs the unit tests, builds the shaded jar
C:\dev\x64_jdk-25.0.4.1+1\bin\java.exe -jar target\game-0.1.0-SNAPSHOT.jar
```

- Packaging uses `maven-shade-plugin` (chosen over the Spring Boot repackager). The long list of "overlapping resource" warnings (LWJGL `.sha1` files) is harmless; leave the files in, LWJGL uses them when extracting natives.
- The manifest sets `Enable-Native-Access` to silence the Java 25 warning. Two LWJGL 3.3.3 warnings remain (`sun.misc.Unsafe`, "Unsupported JNI version"); they come from LibGDX's bundled LWJGL on JDK 25 and are harmless.
- Controls: click to walk (ignored while a script runs), **R** restarts, **F1** toggles an overlay of the authored (green) and margin-adjusted (yellow) walk areas.

## Architecture

- `Main` / `GameApp`: window, 320x180 virtual resolution (`FitViewport`), input, main loop. `Main` calls `System.exit(0)` after the app ends as a safety net, since LuaJ's coroutine threads are non-daemon.
- `Room`: loads `rooms/*.lua`; holds the walk outline, obstacles, props and background. Walking geometry is the outline shrunk and the obstacles grown by `margin`; the authored polygons are kept for drawing and the F1 overlay.
- `PathFinder`: visibility graph over the room's corners plus Dijkstra. Corner-to-corner visibility is precomputed per room; only start and goal are tested per click. Targets that are not walkable are moved to the nearest boundary point.
- `Actor`: position (the feet), path following, facing, distance walked, speech. `CharacterSprite` draws it.
- `RoomRenderer`: draws the background, then props and characters sorted back to front.
- `ScriptRunner`: runs Lua scripts as LuaJ coroutines. `walkTo(x, y)`, `say(text)` and `wait(seconds)` start an `Action` and yield; the script resumes when the action finishes. Add a new blocking call with `register(name, args -> Action)`.

## Conventions and data formats

- Coordinates are in the 320x180 virtual space, **y up**, origin bottom-left. An actor's position is its feet; sprites are drawn with their feet there.
- Depth: larger y is further back and drawn first. A prop's `depth` defaults to its `y`, so characters with a smaller y are drawn in front of it.
- Room file (`rooms/*.lua`) returns `{ background, margin, walkArea = {{x,y},...}, obstacles = {{{x,y},...},...}, props = {{image, x, y, depth?},...} }`. Obstacles are walking footprints only; their visuals are props. Image paths are classpath-relative.
- Character sheet: 16x32 frames, rows front / back / side (facing right), columns idle then 6 walk frames. Left is the side view mirrored. Walk frames advance every 5 units walked, so the feet do not slide.
- Constraints on room geometry: obstacle rings, once grown by the margin, must not overlap each other or the shrunk outline's edge; very narrow or sharp features can make the offset polygons fold over themselves. The margin is per room, not per actor.
- Comments are kept sparse: only where the code is not obvious from names.

## Testing

- JUnit 5 tests run in `mvn package` and need no GL: `PathFinderTest`, `ActorTest`. `Room` can be built from arrays without LibGDX; `Room.load` needs `Gdx.files`. Rendering has no automated test.
- To check behaviour in the running game, drive it with scripted clicks and screenshots (PowerShell with `SetProcessDPIAware`, `SetCursorPos`, `mouse_event`, `CopyFromScreen`). Leave a gap of about 100-200 ms between moving the cursor and pressing, or the game reads the old cursor position and drops the click. Close the window with `PostMessage(WM_CLOSE)` to the process's `MainWindowHandle`.
- The scripts used for this and for generating art lived in the session scratch folder and are not in the repo.

## Art

All art is placeholder, generated with `System.Drawing` scripts (not in the repo): `characters/player.png`, `rooms/hall.png`, `props/counter.png`, `props/window.png`. Replace the files in place, keeping the layouts above.

## Not done yet

- Hotspots and verbs (the planned next step), inventory, dialogue UI, save/load, audio.
- Scripts can only drive the player; no other actors, and no switching rooms.
- Four facings only, no diagonal views.

## Git

Commit only when asked. The repo-local git identity is already set.
