hotspot.switch = {
  look = function()
    if isLit("closet") then
      say("A light switch.")
    else
      say("The outline of a light switch.")
    end
  end,
  interact = function()
    if isLit("closet") then
      setLight("closet", false)
      setFrame("closet_switch", 0)
    else
      setLight("closet", true)
      setFrame("closet_switch", 1)
    end
    say("Click.")
  end,
}

hotspot.shelf = {
  look = function()
    if isLit("closet") then
      say("Shelves of clean glasses for the pub.")
    else
      say("I can't see a thing in here.")
    end
  end,
  interact = function()
    if isLit("closet") then
      say("I'd better leave them for the customers.")
    else
      say("I can't see a thing in here.")
    end
  end,
}

hotspot.door = {
  look = function() say("The door back to the bar.") end,
  interact = function()
    animate("closet_door", 0, 3)
    setFrame("closet_door", 0)
    setFrame("door", 0)
    enterRoom("hall", 136, 90)
  end,
}

hotspot.key = {
  look = function()
    if isLit("closet") then
      say("A small brass key, left on the shelf.")
    else
      say("I can't see a thing in here.")
    end
  end,
  interact = function()
    if isLit("closet") then
      give("brass_key")
      hide("key")
      say("Got it.")
    else
      say("I can't see a thing in here.")
    end
  end,
}
