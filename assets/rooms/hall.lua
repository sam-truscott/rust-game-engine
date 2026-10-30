-- A pub. The bar runs along the right half of the room; the walk area is notched so the strip in front
-- of it is the only part of the right half you can reach. Tables and chairs are on the left, and the
-- window shows the street outside.
-- The barman, the man at the bar and the two at the table are animated props that loop (`idle`).
return {
  background = "rooms/hall.png",
  margin = 8,
  -- Cars pass behind the lower window pane; their headlights sweep the room and the bar mirror.
  street = { glass = { 67.5, 122.5, 38.75, 11.25 }, mirror = { 207.5, 123, 65, 26 }, floor = 60 },
  walkArea = {
    { 20, 20 },
    { 276, 20 },
    { 276, 52 },
    { 160, 52 },
    { 160, 100 },
    { 6, 100 },
  },
  obstacles = {
    -- table and its two chairs
    { { 52, 40 }, { 118, 40 }, { 118, 54 }, { 52, 54 } },
    -- corner cupboard
    { { 30, 80 }, { 60, 80 }, { 60, 82 }, { 30, 82 } },
  },
  props = {
    { image = "props/table.png", x = 52, y = 40, scale = 4 },
    { image = "props/bar.png", x = 160, y = 52, scale = 4 },
    { id = "cupboard", image = "props/cupboard.png", x = 30, y = 80, scale = 4 },
    { id = "patron_left", image = "props/patron_left.png", x = 53, y = 41, depth = 39, frames = 14, idle = "loop", fps = 3, scale = 4, clips = { loop = { 0, 13 } } },
    { id = "patron_right", image = "props/patron_right.png", x = 88, y = 41, depth = 39, frames = 13, idle = "loop", fps = 3, scale = 4, clips = { loop = { 0, 12 } } },
    { id = "barman", image = "props/barman.png", x = 240, y = 68, frames = 12, idle = "clean", fps = 4, scale = 4,
      clips = {
        clean = { 0, 5 },
        take_coin = { 6, 7, fps = 3 },   -- takes the coin and pockets it
        pour = { 8, 9, fps = 3 },
        serve = { 10, 11, fps = 3 },     -- slides the pint over
      } },
    { id = "drinker", image = "props/drinker.png", x = 284, y = 47, frames = 15, idle = "loop", fps = 3, scale = 4, clips = { loop = { 0, 14 } } },
    { id = "door", image = "props/door.png", x = 120, y = 96, frames = 4, fps = 10, scale = 4, clips = { open = { 0, 3 }, close = { 3, 0 } } },
  },
  hotspots = {
    { id = "table", name = "table", rect = { 52, 40, 66, 34 }, walkTo = { 85, 30 } },
    { id = "cupboard", name = "cupboard", rect = { 30, 80, 30, 50 }, walkTo = { 45, 70 } },
    { id = "window", name = "window", rect = { 64, 118, 46, 34 }, walkTo = { 88, 80 } },
    { id = "door", name = "door", door = true, rect = { 120, 96, 32, 54 }, walkTo = { 136, 90 } },
    { id = "bottles", name = "bottles", rect = { 166, 100, 148, 50 }, walkTo = { 240, 40 } },
    { id = "bar", name = "bar", rect = { 160, 52, 160, 36 }, walkTo = { 240, 40 } },
    { id = "barman", name = "barman", rect = { 238, 84, 26, 28 }, walkTo = { 224, 44 } },
    { id = "drinker", name = "man at the bar", rect = { 286, 47, 20, 44 }, walkTo = { 268, 40 } },
    { id = "patron_left", name = "man with a pint", rect = { 56, 42, 20, 34 }, walkTo = { 66, 30 } },
    { id = "patron_right", name = "man in a hat", rect = { 98, 42, 18, 34 }, walkTo = { 104, 30 } },
  },
}
