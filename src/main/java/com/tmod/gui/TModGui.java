package com.tmod.gui;


import com.tmod.cli.commands.CliBridge;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.TitledPane;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.VBox;
import javafx.stage.DirectoryChooser;
import javafx.stage.Stage;
import javafx.scene.control.Button;
import javafx.geometry.Insets;
import java.io.File;


/**
 * Main class for the graphical user interface version of tmod
 */
public class TModGui extends Application {

    private final ObservableList<String> mods = FXCollections.observableArrayList();
    private final javafx.scene.control.TextArea logArea = new javafx.scene.control.TextArea();

    /**
     * Entry point for the GUI version of tmod
     */
    public static void main(String[] argv) {
        System.out.println("tmod GUI");
    }

    @Override public void start(Stage primaryStage) {
        primaryStage.setTitle("TMod Manager");

        /** TOP TOOLBAR (BUTTONS) */
        Button addBtn = new Button("Add");
        Button removeBtn = new Button("Remove");
        Button installBtn = new Button("Install");
        Button refreshBtn = new Button("Refresh");

        HBox topBar = new HBox(10, addBtn, removeBtn, installBtn, refreshBtn);
        topBar.setPadding(new Insets(10));
        topBar.setAlignment(Pos.CENTER_LEFT);


        /** CENTER (MOD LIST) */
        ListView <String> listView = new ListView<>(mods);
        VBox centerBox = new VBox(5, new Label("Installed Mods:"), listView);
        centerBox.setPadding(new Insets(5));

        /** BOTTOM (LOG OUTPUT) */
        logArea.setEditable(false);
        logArea.setPrefRowCount(8);
        TitledPane logPane = new TitledPane("Log Output", logArea);
        logPane.setCollapsible(false);

        /** ROOT LAYOUT */
        BorderPane root = new BorderPane();
        root.setTop(topBar);
        root.setCenter(centerBox);
        root.setBottom(logPane);


        /** EVENT HANDLERS */
        addBtn.setOnAction(e ->  onAddMod());
        removeBtn.setOnAction(e -> onRemoveMod(listView.getSelectionModel().getSelectedItem()));
        installBtn.setOnAction(e -> onInstallMods());
        refreshBtn.setOnAction(e -> refreshModsList());

        refreshModsList();

        Scene scene = new Scene(root, 600, 600);
        primaryStage.setScene(scene);
        primaryStage.show();

        primaryStage.show();
    }


    /** USER ACTIONS */

    private void onAddMod() {
        DirectoryChooser ch = new DirectoryChooser();
       ch.setTitle("Select Mod Directory");


       File dir = ch.showDialog(logArea.getScene().getWindow());

       if(dir == null)
           return;

       runCliAndShow("add", dir.getAbsolutePath());

    }

    private void onRemoveMod(String modName) {
        if (modName == null || modName.isBlank()) return;
        runCliAndShow("remove", modName);
    }

    private void onInstallMods() {
        runCliAndShow("install");
    }

    private void refreshModsList() {
        Task<String> task = FxTasks.background(() -> CliBridge.run("list"));
        task.setOnSucceeded(e -> {
            String output = task.getValue();
            mods.setAll(output.lines().toList());
            appendLog(output);
        });
        new Thread(task).start();
    }

    private void runCliAndShow(String... args) {
        Task<String> task = FxTasks.background(() -> CliBridge.run(args));
        task.setOnSucceeded(ev -> {
            String out = task.getValue();
            appendLog(out);
            refreshModsList();
        });
        new Thread(task).start();
    }

    private void appendLog(String text) {
        Platform.runLater(() -> logArea.appendText(text + System.lineSeparator()));
    }


}
