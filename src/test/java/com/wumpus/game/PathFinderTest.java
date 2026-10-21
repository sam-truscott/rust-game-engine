package com.wumpus.game;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
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
    void marginKeepsPathsClearOfObstacles() {
        Room room = new Room(FLOOR, List.of(BLOCK), 5, null, List.of());
        Vector2 start = new Vector2(20, 50);
        List<Vector2> path = room.findPath(start, 180, 50);

        Vector2 previous = start;
        for (Vector2 point : path) {
            for (float t = 0; t <= 1; t += 0.01f) {
                Vector2 sample = previous.cpy().lerp(point, t);
                boolean insideMargin = sample.x > 75.01f && sample.x < 124.99f
                        && sample.y > 25.01f && sample.y < 74.99f;
                assertFalse(insideMargin, "too close to the block at " + sample);
            }
            previous = point;
        }
        assertEquals(180f, previous.x, 1e-3f);
    }

    @Test
    void marginKeepsTargetsClearOfWalls() {
        Room room = new Room(FLOOR, List.of(), 5, null, List.of());

        List<Vector2> east = room.findPath(new Vector2(50, 50), 300, 50);
        assertEquals(195f, east.get(east.size() - 1).x, 1e-3f);

        List<Vector2> corner = room.findPath(new Vector2(50, 50), -20, -20);
        assertEquals(5f, corner.get(corner.size() - 1).x, 1e-3f);
        assertEquals(5f, corner.get(corner.size() - 1).y, 1e-3f);
    }

    @Test
    void offsetMovesEdgesOutwardOrInwardWhateverTheWinding() {
        float[] counterClockwise = { 10, 10, 20, 10, 20, 20, 10, 20 };
        float[] clockwise = { 10, 10, 10, 20, 20, 20, 20, 10 };

        assertArrayEquals(new float[] { 8, 8, 22, 8, 22, 22, 8, 22 }, Room.offset(counterClockwise, 2), 1e-4f);
        assertArrayEquals(new float[] { 8, 8, 8, 22, 22, 22, 22, 8 }, Room.offset(clockwise, 2), 1e-4f);
        assertArrayEquals(new float[] { 12, 12, 18, 12, 18, 18, 12, 18 }, Room.offset(counterClockwise, -2), 1e-4f);
    }

    @Test
    void repeatedQueriesGiveTheSamePath() {
        Room room = new Room(FLOOR, List.of(BLOCK));
        Vector2 start = new Vector2(20, 50);
        assertEquals(room.findPath(start, 180, 50), room.findPath(start, 180, 50));
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
