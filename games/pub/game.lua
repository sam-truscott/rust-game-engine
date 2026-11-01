-- The game's manifest: everything the engine needs to know before it opens a room.
return {
  -- Names the save folder (%APPDATA%\<id>\saves).
  id = "Game",
  title = "Game",

  start = { room = "hall", x = 30, y = 50 },
  -- Every room is rooms/<id>.lua with its handlers in scripts/<id>.lua.
  rooms = { "hall", "closet" },
  intro = "scripts/intro.lua",
  items = "items.lua",

  -- 24x48 unit frames: rows front / back / side, columns idle then 6 walk frames.
  player = "characters/player.png",
  cursor = "cursor.png",

  -- What the player says when a script has nothing to say about a click.
  defaults = {
    look = "I see nothing special.",
    interact = "I can't do that.",
    use = "That doesn't work.",
  },

  -- Optional; these are the engine's own values.
  tuning = {
    walk_speed = 60,
    speech_seconds = 1.0,
    speech_seconds_per_char = 0.05,
    double_click_seconds = 0.4,
    double_click_distance = 4,
  },
  theme = {
    background = { 0.1, 0.1, 0.2 },
    panel = { 0.06, 0.06, 0.1 },
    darkness = 0.99,
  },
}
