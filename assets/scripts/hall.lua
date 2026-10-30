hotspot.counter = {
  look = function() say("A sturdy counter. Someone polishes it.") end,
  open = function()
    say("The drawers are locked.")
  end,
  pickup = function() say("It weighs a ton.") end,
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
  open = function() say("It's only a frame.") end,
}

hotspot.key = {
  look = function() say("A small brass key.") end,
  pickup = function()
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
