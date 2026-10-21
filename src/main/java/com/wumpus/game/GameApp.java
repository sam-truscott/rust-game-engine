package com.wumpus.game;

import com.badlogic.gdx.ApplicationAdapter;
import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.Input;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.OrthographicCamera;
import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.graphics.g2d.GlyphLayout;
import com.badlogic.gdx.graphics.g2d.SpriteBatch;
import com.badlogic.gdx.graphics.glutils.ShapeRenderer;
import com.badlogic.gdx.math.MathUtils;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.utils.Align;
import com.badlogic.gdx.utils.ScreenUtils;
import com.badlogic.gdx.utils.viewport.FitViewport;
import com.badlogic.gdx.utils.viewport.Viewport;

public class GameApp extends ApplicationAdapter {
    private static final int VIRTUAL_WIDTH = 320;
    private static final int VIRTUAL_HEIGHT = 180;
    private static final int PLAYER_WIDTH = 16;
    private static final int PLAYER_HEIGHT = 32;
    private static final float SPEECH_WRAP_WIDTH = 160;
    private static final String INTRO_SCRIPT = "scripts/intro.lua";
    private static final String START_ROOM = "rooms/hall.lua";

    private OrthographicCamera camera;
    private Viewport viewport;
    private SpriteBatch batch;
    private ShapeRenderer shapes;
    private Room room;
    private Texture playerTexture;
    private BitmapFont font;
    private final GlyphLayout layout = new GlyphLayout();
    private Actor player;
    private ScriptRunner scripts;

    @Override
    public void create() {
        camera = new OrthographicCamera();
        viewport = new FitViewport(VIRTUAL_WIDTH, VIRTUAL_HEIGHT, camera);
        batch = new SpriteBatch();
        shapes = new ShapeRenderer();
        font = new BitmapFont();
        font.getData().setScale(0.6f);

        Pixmap pixmap = new Pixmap(PLAYER_WIDTH, PLAYER_HEIGHT, Pixmap.Format.RGBA8888);
        pixmap.setColor(Color.ORANGE);
        pixmap.fill();
        playerTexture = new Texture(pixmap);
        playerTexture.setFilter(Texture.TextureFilter.Nearest, Texture.TextureFilter.Nearest);
        pixmap.dispose();

        restart();
    }

    private void restart() {
        room = Room.load(START_ROOM);
        player = new Actor(room, 60, 50);
        scripts = new ScriptRunner(player);
        scripts.start(INTRO_SCRIPT, Gdx.files.classpath(INTRO_SCRIPT).readString("UTF-8"));
    }

    @Override
    public void resize(int width, int height) {
        viewport.update(width, height, true);
    }

    @Override
    public void render() {
        if (Gdx.input.isKeyJustPressed(Input.Keys.R)) {
            restart();
        }

        if (Gdx.input.justTouched() && !scripts.isRunning()) {
            Vector2 target = viewport.unproject(new Vector2(Gdx.input.getX(), Gdx.input.getY()));
            player.walkTo(target.x, target.y);
        }

        float delta = Gdx.graphics.getDeltaTime();
        player.update(delta);
        scripts.update(delta);

        ScreenUtils.clear(0.1f, 0.1f, 0.2f, 1f);
        viewport.apply();
        shapes.setProjectionMatrix(camera.combined);
        shapes.begin(ShapeRenderer.ShapeType.Filled);
        shapes.setColor(0.25f, 0.3f, 0.4f, 1f);
        room.drawFloor(shapes);
        shapes.setColor(0.15f, 0.18f, 0.25f, 1f);
        room.drawObstacles(shapes);
        shapes.end();
        batch.setProjectionMatrix(camera.combined);
        batch.begin();
        batch.draw(playerTexture, player.getPosition().x - PLAYER_WIDTH / 2f, player.getPosition().y);
        drawSpeech();
        batch.end();
    }

    private void drawSpeech() {
        String speech = player.getSpeech();
        if (speech == null) {
            return;
        }
        layout.setText(font, speech, Color.WHITE, SPEECH_WRAP_WIDTH, Align.center, true);
        float x = MathUtils.clamp(player.getPosition().x - SPEECH_WRAP_WIDTH / 2f, 0, VIRTUAL_WIDTH - SPEECH_WRAP_WIDTH);
        float y = Math.min(player.getPosition().y + PLAYER_HEIGHT + 4 + layout.height, VIRTUAL_HEIGHT);
        font.draw(batch, layout, x, y);
    }

    @Override
    public void dispose() {
        batch.dispose();
        shapes.dispose();
        font.dispose();
        playerTexture.dispose();
    }
}
