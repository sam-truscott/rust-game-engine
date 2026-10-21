package com.wumpus.game;

import java.util.ArrayList;
import java.util.List;

import org.luaj.vm2.Globals;
import org.luaj.vm2.LuaValue;
import org.luaj.vm2.lib.jse.JsePlatform;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.glutils.ShapeRenderer;
import com.badlogic.gdx.math.EarClippingTriangulator;
import com.badlogic.gdx.math.Intersector;
import com.badlogic.gdx.math.Vector2;

/** A room's walkable area: an outline (possibly concave) with obstacle holes cut out of it. */
public class Room {
    private static final float BOUNDARY_EPS = 0.01f;

    public record Segment(Vector2 a, Vector2 b) {
    }

    private final float[] outline;
    private final List<float[]> obstacles;
    private final List<Vector2> vertices = new ArrayList<>();
    private final List<Segment> edges = new ArrayList<>();

    public Room(float[] outline, List<float[]> obstacles) {
        this.outline = outline;
        this.obstacles = obstacles;
        addBoundary(outline);
        obstacles.forEach(this::addBoundary);
    }

    private void addBoundary(float[] ring) {
        int count = ring.length / 2;
        int first = vertices.size();
        for (int i = 0; i < count; i++) {
            vertices.add(new Vector2(ring[2 * i], ring[2 * i + 1]));
        }
        for (int i = 0; i < count; i++) {
            edges.add(new Segment(vertices.get(first + i), vertices.get(first + (i + 1) % count)));
        }
    }

    public static Room load(String classpathPath) {
        Globals globals = JsePlatform.standardGlobals();
        String source = Gdx.files.classpath(classpathPath).readString("UTF-8");
        LuaValue table = globals.load(source, classpathPath).call();

        List<float[]> obstacles = new ArrayList<>();
        LuaValue obstacleTables = table.get("obstacles");
        for (int i = 1; i <= obstacleTables.length(); i++) {
            obstacles.add(readPoints(obstacleTables.get(i)));
        }
        return new Room(readPoints(table.get("walkArea")), obstacles);
    }

    private static float[] readPoints(LuaValue points) {
        float[] flat = new float[points.length() * 2];
        for (int i = 0; i < points.length(); i++) {
            LuaValue point = points.get(i + 1);
            flat[2 * i] = (float) point.get(1).checkdouble();
            flat[2 * i + 1] = (float) point.get(2).checkdouble();
        }
        return flat;
    }

    public List<Vector2> getVertices() {
        return vertices;
    }

    public List<Segment> getEdges() {
        return edges;
    }

    /** The boundary counts as walkable, so paths can run along walls and around obstacle corners. */
    public boolean isWalkable(Vector2 point) {
        if (distanceToBoundary(point) < BOUNDARY_EPS) {
            return true;
        }
        if (!Intersector.isPointInPolygon(outline, 0, outline.length, point.x, point.y)) {
            return false;
        }
        for (float[] obstacle : obstacles) {
            if (Intersector.isPointInPolygon(obstacle, 0, obstacle.length, point.x, point.y)) {
                return false;
            }
        }
        return true;
    }

    private float distanceToBoundary(Vector2 point) {
        return point.dst(nearestBoundaryPoint(point));
    }

    private Vector2 nearestBoundaryPoint(Vector2 point) {
        Vector2 nearest = new Vector2();
        Vector2 best = new Vector2();
        float bestDistance = Float.MAX_VALUE;
        for (Segment edge : edges) {
            Intersector.nearestSegmentPoint(edge.a(), edge.b(), point, nearest);
            float distance = nearest.dst2(point);
            if (distance < bestDistance) {
                bestDistance = distance;
                best.set(nearest);
            }
        }
        return best;
    }

    /** Returns the point itself if it is walkable, otherwise the nearest point on the boundary. */
    public Vector2 constrain(float x, float y) {
        Vector2 point = new Vector2(x, y);
        return isWalkable(point) ? point : nearestBoundaryPoint(point);
    }

    /** Waypoints for walking from a point to the nearest walkable point to (x, y); empty if unreachable. */
    public List<Vector2> findPath(Vector2 from, float x, float y) {
        List<Vector2> path = PathFinder.find(this, from, constrain(x, y));
        return path == null ? List.of() : path;
    }

    /** Fills the walkable outline using the shape renderer's current colour. */
    public void drawFloor(ShapeRenderer shapes) {
        fill(shapes, outline);
    }

    /** Fills the obstacles using the shape renderer's current colour. */
    public void drawObstacles(ShapeRenderer shapes) {
        obstacles.forEach(obstacle -> fill(shapes, obstacle));
    }

    private static void fill(ShapeRenderer shapes, float[] ring) {
        short[] t = new EarClippingTriangulator().computeTriangles(ring).toArray();
        for (int i = 0; i < t.length; i += 3) {
            shapes.triangle(
                    ring[2 * t[i]], ring[2 * t[i] + 1],
                    ring[2 * t[i + 1]], ring[2 * t[i + 1] + 1],
                    ring[2 * t[i + 2]], ring[2 * t[i + 2] + 1]);
        }
    }
}
