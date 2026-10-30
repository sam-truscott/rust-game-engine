-- A pub. The bar runs along the right half of the room; the walk area is notched so the strip in front
-- of it is the only part of the right half you can reach. Tables and chairs are on the left, and the
-- window shows the street outside.
return {
  background = "rooms/hall.png",
  margin = 8,
  walkArea = {
    { 20, 20 },
    { 314, 20 },
    { 314, 52 },
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
    { id = "door", image = "props/door.png", x = 120, y = 96, frames = 4, fps = 10, scale = 4 },
  },
  hotspots = {
    { id = "table", name = "table", rect = { 52, 40, 66, 34 }, walkTo = { 85, 30 } },
    { id = "cupboard", name = "cupboard", rect = { 30, 80, 30, 50 }, walkTo = { 45, 70 } },
    { id = "window", name = "window", rect = { 64, 118, 46, 34 }, walkTo = { 88, 80 } },
    { id = "door", name = "door", rect = { 120, 96, 32, 54 }, walkTo = { 136, 90 } },
    { id = "bottles", name = "bottles", rect = { 166, 100, 148, 50 }, walkTo = { 240, 40 } },
    { id = "bar", name = "bar", rect = { 160, 52, 160, 36 }, walkTo = { 240, 40 } },
  },
}
