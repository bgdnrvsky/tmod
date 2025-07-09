/**
 * @author: Era
 */

package com.tmod.gui;

import com.tmod.cli.commands.CliBridge;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.layout.*;
import javafx.stage.DirectoryChooser;
import javafx.stage.Stage;
import java.io.File;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.Objects;

/**
 * TModGui - user interface for TMod Manager
 * Application uses JavaFX to provide a graphical interface
 */
public class TModGui extends Application {

    // Data
    private final ObservableList<String> mods = FXCollections.observableArrayList();
    private final TextArea logArea = new TextArea();
    private final ListView<String> modListView = new ListView<>(mods);
    private final Label statusLabel = new Label("Ready");
    private final ProgressBar progressBar = new ProgressBar();
    private final Label modCountLabel = new Label("0 mods installed");

    // UI Comps
    private Button addBtn, removeBtn, installBtn, refreshBtn;

    /**
     * Entry point for the GUI version of tmod
     */
    public static void main(String[] argv) {
        System.out.println("TMod Manager GUI");
        launch(argv);
    }

    @Override
    public void start(Stage primaryStage) {
        primaryStage.setTitle("TMod Manager");
        primaryStage.setMinWidth(800);
        primaryStage.setMinHeight(700);

        // Create main layout
        BorderPane root = createMainLayout();

        // Create and apply scene with CSS
        Scene scene = new Scene(root, 800, 550);
        scene.getStylesheets().add(Objects.requireNonNull(getClass().getResource("/style.css")).toExternalForm());

        primaryStage.setScene(scene);
        primaryStage.show();

        // Initialize data
        refreshModsList();
        updateModCount();
    }

    /**
     * Creates the main application layout
     */
    private BorderPane createMainLayout() {
        BorderPane root = new BorderPane();
        root.getStyleClass().add("main-container");

        // Header
        root.setTop(createHeader());

        // Main content area
        root.setCenter(createMainContent());

        // Footer with status
        root.setBottom(createFooter());

        return root;
    }

    /**
     * Creates the application header with title and toolbar
     */
    private VBox createHeader() {
        // Title section
        Label titleLabel = new Label("TMod Manager");
        titleLabel.getStyleClass().add("app-title");

        Label subtitleLabel = new Label("Minecraft Mods at your service");
        subtitleLabel.getStyleClass().add("app-subtitle");

        VBox titleBox = new VBox(5, titleLabel, subtitleLabel);
        titleBox.getStyleClass().add("title-section");

        // Toolbar
        HBox toolbar = createToolbar();

        VBox header = new VBox(titleBox, toolbar);
        header.getStyleClass().add("header-section");

        return header;
    }

    /**
     * Creates the main toolbar with action buttons
     */
    private HBox createToolbar() {
        addBtn = createStyledButton("Add Mod", "add-button");
        removeBtn = createStyledButton("Remove Mod", "remove-button");
        installBtn = createStyledButton("Install All", "install-button");
        refreshBtn = createStyledButton("Refresh", "refresh-button");

        // Event handlers
        addBtn.setOnAction(e -> onAddMod());
        removeBtn.setOnAction(e -> onRemoveMod(modListView.getSelectionModel().getSelectedItem()));
        installBtn.setOnAction(e -> onInstallMods());
        refreshBtn.setOnAction(e -> refreshModsList());

        // Disable remove button initially
        removeBtn.setDisable(true);

        // TODO: change separator to a custom styled one
        HBox toolbar = new HBox(15, addBtn, removeBtn, new Separator(), installBtn, refreshBtn);
        toolbar.getStyleClass().add("toolbar");
        toolbar.setAlignment(Pos.CENTER_LEFT);

        return toolbar;
    }

    /**
     * Creates the main content area with mod list and log
     */
    private HBox createMainContent() {
        // Left panel - Mod list
        VBox leftPanel = createModListPanel();

        // Right panel - Log output
        VBox rightPanel = createLogPanel();

        HBox mainContent = new HBox(10, leftPanel, rightPanel);
        mainContent.getStyleClass().add("main-content");

        // Flexible sizing
        HBox.setHgrow(leftPanel, Priority.ALWAYS);
        HBox.setHgrow(rightPanel, Priority.ALWAYS);

        return mainContent;
    }

    /**
     * Creates the mod list panel
     */
    private VBox createModListPanel() {
        Label modListTitle = new Label("Installed Mods");
        modListTitle.getStyleClass().add("panel-title");

        // Configure mod list view
        modListView.getStyleClass().add("mod-list");
        modListView.setPlaceholder(new Label("No mods installed yet\n Just click 'Add Mod' bro!"));

        // SListener to enable/disable remove button
        modListView.getSelectionModel().selectedItemProperty().addListener((obs, oldVal, newVal) -> {
            removeBtn.setDisable(newVal == null || newVal.isBlank());
        });


        modCountLabel.getStyleClass().add("mod-count-label");

        VBox modPanel = new VBox(10, modListTitle, modListView, modCountLabel);
        modPanel.getStyleClass().add("mod-panel");
        VBox.setVgrow(modListView, Priority.ALWAYS);

        return modPanel;
    }

