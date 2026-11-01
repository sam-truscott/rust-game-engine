hotspot.gem = {
  look = function() say("A gem.") end,
  interact = function() give("gem") hide("gem") end,
}
hotspot.door = {
  interact = function() enterRoom("b", 50, 20) end,
}
