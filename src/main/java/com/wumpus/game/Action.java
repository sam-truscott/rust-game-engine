package com.wumpus.game;

/** Something a script waits on; the script resumes once {@link #update} returns true. */
@FunctionalInterface
public interface Action {
    boolean update(float deltaSeconds);
}
