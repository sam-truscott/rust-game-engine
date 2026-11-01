return {
  background = "rooms/a.png",
  walkArea = { { 0, 0 }, { 200, 0 }, { 200, 100 }, { 0, 100 } },
  hotspots = {
    { id = "gem", name = "gem", rect = { 10, 10, 10, 10 }, walkTo = { 30, 20 } },
    { id = "door", name = "door", door = true, rect = { 150, 10, 20, 40 }, walkTo = { 140, 20 } },
  },
}
