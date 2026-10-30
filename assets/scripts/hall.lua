hotspot.counter = {
  look = function() say("A sturdy counter. Someone polishes it.") end,
  interact = function() say("The drawers are locked.") end,
  use_with = {
    brass_key = function()
      say("The key fits the drawer!")
      take("brass_key")
      give("coin")
      say("Inside is a silver coin.")
    end,
  },
}

hotspot.window = {
  look = function()
    say("I can see right through it.")
    wait(0.3)
    say("Literally.")
  end,
  interact = function() say("It's only a frame.") end,
}

hotspot.key = {
  look = function() say("A small brass key.") end,
  interact = function()
    give("brass_key")
    hide("key")
    say("Got it.")
  end,
}

item.brass_key = {
  look = function() say("It must open something.") end,
}

item.coin = {
  look = function() say("A silver coin. Heads, probably.") end,
}

local cupboard_open = false

-- There is no "Close", so interacting with an open cupboard closes it.
hotspot.cupboard = {
  look = function() say("A little wooden cupboard.") end,
  interact = function()
    if cupboard_open then
      animate("cupboard", 3, 0)
      cupboard_open = false
    else
      animate("cupboard", 0, 3)
      cupboard_open = true
      say("Nothing inside but dust.")
    end
  end,
}
