package com.wumpus.game;

import org.luaj.vm2.Globals;
import org.luaj.vm2.LuaValue;
import org.luaj.vm2.lib.jse.JsePlatform;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.glutils.ShapeRenderer;
import com.badlogic.gdx.math.EarClippingTriangulator;
import com.badlogic.gdx.math.Intersector;
import com.badlogic.gdx.math.Vector2;

/** A room's walkable area. The area must be convex for straight-line walking to stay inside it. */
public class Room {
    private final float[] walkArea;
    private final short[] triangles;

    public Room(float[] walkArea) {
        this.walkArea = walkArea;
        this.triangles = new EarClippingTriangulator().computeTriangles(walkArea).toArray();
    }

    public static Room load(String classpathPath) {
        Globals globals = JsePlatform.standardGlobals();
        String source = Gdx.files.classpath(classpathPath).readString("UTF-8");
        LuaValue points = globals.load(source, classpathPath).call().get("walkArea");
        float[] area = new float[points.length() * 2];
        for (int i = 0; i < points.length(); i++) {
            LuaValue point = points.get(i + 1);
            area[2 * i] = (float) point.get(1).checkdouble();
            area[2 * i + 1] = (float) point.get(2).checkdouble();
        }
        return new Room(area);
    }

    /** Returns the point itself if it is walkable, otherwise the nearest point on the walk area's edge. */
    public Vector2 constrain(float x, float y) {
        if (Intersector.isPointInPolygon(walkArea, 0, walkArea.length, x, y)) {
            return new Vector2(x, y);
        }
        Vector2 point = new Vector2(x, y);
        Vector2 nearest = new Vector2();
        Vector2 best = new Vector2();
        float bestDistance = Float.MAX_VALUE;
        int count = walkArea.length / 2;
        for (int i = 0; i < count; i++) {
            int next = (i + 1) % count;
            Vector2 start = new Vector2(walkArea[2 * i], walkArea[2 * i + 1]);
            Vector2 end = new Vector2(walkArea[2 * next], walkArea[2 * next + 1]);
            Intersector.nearestSegmentPoint(start, end, point, nearest);
            float distance = nearest.dst2(point);
            if (distance < bestDistance) {
                bestDistance = distance;
                best.set(nearest);
            }
        }
        return best;
    }

    /** Fills the walk area using the shape renderer's current colour. */
    public void drawFloor(ShapeRenderer shapes) {
        for (int i = 0; i < triangles.length; i += 3) {
            shapes.triangle(
                    walkArea[2 * triangles[i]], walkArea[2 * triangles[i] + 1],
                    walkArea[2 * triangles[i + 1]], walkArea[2 * triangles[i + 1] + 1],
                    walkArea[2 * triangles[i + 2]], walkArea[2 * triangles[i + 2] + 1]);
        }
    }
}
