hotspot.bar = {
  look = function() say("A long oak bar, ring-stained and well loved.") end,
  interact = function() say("Nobody is serving.") end,
}

hotspot.bottles = {
  look = function() say("Plenty of bottles. All of them behind the bar.") end,
  interact = function() say("Out of reach.") end,
}

hotspot.table = {
  look = function() say("A wobbly table with a chair either side.") end,
  interact = function() say("I'm not thirsty yet.") end,
}

hotspot.window = {
  look = function()
    say("A road, and a hedge on the other side.")
    wait(0.3)
    say("Nobody about.")
  end,
  interact = function() say("It doesn't open.") end,
}

hotspot.key = {
  look = function() say("A small brass key.") end,
  interact = function()
    give("brass_key")
    hide("key")
    say("Got it.")
  end,
}

hotspot.cupboard = {
  look = function() say("A little cupboard in the corner.") end,
  interact = function() say("It's locked.") end,
  use_with = {
    brass_key = function()
      say("The key fits the cupboard!")
      take("brass_key")
      give("coin")
      say("Inside is a silver coin.")
    end,
  },
}

hotspot.door = {
  look = function() say("The door to the cupboard.") end,
  interact = function()
    animate("door", 0, 3)
    enterRoom("closet", 182, 28)
  end,
}

item.brass_key = {
  look = function() say("It must open something.") end,
}

item.coin = {
  look = function() say("A silver coin. Heads, probably.") end,
}
