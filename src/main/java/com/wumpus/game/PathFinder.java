package com.wumpus.game;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedList;
import java.util.List;

import com.badlogic.gdx.math.Vector2;

/** Shortest paths through a room's walkable area, found over the visibility graph of its corners. */
final class PathFinder {
    private static final float EPS = 1e-3f;
    private static final int START = 0;
    private static final int GOAL = 1;

    private PathFinder() {
    }

    /** Returns the waypoints from start (exclusive) to goal (inclusive), or null if the goal is unreachable. */
    static List<Vector2> find(Room room, Vector2 start, Vector2 goal) {
        List<Vector2> nodes = new ArrayList<>();
        nodes.add(start);
        nodes.add(goal);
        nodes.addAll(room.getVertices());

        int count = nodes.size();
        float[] distance = new float[count];
        Arrays.fill(distance, Float.MAX_VALUE);
        distance[START] = 0;
        int[] previous = new int[count];
        Arrays.fill(previous, -1);
        boolean[] done = new boolean[count];

        while (true) {
            int u = -1;
            for (int i = 0; i < count; i++) {
                if (!done[i] && distance[i] < Float.MAX_VALUE && (u == -1 || distance[i] < distance[u])) {
                    u = i;
                }
            }
            if (u == -1) {
                return null;
            }
            if (u == GOAL) {
                break;
            }
            done[u] = true;
            for (int v = 0; v < count; v++) {
                if (done[v]) {
                    continue;
                }
                float candidate = distance[u] + nodes.get(u).dst(nodes.get(v));
                if (candidate < distance[v] && isVisible(room, nodes.get(u), nodes.get(v))) {
                    distance[v] = candidate;
                    previous[v] = u;
                }
            }
        }

        LinkedList<Vector2> path = new LinkedList<>();
        for (int i = GOAL; i != START; i = previous[i]) {
            path.addFirst(nodes.get(i).cpy());
        }
        return path;
    }

    /** True if the straight segment from a to b stays within the walkable area. */
    static boolean isVisible(Room room, Vector2 a, Vector2 b) {
        if (a.dst2(b) < EPS * EPS) {
            return true;
        }
        for (Room.Segment edge : room.getEdges()) {
            if (crossesProperly(a, b, edge.a(), edge.b())) {
                return false;
            }
        }

        // Corners lying on the segment split it, since each piece may be on a different side of the boundary.
        Vector2 direction = b.cpy().sub(a);
        float length2 = direction.len2();
        List<Float> splits = new ArrayList<>();
        splits.add(0f);
        splits.add(1f);
        for (Vector2 vertex : room.getVertices()) {
            float t = vertex.cpy().sub(a).dot(direction) / length2;
            if (t > EPS && t < 1 - EPS && a.cpy().mulAdd(direction, t).dst2(vertex) < EPS * EPS) {
                splits.add(t);
            }
        }
        splits.sort(null);
        for (int i = 0; i + 1 < splits.size(); i++) {
            float middle = (splits.get(i) + splits.get(i + 1)) / 2;
            if (!room.isWalkable(a.cpy().mulAdd(direction, middle))) {
                return false;
            }
        }
        return true;
    }

    private static boolean crossesProperly(Vector2 a, Vector2 b, Vector2 c, Vector2 d) {
        float d1 = orientation(a, b, c);
        float d2 = orientation(a, b, d);
        float d3 = orientation(c, d, a);
        float d4 = orientation(c, d, b);
        return ((d1 > EPS && d2 < -EPS) || (d1 < -EPS && d2 > EPS))
                && ((d3 > EPS && d4 < -EPS) || (d3 < -EPS && d4 > EPS));
    }

    private static float orientation(Vector2 a, Vector2 b, Vector2 c) {
        return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    }
}