    /**
     * Creates the log output panel
     */
    private VBox createLogPanel() {
        Label logTitle = new Label("Activity Log");
        logTitle.getStyleClass().add("panel-title");

        // Configure log area
        logArea.setEditable(false);
        logArea.setWrapText(true);
        logArea.getStyleClass().add("log-area");
        logArea.appendText(getCurrentTime() + " TMod Manager initialized\n");

        // Clear log button
        Button clearLogBtn = new Button("Clear Log");
        clearLogBtn.getStyleClass().add("clear-log-button");
//        clearLogBtn.setPadding(new Insets(0,0,20,0));
        clearLogBtn.setOnAction(e -> logArea.clear());

        HBox logHeader = new HBox(logTitle);
        logHeader.getChildren().add(clearLogBtn);
        HBox.setHgrow(logTitle, Priority.ALWAYS);
        logHeader.setAlignment(Pos.CENTER_LEFT);
        logHeader.setSpacing(10);

        VBox logPanel = new VBox(10, logHeader, logArea);
        logPanel.getStyleClass().add("log-panel");
        VBox.setVgrow(logArea, Priority.ALWAYS);

        return logPanel;
    }

    /**
     * Creates the footer with status information
     */
    private HBox createFooter() {
        statusLabel.getStyleClass().add("status-label");

        progressBar.getStyleClass().add("progress-bar");
        progressBar.setVisible(false);
        progressBar.setPrefWidth(200);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        HBox footer = new HBox(15, statusLabel, spacer, progressBar);
        footer.getStyleClass().add("footer");

        return footer;
    }

    /**
     * Creates a styled button with consistent appearance
     */
    private Button createStyledButton(String text, String styleClass) {
        Button button = new Button(text);
        button.getStyleClass().addAll("styled-button", styleClass);
        button.setMinWidth(120);
        return button;
    }

    /** USER ACTIONS */

    private void onAddMod() {
        DirectoryChooser chooser = new DirectoryChooser();
        chooser.setTitle("Select Mod Directory");

        File dir = chooser.showDialog(logArea.getScene().getWindow());
        if (dir == null) return;

        updateStatus("Adding mod from: " + dir.getName());
        runCliAndShow("add", dir.getAbsolutePath());
    }

    private void onRemoveMod(String modName) {
        if (modName == null || modName.isBlank()) return;

        // Confirmation dialog
        Alert alert = new Alert(Alert.AlertType.CONFIRMATION);
        alert.setTitle("Confirm Removal");
        alert.setHeaderText("Remove Mod");
        alert.setContentText("Are you sure you want to remove '" + modName + "'?");

        if (alert.showAndWait().orElse(ButtonType.CANCEL) == ButtonType.OK) {
            updateStatus("Removing mod: " + modName);
            runCliAndShow("remove", modName);
        }
    }

    private void onInstallMods() {
        if (mods.isEmpty()) {
            showInfoDialog("No Mods", "No mods to install, MyBoy. Add some mods first");
            return;
        }

        updateStatus("Installing all mods...");
        runCliAndShow("install");
    }

    private void refreshModsList() {
        updateStatus("Refreshing mod list...");
        showProgress(true);

        Task<String> task = FxTasks.background(() -> CliBridge.run("list"));
        task.setOnSucceeded(e -> {
            String output = task.getValue();
            Platform.runLater(() -> {
                mods.setAll(output.lines().filter(line -> !line.isBlank()).toList());
                updateModCount();
                appendLog("Mod list refreshed");
                updateStatus("Ready");
                showProgress(false);
            });
        });

        task.setOnFailed(e -> {
            Platform.runLater(() -> {
                appendLog("Error refreshing mod list: " + task.getException().getMessage());
                updateStatus("Error occurred");
                showProgress(false);
            });
        });

        new Thread(task).start();
    }

    private void runCliAndShow(String... args) {
        showProgress(true);

        Task<String> task = FxTasks.background(() -> CliBridge.run(args));
        task.setOnSucceeded(ev -> {
            String output = task.getValue();
            Platform.runLater(() -> {
                appendLog(output);
                refreshModsList();
                updateStatus("Operation completed");
                showProgress(false);
            });
        });

        task.setOnFailed(ev -> {
            Platform.runLater(() -> {
                appendLog("Error: " + task.getException().getMessage());
                updateStatus("Error occurred");
                showProgress(false);
            });
        });

        new Thread(task).start();
    }

    /** UTILITY METHODS */

    private void appendLog(String text) {
        Platform.runLater(() -> {
            logArea.appendText(getCurrentTime() + " " + text + System.lineSeparator());
            logArea.setScrollTop(Double.MAX_VALUE);
        });
    }

    private void updateStatus(String status) {
        Platform.runLater(() -> statusLabel.setText(status));
    }

    private void showProgress(boolean show) {
        Platform.runLater(() -> progressBar.setVisible(show));
    }

    private void updateModCount() {
        Platform.runLater(() -> {
            int count = mods.size();
            modCountLabel.setText(count + " mod" + (count != 1 ? "s" : "") + " installed");
        });
    }

    private String getCurrentTime() {
        return LocalTime.now().format(DateTimeFormatter.ofPattern("HH:mm:ss"));
    }

    private void showInfoDialog(String title, String message) {
        Alert alert = new Alert(Alert.AlertType.INFORMATION);
        alert.setTitle(title);
        alert.setHeaderText(null);
        alert.setContentText(message);
        alert.showAndWait();
    }
}
