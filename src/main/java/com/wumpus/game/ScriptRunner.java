package com.wumpus.game;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.function.Function;

import org.luaj.vm2.Globals;
import org.luaj.vm2.LuaThread;
import org.luaj.vm2.LuaValue;
import org.luaj.vm2.Varargs;
import org.luaj.vm2.lib.VarArgFunction;
import org.luaj.vm2.lib.jse.JsePlatform;

import com.badlogic.gdx.Gdx;

/**
 * Runs Lua scripts as coroutines. A blocking call such as walkTo() starts an {@link Action}
 * and yields; the script is resumed once that action has finished.
 */
public class ScriptRunner {
    private static final class Script {
        final String name;
        final LuaThread thread;
        Action action;

        Script(String name, LuaThread thread) {
            this.name = name;
            this.thread = thread;
        }
    }

    private static final class Wait implements Action {
        private float remaining;

        Wait(float seconds) {
            remaining = seconds;
        }

        @Override
        public boolean update(float deltaSeconds) {
            remaining -= deltaSeconds;
            return remaining <= 0;
        }
    }

    private final Globals globals = JsePlatform.standardGlobals();
    private final List<Script> scripts = new ArrayList<>();
    private Script current;

    public ScriptRunner(Actor player) {
        register("walkTo", args -> {
            player.walkTo((float) args.checkdouble(1), (float) args.checkdouble(2));
            return dt -> !player.isWalking();
        });
        register("say", args -> {
            player.say(args.checkjstring(1));
            return dt -> !player.isSpeaking();
        });
        register("wait", args -> new Wait((float) args.checkdouble(1)));
    }

    private void register(String name, Function<Varargs, Action> start) {
        globals.set(name, new VarArgFunction() {
            @Override
            public Varargs invoke(Varargs args) {
                current.action = start.apply(args);
                globals.yield(LuaValue.NONE);
                return LuaValue.NONE;
            }
        });
    }

    public void start(String name, String source) {
        LuaValue chunk = globals.load(source, name);
        scripts.add(new Script(name, new LuaThread(globals, chunk)));
    }

    public boolean isRunning() {
        return !scripts.isEmpty();
    }

    public void clear() {
        scripts.clear();
    }

    public void update(float deltaSeconds) {
        Iterator<Script> it = scripts.iterator();
        while (it.hasNext()) {
            Script script = it.next();
            if (script.action != null) {
                if (!script.action.update(deltaSeconds)) {
                    continue;
                }
                script.action = null;
            }
            current = script;
            Varargs result = script.thread.resume(LuaValue.NONE);
            current = null;
            if (!result.arg1().toboolean()) {
                Gdx.app.error("lua", script.name + ": " + result.arg(2).tojstring());
                it.remove();
            } else if ("dead".equals(script.thread.getStatus())) {
                Gdx.app.log("lua", script.name + " finished");
                it.remove();
            }
        }
    }
}
