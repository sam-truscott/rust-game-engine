package com.wumpus.game;

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration;

public final class Main {
    private Main() {
    }

    public static void main(String[] args) {
        Lwjgl3ApplicationConfiguration config = new Lwjgl3ApplicationConfiguration();
        config.setTitle("Game");
        config.setWindowedMode(1280, 720);
        config.useVsync(true);
        new Lwjgl3Application(new GameApp(), config);
        // LuaJ coroutine threads are non-daemon and would keep the JVM alive
        System.exit(0);
    }
}
