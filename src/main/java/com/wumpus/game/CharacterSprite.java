package com.wumpus.game;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.g2d.TextureRegion;
import com.badlogic.gdx.utils.Disposable;

/**
 * Draws an actor from a sheet with one row per view (front, back, right-facing side) and one column
 * per frame (idle, then the walk cycle). The left view is the side view mirrored. The walk frame
 * follows the distance walked, so the feet do not slide.
 */
public class CharacterSprite implements Disposable {
    public static final int FRAME_WIDTH = 16;
    public static final int FRAME_HEIGHT = 32;

    private static final int IDLE_COLUMN = 0;
    private static final int WALK_FRAMES = 6;
    private static final float DISTANCE_PER_FRAME = 5f;

    private final Texture texture;
    private final TextureRegion[][] frames;

    public CharacterSprite(String classpathPath) {
        texture = new Texture(Gdx.files.classpath(classpathPath));
        texture.setFilter(Texture.TextureFilter.Nearest, Texture.TextureFilter.Nearest);
        frames = TextureRegion.split(texture, FRAME_WIDTH, FRAME_HEIGHT);
    }

    /** Draws the actor standing with its feet at its position, centred horizontally. */
    public void draw(Batch batch, Actor actor) {
        Direction facing = actor.getFacing();
        int row = switch (facing) {
            case FRONT -> 0;
            case BACK -> 1;
            case LEFT, RIGHT -> 2;
        };
        int column = actor.isWalking()
                ? IDLE_COLUMN + 1 + (int) (actor.getWalkDistance() / DISTANCE_PER_FRAME) % WALK_FRAMES
                : IDLE_COLUMN;

        float x = actor.getPosition().x - FRAME_WIDTH / 2f;
        float y = actor.getPosition().y;
        if (facing == Direction.LEFT) {
            batch.draw(frames[row][column], x + FRAME_WIDTH, y, -FRAME_WIDTH, FRAME_HEIGHT);
        } else {
            batch.draw(frames[row][column], x, y, FRAME_WIDTH, FRAME_HEIGHT);
        }
    }

    @Override
    public void dispose() {
        texture.dispose();
    }
}
