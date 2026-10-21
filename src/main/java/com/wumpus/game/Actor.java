package com.wumpus.game;

import java.util.ArrayDeque;
import java.util.Deque;

import com.badlogic.gdx.math.Vector2;

public class Actor {
    private static final float WALK_SPEED = 60f;
    private static final float SPEECH_BASE_SECONDS = 1f;
    private static final float SPEECH_SECONDS_PER_CHAR = 0.05f;

    private final Room room;
    private final Vector2 position = new Vector2();
    private final Deque<Vector2> path = new ArrayDeque<>();
    private String speech;
    private float speechRemaining;

    public Actor(Room room, float x, float y) {
        this.room = room;
        position.set(x, y);
    }

    public Vector2 getPosition() {
        return position;
    }

    public String getSpeech() {
        return speech;
    }

    public void walkTo(float x, float y) {
        path.clear();
        path.addAll(room.findPath(position, x, y));
    }

    public boolean isWalking() {
        return !path.isEmpty();
    }

    public void say(String text) {
        speech = text;
        speechRemaining = SPEECH_BASE_SECONDS + SPEECH_SECONDS_PER_CHAR * text.length();
    }

    public boolean isSpeaking() {
        return speech != null;
    }

    public void update(float deltaSeconds) {
        float step = WALK_SPEED * deltaSeconds;
        while (step > 0 && !path.isEmpty()) {
            Vector2 next = path.peekFirst();
            float distance = position.dst(next);
            if (distance <= step) {
                position.set(next);
                path.removeFirst();
                step -= distance;
            } else {
                position.add(next.cpy().sub(position).nor().scl(step));
                step = 0;
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
