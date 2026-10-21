package com.wumpus.game;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;

import org.junit.jupiter.api.Test;

class ActorTest {
    private static final float[] FLOOR = { 0, 0, 200, 0, 200, 100, 0, 100 };

    private static Actor actorAt(float x, float y) {
        return new Actor(new Room(FLOOR, List.of()), x, y);
    }

    @Test
    void startsFacingTheViewerAndStanding() {
        Actor actor = actorAt(50, 50);
        assertEquals(Direction.FRONT, actor.getFacing());
        assertFalse(actor.isWalking());
        assertEquals(0f, actor.getWalkDistance());
    }

    @Test
    void facesTheDirectionOfTravel() {
        Actor actor = actorAt(100, 50);

        actor.walkTo(150, 50);
        actor.update(0.1f);
        assertEquals(Direction.RIGHT, actor.getFacing());

        actor.walkTo(20, 50);
        actor.update(0.1f);
        assertEquals(Direction.LEFT, actor.getFacing());

        actor.walkTo(actor.getPosition().x, 95);
        actor.update(0.1f);
        assertEquals(Direction.BACK, actor.getFacing());

        actor.walkTo(actor.getPosition().x, 5);
        actor.update(0.1f);
        assertEquals(Direction.FRONT, actor.getFacing());
    }

    @Test
    void mostlyVerticalDiagonalsFaceFrontOrBack() {
        Actor actor = actorAt(100, 50);
        actor.walkTo(110, 10);
        actor.update(0.1f);
        assertEquals(Direction.FRONT, actor.getFacing());
    }

    @Test
    void keepsFacingAfterStopping() {
        Actor actor = actorAt(50, 50);
        actor.walkTo(60, 50);
        actor.update(5f);
        assertFalse(actor.isWalking());
        assertEquals(Direction.RIGHT, actor.getFacing());
    }

    @Test
    void walkDistanceMatchesDistanceTravelled() {
        Actor actor = actorAt(50, 50);
        actor.walkTo(110, 50);
        actor.update(0.5f);
        assertEquals(30f, actor.getWalkDistance(), 1e-3f);
        actor.update(5f);
        assertTrue(!actor.isWalking());
        assertEquals(60f, actor.getWalkDistance(), 1e-3f);
    }

    @Test
    void walkDistanceRestartsWithEachWalkButNotWhenRedirectedMidWalk() {
        Actor actor = actorAt(50, 50);
        actor.walkTo(110, 50);
        actor.update(5f);
        assertEquals(60f, actor.getWalkDistance(), 1e-3f);

        actor.walkTo(60, 50);
        assertEquals(0f, actor.getWalkDistance(), 1e-3f);
        actor.update(0.25f);
        assertEquals(15f, actor.getWalkDistance(), 1e-3f);

        actor.walkTo(110, 50);
        assertEquals(15f, actor.getWalkDistance(), 1e-3f);
    }
}
