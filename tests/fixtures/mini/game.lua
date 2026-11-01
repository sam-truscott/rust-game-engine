-- A two-room game for the engine tests: nothing here is the pub.
return {
  id = "Mini",
  title = "Mini",
  start = { room = "a", x = 20, y = 20 },
  rooms = { "a", "b" },
  intro = "scripts/intro.lua",
  player = "characters/p.png",
  cursor = "cursor.png",
  defaults = { look = "Nothing.", interact = "No.", use = "Nope." },
  tuning = { walk_speed = 120 },
  theme = { darkness = 0.5 },
}
