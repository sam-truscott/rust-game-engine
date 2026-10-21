package com.wumpus.game;

import java.util.List;

import com.badlogic.gdx.ApplicationAdapter;
import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.Input;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.OrthographicCamera;
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
    private static final float SPEECH_WRAP_WIDTH = 160;
    private static final String PLAYER_SPRITE = "characters/player.png";
    private static final String INTRO_SCRIPT = "scripts/intro.lua";
    private static final String START_ROOM = "rooms/hall.lua";

    private OrthographicCamera camera;
    private Viewport viewport;
    private SpriteBatch batch;
    private ShapeRenderer shapes;
    private Room room;
    private RoomRenderer roomRenderer;
    private CharacterSprite playerSprite;
    private BitmapFont font;
    private final GlyphLayout layout = new GlyphLayout();
    private Actor player;
    private ScriptRunner scripts;
    private boolean showDebug;

    @Override
    public void create() {
        camera = new OrthographicCamera();
        viewport = new FitViewport(VIRTUAL_WIDTH, VIRTUAL_HEIGHT, camera);
        batch = new SpriteBatch();
        shapes = new ShapeRenderer();
        font = new BitmapFont();
        font.getData().setScale(0.6f);

        playerSprite = new CharacterSprite(PLAYER_SPRITE);

        restart();
    }

    private void restart() {
        if (scripts != null) {
            scripts.clear();
        }
        if (roomRenderer != null) {
            roomRenderer.dispose();
        }
        room = Room.load(START_ROOM);
        roomRenderer = new RoomRenderer(room);
        Vector2 start = room.constrain(60, 50);
        player = new Actor(room, start.x, start.y);
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
        if (Gdx.input.isKeyJustPressed(Input.Keys.F1)) {
            showDebug = !showDebug;
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
        batch.setProjectionMatrix(camera.combined);
        batch.begin();
        roomRenderer.draw(batch, List.of(new RoomRenderer.Renderable(
                player.getPosition().y, b -> playerSprite.draw(b, player))));
        drawSpeech();
        batch.end();

        if (showDebug) {
            drawDebug();
        }
    }

    private void drawSpeech() {
        String speech = player.getSpeech();
        if (speech == null) {
            return;
        }
        layout.setText(font, speech, Color.WHITE, SPEECH_WRAP_WIDTH, Align.center, true);
        float x = MathUtils.clamp(player.getPosition().x - SPEECH_WRAP_WIDTH / 2f, 0, VIRTUAL_WIDTH - SPEECH_WRAP_WIDTH);
        float y = Math.min(player.getPosition().y + CharacterSprite.FRAME_HEIGHT + 4 + layout.height, VIRTUAL_HEIGHT);
        font.draw(batch, layout, x, y);
    }

    /** F1: authored walk area in green, the margin-adjusted area characters actually walk on in yellow. */
    private void drawDebug() {
        shapes.setProjectionMatrix(camera.combined);
        shapes.begin(ShapeRenderer.ShapeType.Line);
        shapes.setColor(Color.GREEN);
        shapes.polygon(room.getOutline());
        room.getObstacles().forEach(shapes::polygon);
        shapes.setColor(Color.YELLOW);
        shapes.polygon(room.getWalkOutline());
        room.getWalkObstacles().forEach(shapes::polygon);
        shapes.end();
    }

    @Override
    public void dispose() {
        scripts.clear();
        roomRenderer.dispose();
        batch.dispose();
        shapes.dispose();
        font.dispose();
        playerSprite.dispose();
    }
}
