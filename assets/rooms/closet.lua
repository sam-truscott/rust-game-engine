-- The pub's cupboard: a tiny room, a quarter of the screen wide and half as high, drawn in the middle
-- of an otherwise black background. It is dark until the light switch is used.
return {
  background = "rooms/closet.png",
  margin = 2,
  dark = true,
  walkArea = {
    { 126, 14 },
    { 194, 14 },
    { 194, 36 },
    { 126, 36 },
  },
  props = {
    { id = "closet_switch", image = "props/switch.png", x = 156, y = 62, frames = 2, glow = true, scale = 4 },
    { id = "key", image = "props/key.png", x = 146, y = 63, scale = 4 },
    { id = "closet_door", image = "props/door.png", x = 164, y = 36, frames = 4, fps = 10, scale = 4, clips = { open = { 0, 3 }, close = { 3, 0 } } },
  },
  hotspots = {
    { id = "shelf", name = "shelf", rect = { 124, 58, 32, 40 }, walkTo = { 146, 24 } },
    { id = "key", name = "brass key", rect = { 145, 62, 12, 8 }, walkTo = { 150, 24 } },
    { id = "switch", name = "light switch", rect = { 155, 61, 10, 14 }, walkTo = { 160, 26 } },
    { id = "door", name = "door", door = true, rect = { 164, 36, 32, 54 }, walkTo = { 180, 28 } },
  },
}
