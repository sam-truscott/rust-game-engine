package com.wumpus.game;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;

import org.junit.jupiter.api.Test;

import com.badlogic.gdx.math.Vector2;

class PathFinderTest {
    private static final float[] FLOOR = { 0, 0, 200, 0, 200, 100, 0, 100 };
    private static final float[] BLOCK = { 80, 30, 120, 30, 120, 70, 80, 70 };
    private static final float[] L_SHAPE = { 0, 0, 100, 0, 100, 40, 40, 40, 40, 100, 0, 100 };

    private static float length(Vector2 start, List<Vector2> path) {
        float total = 0;
        Vector2 previous = start;
        for (Vector2 point : path) {
            total += previous.dst(point);
            previous = point;
        }
        return total;
    }

    private static void assertWalkable(Room room, Vector2 start, List<Vector2> path) {
        Vector2 previous = start;
        for (Vector2 point : path) {
            assertTrue(PathFinder.isVisible(room, previous, point), previous + " -> " + point);
            previous = point;
        }
    }

    @Test
    void clearLineGoesStraightToTheGoal() {
        Room room = new Room(FLOOR, List.of());
        Vector2 start = new Vector2(10, 50);
        List<Vector2> path = room.findPath(start, 150, 20);
        assertEquals(1, path.size());
        assertEquals(new Vector2(150, 20), path.get(0));
    }

    @Test
    void goesAroundAnObstacle() {
        Room room = new Room(FLOOR, List.of(BLOCK));
        Vector2 start = new Vector2(20, 50);
        List<Vector2> path = room.findPath(start, 180, 50);

        assertTrue(path.size() > 1);
        assertEquals(new Vector2(180, 50), path.get(path.size() - 1));
        assertWalkable(room, start, path);
        assertTrue(length(start, path) > 160);
        assertTrue(length(start, path) < 200);
    }

    @Test
    void obstacleCornersOnTheLineAreNotCutThrough() {
        // The straight line from corner to opposite corner passes through the block's interior.
        Room room = new Room(FLOOR, List.of(BLOCK));
        Vector2 start = new Vector2(80, 30);
        List<Vector2> path = room.findPath(start, 120, 70);
        assertWalkable(room, start, path);
        assertTrue(length(start, path) > start.dst(120, 70) + 1);
    }

    @Test
    void routesThroughTheInnerCornerOfAConcaveRoom() {
        Room room = new Room(L_SHAPE, List.of());
        Vector2 start = new Vector2(90, 10);
        List<Vector2> path = room.findPath(start, 10, 90);

        assertEquals(2, path.size());
        assertEquals(new Vector2(40, 40), path.get(0));
        assertEquals(new Vector2(10, 90), path.get(1));
        assertWalkable(room, start, path);
    }

    @Test
    void targetInsideAnObstacleIsMovedToItsEdge() {
        Room room = new Room(FLOOR, List.of(BLOCK));
        Vector2 start = new Vector2(20, 50);
        List<Vector2> path = room.findPath(start, 100, 50);

        Vector2 end = path.get(path.size() - 1);
        assertTrue(room.isWalkable(end));
        assertFalse(end.x > 80.01f && end.x < 119.99f && end.y > 30.01f && end.y < 69.99f);
        assertWalkable(room, start, path);
    }

    @Test
    void targetOutsideTheRoomIsMovedToItsEdge() {
        Room room = new Room(FLOOR, List.of());
        List<Vector2> path = room.findPath(new Vector2(10, 10), 300, 50);
        Vector2 end = path.get(path.size() - 1);
        assertEquals(200f, end.x, 1e-3f);
        assertEquals(50f, end.y, 1e-3f);
    }
}
