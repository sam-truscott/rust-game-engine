package com.wumpus.game;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.function.Consumer;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.utils.Disposable;

/** Draws a room's background and props, interleaved with other things by depth. */
public class RoomRenderer implements Disposable {
    /** Something drawn in the room; those with a larger depth are further back and drawn first. */
    public record Renderable(float depth, Consumer<Batch> draw) {
    }

    private final Texture background;
    private final List<Texture> textures = new ArrayList<>();
    private final List<Renderable> props = new ArrayList<>();

    public RoomRenderer(Room room) {
        background = room.getBackground() == null ? null : load(room.getBackground());
        for (Room.Prop prop : room.getProps()) {
            Texture texture = load(prop.image());
            props.add(new Renderable(prop.depth(), batch -> batch.draw(texture, prop.x(), prop.y())));
        }
    }

    private Texture load(String classpathPath) {
        Texture texture = new Texture(Gdx.files.classpath(classpathPath));
        texture.setFilter(Texture.TextureFilter.Nearest, Texture.TextureFilter.Nearest);
        textures.add(texture);
        return texture;
    }

    public void draw(Batch batch, List<Renderable> others) {
        if (background != null) {
            batch.draw(background, 0, 0);
        }
        List<Renderable> sorted = new ArrayList<>(props);
        sorted.addAll(others);
        sorted.sort(Comparator.comparingDouble(Renderable::depth).reversed());
        sorted.forEach(renderable -> renderable.draw().accept(batch));
    }

    @Override
    public void dispose() {
        textures.forEach(Texture::dispose);
    }
}
