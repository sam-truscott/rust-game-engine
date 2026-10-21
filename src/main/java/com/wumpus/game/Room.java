package com.wumpus.game;

import java.util.ArrayList;
import java.util.List;

import org.luaj.vm2.Globals;
import org.luaj.vm2.LuaValue;
import org.luaj.vm2.lib.jse.JsePlatform;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.math.Intersector;
import com.badlogic.gdx.math.Vector2;

/**
 * A room: a walkable outline (possibly concave) with obstacle holes, plus the images drawn for it.
 * Walking is done on the outline shrunk, and the obstacles grown, by a margin, so a character
 * standing on the walkable boundary does not overlap walls or obstacles.
 */
public class Room {
    private static final float BOUNDARY_EPS = 0.01f;
    private static final float MIN_MITER_DENOMINATOR = 0.1f;

    public record Segment(Vector2 a, Vector2 b) {
    }

    /** An image placed in the room; characters with a smaller y than the depth are drawn in front of it. */
    public record Prop(String image, float x, float y, float depth) {
    }

    private final float[] outline;
    private final List<float[]> obstacles;
    private final float[] walkOutline;
    private final List<float[]> walkObstacles = new ArrayList<>();
    private final String background;
    private final List<Prop> props;
    private final List<Vector2> vertices = new ArrayList<>();
    private final List<Segment> edges = new ArrayList<>();
    private final PathFinder pathFinder;

    public Room(float[] outline, List<float[]> obstacles) {
        this(outline, obstacles, 0, null, List.of());
    }

    public Room(float[] outline, List<float[]> obstacles, float margin, String background, List<Prop> props) {
        this.outline = outline;
        this.obstacles = obstacles;
        this.background = background;
        this.props = props;
        walkOutline = offset(outline, -margin);
        obstacles.forEach(obstacle -> walkObstacles.add(offset(obstacle, margin)));
        addBoundary(walkOutline);
        walkObstacles.forEach(this::addBoundary);
        pathFinder = new PathFinder(this);
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

    /** Moves every edge of the ring outwards by the distance (inwards if negative), mitring the corners. */
    static float[] offset(float[] ring, float distance) {
        if (distance == 0) {
            return ring;
        }
        int count = ring.length / 2;
        float doubleArea = 0;
        for (int i = 0; i < count; i++) {
            int next = (i + 1) % count;
            doubleArea += ring[2 * i] * ring[2 * next + 1] - ring[2 * next] * ring[2 * i + 1];
        }
        float outward = doubleArea > 0 ? 1 : -1;

        float[] result = new float[ring.length];
        for (int i = 0; i < count; i++) {
            Vector2 point = new Vector2(ring[2 * i], ring[2 * i + 1]);
            Vector2 previous = new Vector2(ring[2 * ((i + count - 1) % count)], ring[2 * ((i + count - 1) % count) + 1]);
            Vector2 next = new Vector2(ring[2 * ((i + 1) % count)], ring[2 * ((i + 1) % count) + 1]);
            Vector2 in = point.cpy().sub(previous).nor();
            Vector2 out = next.cpy().sub(point).nor();
            Vector2 normalIn = new Vector2(in.y, -in.x).scl(outward);
            Vector2 normalOut = new Vector2(out.y, -out.x).scl(outward);
            float denominator = Math.max(1 + normalIn.dot(normalOut), MIN_MITER_DENOMINATOR);
            Vector2 moved = point.add(normalIn.add(normalOut).scl(distance / denominator));
            result[2 * i] = moved.x;
            result[2 * i + 1] = moved.y;
        }
        return result;
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

        List<Prop> props = new ArrayList<>();
        LuaValue propTables = table.get("props");
        for (int i = 1; i <= propTables.length(); i++) {
            LuaValue prop = propTables.get(i);
            float y = (float) prop.get("y").checkdouble();
            props.add(new Prop(
                    prop.get("image").checkjstring(),
                    (float) prop.get("x").checkdouble(),
                    y,
                    (float) prop.get("depth").optdouble(y)));
        }

        return new Room(
                readPoints(table.get("walkArea")),
                obstacles,
                (float) table.get("margin").optdouble(0),
                table.get("background").optjstring(null),
                props);
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

    public String getBackground() {
        return background;
    }

    public List<Prop> getProps() {
        return props;
    }

    public float[] getOutline() {
        return outline;
    }

    public List<float[]> getObstacles() {
        return obstacles;
    }

    public float[] getWalkOutline() {
        return walkOutline;
    }

    public List<float[]> getWalkObstacles() {
        return walkObstacles;
    }

    /** Corners of the walkable boundary, which paths bend around. */
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
        if (!Intersector.isPointInPolygon(walkOutline, 0, walkOutline.length, point.x, point.y)) {
            return false;
        }
        for (float[] obstacle : walkObstacles) {
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

    /** Returns the point itself if it is walkable, otherwise the nearest point on the walkable boundary. */
    public Vector2 constrain(float x, float y) {
        Vector2 point = new Vector2(x, y);
        return isWalkable(point) ? point : nearestBoundaryPoint(point);
    }

    /** Waypoints for walking from a point to the nearest walkable point to (x, y); empty if unreachable. */
    public List<Vector2> findPath(Vector2 from, float x, float y) {
        List<Vector2> path = pathFinder.find(from, constrain(x, y));
        return path == null ? List.of() : path;
    }
}
