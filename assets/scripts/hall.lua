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
    play("door", "open")
    enterRoom("closet", 180, 28)
  end,
}

item.brass_key = {
  look = function() say("It must open something.") end,
}

item.coin = {
  look = function() say("A silver coin. Heads, probably.") end,
}

hotspot.barman = {
  look = function() say("The barman polishes a glass. Probably the same one as an hour ago.") end,
  interact = function()
    if flag("served") then
      say("One's plenty for now.")
    else
      say("A beer, please.")
      wait(0.4)
      say("He eyes me and keeps polishing. I think he wants paying.")
    end
  end,
  use_with = {
    coin = function()
      take("coin")
      say("One beer, please.")
      play("barman", "take_coin")
      play("barman", "pour")
      play("barman", "serve")
      give("beer")
      resume("barman")
      setFlag("served")
      say("Cheers.")
    end,
    empty_glass = function()
      take("empty_glass")
      say("Your glass.")
      wait(0.4)
      say("Barman: Thanks. I've been dying for one of those.")
    end,
  },
}

hotspot.drinker = {
  look = function() say("He's propping up the end of the bar, and it's propping him up.") end,
  interact = function() say("He's not in a talking mood.") end,
}

hotspot.patron_left = {
  look = function() say("He's nursing a pint like it owes him money.") end,
  interact = function() say("He doesn't look up.") end,
}

hotspot.patron_right = {
  look = function() say("A man in a hat, smoking and keeping his own counsel.") end,
  interact = function() say("He blows smoke at the ceiling.") end,
}

item.beer = {
  look = function() say("A cold pint. Better than the key, anyway.") end,
  interact = function()
    say("Down the hatch.")
    take("beer")
    give("empty_glass")
    say("Ahh. That was almost worth the coin.")
    corruptionEvent()
    say("What the hell was that?")
  end,
}

item.empty_glass = {
  look = function() say("Empty. The story of my life.") end,
}
