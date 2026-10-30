hotspot.counter = {
  look = function() say("A sturdy counter. Someone polishes it.") end,
  open = function()
    say("The drawers are locked.")
  end,
  pickup = function() say("It weighs a ton.") end,
}

hotspot.window = {
  look = function()
    say("I can see right through it.")
    wait(0.3)
    say("Literally.")
  end,
  open = function() say("It's only a frame.") end,
}
