package com.wumpus.game;

import java.util.ArrayDeque;
import java.util.Deque;

import com.badlogic.gdx.math.Vector2;

public class Actor {
    private static final float WALK_SPEED = 60f;
    private static final float SPEECH_BASE_SECONDS = 1f;
    private static final float SPEECH_SECONDS_PER_CHAR = 0.05f;

    private static final float MIN_MOVE = 1e-3f;

    private final Room room;
    private final Vector2 position = new Vector2();
    private final Deque<Vector2> path = new ArrayDeque<>();
    private Direction facing = Direction.FRONT;
    private float walkDistance;
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

    public Direction getFacing() {
        return facing;
    }

    /** Distance walked since the current walk began, so animation can follow the feet. */
    public float getWalkDistance() {
        return walkDistance;
    }

    public void walkTo(float x, float y) {
        if (path.isEmpty()) {
            walkDistance = 0;
        }
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

    private void face(float dx, float dy) {
        if (Math.abs(dx) > Math.abs(dy)) {
            facing = dx > 0 ? Direction.RIGHT : Direction.LEFT;
        } else {
            facing = dy > 0 ? Direction.BACK : Direction.FRONT;
        }
    }

    public void update(float deltaSeconds) {
        float step = WALK_SPEED * deltaSeconds;
        while (step > 0 && !path.isEmpty()) {
            Vector2 next = path.peekFirst();
            float distance = position.dst(next);
            if (distance > MIN_MOVE) {
                face(next.x - position.x, next.y - position.y);
            }
            if (distance <= step) {
                position.set(next);
                path.removeFirst();
                walkDistance += distance;
                step -= distance;
            } else {
                position.add(next.cpy().sub(position).nor().scl(step));
                walkDistance += step;
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
