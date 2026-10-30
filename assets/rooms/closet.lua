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
    { id = "closet_switch", image = "props/switch.png", x = 159, y = 54, frames = 2, glow = true, scale = 4 },
    { id = "closet_door", image = "props/door.png", x = 168, y = 36, frames = 4, fps = 10, scale = 4 },
  },
  hotspots = {
    { id = "shelf", name = "shelf", rect = { 124, 58, 32, 40 }, walkTo = { 146, 24 } },
    { id = "switch", name = "light switch", rect = { 158, 53, 10, 14 }, walkTo = { 162, 26 } },
    { id = "door", name = "door", rect = { 168, 36, 28, 36 }, walkTo = { 182, 28 } },
  },
}
