package com.tmod.gui;

import javafx.concurrent.Task;

import java.util.concurrent.Callable;

/**
 * Utility class for working with asynchronous JavaFX tasks
 */
public class FxTasks {

    /**
     * Creates an asynchronous task that runs in a background thread
     * @param callable the function to be executed
     * @return the created task
     */
    public static <T> Task<T> background(Callable<T> callable) {
        return new Task<>() {
            @Override
            protected T call() throws Exception {
                return callable.call();
            }
        };
    }
}
