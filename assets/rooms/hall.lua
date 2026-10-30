return {
  background = "rooms/hall.png",
  margin = 8,
  walkArea = {
    { 20, 20 },
    { 300, 20 },
    { 314, 100 },
    { 6, 100 },
  },
  obstacles = {
    { { 120, 45 }, { 200, 45 }, { 200, 70 }, { 120, 70 } },
    { { 232, 55 }, { 280, 55 }, { 280, 59 }, { 232, 59 } },
  },
  props = {
    { image = "props/counter.png", x = 120, y = 45 },
    { image = "props/window.png", x = 232, y = 55 },
    { id = "key", image = "props/key.png", x = 40, y = 36 },
  },
  hotspots = {
    { id = "counter", name = "counter", rect = { 120, 45, 80, 55 }, walkTo = { 160, 30 } },
    { id = "window", name = "window", rect = { 232, 55, 48, 48 }, walkTo = { 256, 40 } },
    { id = "key", name = "brass key", rect = { 36, 33, 20, 12 }, walkTo = { 46, 30 } },
  },
}
