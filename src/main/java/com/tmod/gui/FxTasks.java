package com.tmod.gui;

import javafx.concurrent.Task;

import java.util.function.Supplier;

public final class FxTasks {

    private FxTasks() {}

    public static <T> Task<T> background(Supplier<T> supplier) {
        return new Task<>() {
            @Override
            protected T call() {
                return supplier.get();
            }
        };
    }
}