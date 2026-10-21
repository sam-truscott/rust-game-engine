package com.wumpus.game;

import com.badlogic.gdx.math.Vector2;

public class Actor {
    private static final float WALK_SPEED = 60f;
    private static final float SPEECH_BASE_SECONDS = 1f;
    private static final float SPEECH_SECONDS_PER_CHAR = 0.05f;

    private final Vector2 position = new Vector2();
    private Vector2 walkTarget;
    private String speech;
    private float speechRemaining;

    public Actor(float x, float y) {
        position.set(x, y);
    }

    public Vector2 getPosition() {
        return position;
    }

    public String getSpeech() {
        return speech;
    }

    public void walkTo(float x, float y) {
        walkTarget = new Vector2(x, y);
    }

    public boolean isWalking() {
        return walkTarget != null;
    }

    public void say(String text) {
        speech = text;
        speechRemaining = SPEECH_BASE_SECONDS + SPEECH_SECONDS_PER_CHAR * text.length();
    }

    public boolean isSpeaking() {
        return speech != null;
    }

    public void update(float deltaSeconds) {
        if (walkTarget != null) {
            float step = WALK_SPEED * deltaSeconds;
            if (position.dst(walkTarget) <= step) {
                position.set(walkTarget);
                walkTarget = null;
            } else {
                position.add(walkTarget.cpy().sub(position).nor().scl(step));
            }
        }
        if (speech != null) {
            speechRemaining -= deltaSeconds;
            if (speechRemaining <= 0) {
                speech = null;
            }
        }
    }
}
