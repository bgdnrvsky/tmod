package com.tmod.gui;


import javafx.application.Application;
import javafx.stage.Stage;

/**
 * Main class for the graphical user interface version of tmod
 */
public class TModGui extends Application {

    /**
     * Entry point for the GUI version of tmod
     */
    public static void main(String[] argv) {
        System.out.println("tmod GUI");
    }

    @Override public void start(Stage primaryStage) {
        primaryStage.setTitle("tmod GUI");
        primaryStage.show();

    }
}
