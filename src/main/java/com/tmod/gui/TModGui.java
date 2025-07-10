/**
 * @author: Era
 */

package com.tmod.gui;

import com.tmod.cli.commands.CliBridge;
import javafx.animation.KeyFrame;
import javafx.animation.KeyValue;
import javafx.animation.Timeline;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import javafx.scene.paint.Paint;
import javafx.scene.text.Text;
import javafx.stage.DirectoryChooser;
import javafx.stage.Modality;
import javafx.stage.Stage;
import java.io.File;
import java.time.Duration;
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

    // Animation fields
    private Timeline statusTimeline;
    private Timeline progressTimeline;
    private Text statusIcon;
    private Text progressIcon;
    private HBox progressBox;

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
        scene.getStylesheets().add(Objects.requireNonNull(getClass().getResource("/stylesheet/style.css")).toExternalForm());


        primaryStage.getIcons().add(
                new Image(Objects.requireNonNull(
                        getClass().getResourceAsStream("/images/png/logo_tmod_bg(0)_zoomed.png")
                ))
        );

        primaryStage.setScene(scene);
        primaryStage.show();

        Platform.runLater(() -> {
                TitleBarCustomizer.applyTheme(primaryStage,
                      "#3a3a38",
                        "#f9f8f4",
                      "#5a5a58"
                );

                // WELCOME WIZARD needs checking
//                if (FRW_welcome.needsInitialization()) {
//                    showFirstRunWizard(primaryStage);
//                } else {
//                    // Initialize data
//                    refreshModsList();
//                    updateModCount();
//                }
            FRW_welcome.show(primaryStage);
        });
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
        addBtn = createStyledButton("Add Mod", "add-button", "FOLDER");
        removeBtn = createStyledButton("Remove Mod", "remove-button", "TRASH_ALT");
        installBtn = createStyledButton("Install All", "install-button", "DOWNLOAD");
        refreshBtn = createStyledButton("Refresh", "refresh-button", "REFRESH");

        // Event handlers
        addBtn.setOnAction(e -> onAddMod());
        removeBtn.setOnAction(e -> onRemoveMod(modListView.getSelectionModel().getSelectedItem()));
        installBtn.setOnAction(e -> onInstallMods());
        refreshBtn.setOnAction(e -> refreshModsList());

        // Disable remove button initially
        removeBtn.setDisable(true);

        // Create HBoxes for left and right sides
        HBox leftBox = new HBox(15, addBtn, removeBtn);
        leftBox.setAlignment(Pos.CENTER_LEFT); // Align buttons to the left

        HBox rightBox = new HBox(15, installBtn, refreshBtn);
        rightBox.setAlignment(Pos.CENTER_RIGHT); // Align buttons to the right

        // Create the main toolbar, set the alignment to spread out the buttons
        HBox toolbar = new HBox(15, leftBox, rightBox);
        toolbar.setHgrow(leftBox, Priority.ALWAYS); // Left box grows and takes up space
        toolbar.setHgrow(rightBox, Priority.ALWAYS); // Right box grows and takes up space

        toolbar.getStyleClass().add("toolbar");

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

        // Реализация drag-and-drop для файлов модов
        modListView.setOnDragOver(event -> {
            if (event.getGestureSource() != modListView && 
                    event.getDragboard().hasFiles()) {
                event.acceptTransferModes(javafx.scene.input.TransferMode.COPY_OR_MOVE);
            }
            event.consume();
        });

        modListView.setOnDragDropped(event -> {
            javafx.scene.input.Dragboard db = event.getDragboard();
            boolean success = false;

            if (db.hasFiles()) {
                success = true;
                for (File file : db.getFiles()) {
                    if (file.getName().toLowerCase().endsWith(".jar")) {
                        updateStatus("Adding a mod: " + file.getName());
                        runCliAndShow("add", file.getAbsolutePath());
                    }
                }
            }

            event.setDropCompleted(success);
            event.consume();
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
    /**
     * Enhanced footer method with icon and blinking animation
     */
    private HBox createFooter() {
        // Status icon
        Text statusIcon = FontAwesomeIcon.createIcon("INFO_CIRCLE", "status-icon", 14);

        // Status label with icon
        HBox statusBox = new HBox(8);
        statusBox.setAlignment(Pos.CENTER_LEFT);

        statusBox.getChildren().addAll(statusIcon, statusLabel);
        statusBox.getStyleClass().add("status-box");

        statusLabel.getStyleClass().add("status-label");

        // Progress section
        Text progressIcon = FontAwesomeIcon.createIcon("SPINNER", "progress-icon", 14);
        progressBar.getStyleClass().add("progress-bar");
        progressBar.setVisible(false);
        progressBar.setPrefWidth(200);

        HBox progressBox = new HBox(8);
        progressBox.setAlignment(Pos.CENTER_RIGHT);
        progressBox.getChildren().addAll(progressIcon, progressBar);
        progressBox.getStyleClass().add("progress-box");
        progressIcon.setVisible(false);

        // Spacer
        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        // Main footer container
        HBox footer = new HBox(15, statusBox, spacer, progressBox);
        footer.getStyleClass().add("footer");

        // Store references for animation
        this.statusIcon = statusIcon;
        this.progressIcon = progressIcon;
        this.progressBox = progressBox;

        return footer;
    }

    /**
     * Enhanced progress display with animation
     */
    private void showProgress(boolean show) {
        Platform.runLater(() -> {
            progressBar.setVisible(show);
            progressIcon.setVisible(show);

            if (show) {
                startProgressAnimation();
            } else {
                stopProgressAnimation();
            }
        });
    }

    /**
     * Enhanced status update with icon animation
     */
    private void updateStatus(String status) {
        Platform.runLater(() -> {
            statusLabel.setText(status);

            // Change icon based on status
            if (status.toLowerCase().contains("error")) {
                statusIcon.getStyleClass().clear();
                statusIcon.getStyleClass().addAll("status-icon", "error-icon");
                statusIcon = FontAwesomeIcon.createIcon("EXCLAMATION_TRIANGLE", "status-icon error-icon", 14);
            } else if (status.toLowerCase().contains("completed")) {
                statusIcon.getStyleClass().clear();
                statusIcon.getStyleClass().addAll("status-icon", "success-icon");
                statusIcon = FontAwesomeIcon.createIcon("CHECK_CIRCLE", "status-icon success-icon", 14);
            } else {
                statusIcon.getStyleClass().clear();
                statusIcon.getStyleClass().addAll("status-icon");
                statusIcon = FontAwesomeIcon.createIcon("INFO_CIRCLE", "status-icon", 14);
            }

            // Update the icon in the status box
            HBox statusBox = (HBox) statusLabel.getParent();
            if (statusBox != null) {
                statusBox.getChildren().set(0, statusIcon);
            }

            // Trigger status animation
            startStatusAnimation();
        });
    }



    /**
     * Start blinking animation for status
     */
    private void startStatusAnimation() {
        if (statusTimeline != null) {
            statusTimeline.stop();
        }

        statusTimeline = new Timeline(
                new KeyFrame(javafx.util.Duration.ZERO, new KeyValue(statusIcon.opacityProperty(), 1.0)),
                new KeyFrame(javafx.util.Duration.seconds(0.8), new KeyValue(statusIcon.opacityProperty(), 0.3)),
                new KeyFrame(javafx.util.Duration.seconds(1.6), new KeyValue(statusIcon.opacityProperty(), 1.0))
        );
        statusTimeline.setCycleCount(43); // Blink 43 times
        statusTimeline.play();
    }

    /**
     * Start spinning animation for progress
     */
    private void startProgressAnimation() {
        if (progressTimeline != null) {
            progressTimeline.stop();
        }

        progressTimeline = new Timeline(
                new KeyFrame(javafx.util.Duration.ZERO, new KeyValue(progressIcon.rotateProperty(), 0)),
                new KeyFrame(javafx.util.Duration.seconds(1), new KeyValue(progressIcon.rotateProperty(), 360))
        );
        progressTimeline.setCycleCount(Timeline.INDEFINITE);
        progressTimeline.play();
    }

    /**
     * Stop progress animation
     */
    private void stopProgressAnimation() {
        if (progressTimeline != null) {
            progressTimeline.stop();
        }
        progressIcon.setRotate(0);
    }

    /**
     * Creates a styled button with consistent appearance
     * Structure :
     *           {BUTTON}
     *              |
     *       {[ICON][ ][TEXT]}
     */
    protected static Button createStyledButton(String text, String styleClass, String iconName) {
        Button button = new Button(text);
        button.getStyleClass().addAll("styled-button", styleClass);
        button.setMinWidth(120);

        // Add FontAwesome icon
        if (iconName != null) {
            Text icon = FontAwesomeIcon.createIcon(iconName, "button-icon", 16);
            icon.setFill(Paint.valueOf("#f9f8f4"));
            button.setGraphic(icon);
            button.setContentDisplay(ContentDisplay.LEFT);
            button.setGraphicTextGap(8);
        }

        return button;
    }

    /** USER ACTIONS */

    private void onAddMod() {

        // Create a dialog with tabs for adding mods (MOD CHOICE DIALOG)
        TabPane tabPane = new TabPane();
        tabPane.setTabClosingPolicy(TabPane.TabClosingPolicy.UNAVAILABLE);

        // Adding from folder tab
        Tab folderTab = new Tab("from folder");
        VBox folderContent = new VBox(10);
        folderContent.setPadding(new Insets(15));

        Label folderLabel = new Label("Select the folder containing the mod files:");
        TextField folderField = new TextField();
        folderField.setEditable(false);

        Button folderBrowseButton = new Button("Browse...");
        folderBrowseButton.setOnAction(e -> {
            DirectoryChooser chooser = new DirectoryChooser();
            chooser.setTitle("Chose the folder with mod files");

            // Use last directory from settings
            String lastDir = FRW_welcome.getLastDirectory();
            if (lastDir != null && !lastDir.isEmpty()) {
                File dir = new File(lastDir);
                if (dir.exists()) chooser.setInitialDirectory(dir);
            }

            File dir = chooser.showDialog(tabPane.getScene().getWindow());
            if (dir != null) {
                folderField.setText(dir.getAbsolutePath());
                FRW_welcome.setLastDirectory(dir.getParent());
            }
        });

        HBox folderBox = new HBox(10, folderField, folderBrowseButton);
        HBox.setHgrow(folderField, Priority.ALWAYS);

        Button addFolderButton = new Button("Add mod");
        addFolderButton.getStyleClass().add("action-button");
        addFolderButton.setOnAction(e -> {
            if (folderField.getText().isEmpty()) {
                showInfoDialog("Issue", "Choose a folder with mod files");
                return;
            }

            Stage stage = (Stage) addFolderButton.getScene().getWindow();
            stage.close();

            updateStatus("Adding mod from: " + folderField.getText());
            runCliAndShow("add", folderField.getText());
        });

        folderContent.getChildren().addAll(folderLabel, folderBox, addFolderButton);
        folderTab.setContent(folderContent);

        // Adding by URL tab
        Tab urlTab = new Tab("using URL");
        VBox urlContent = new VBox(10);
        urlContent.setPadding(new Insets(15));

        Label urlLabel = new Label("Enter the URL of the mod (CurseForge, Modrinth, etc.):");
        TextField urlField = new TextField();

        Button addUrlButton = new Button("Add mod");
        addUrlButton.getStyleClass().add("action-button");
        addUrlButton.setOnAction(e -> {
            if (urlField.getText().isEmpty()) {
                showInfoDialog("Issue", "Enter a valid URL for the mod");
                return;
            }

            Stage stage = (Stage) addUrlButton.getScene().getWindow();
            stage.close();

            updateStatus("Adding mod by URL: " + urlField.getText());
            runCliAndShow("add", "--url", urlField.getText());
        });

        urlContent.getChildren().addAll(urlLabel, urlField, addUrlButton);
        urlTab.setContent(urlContent);

        tabPane.getTabs().addAll(folderTab, urlTab);

        Stage dialogStage = new Stage();
        dialogStage.setTitle("Add mod");
        dialogStage.initModality(Modality.APPLICATION_MODAL);
        dialogStage.initOwner(logArea.getScene().getWindow());

        Scene scene = new Scene(tabPane, 500, 250);
        scene.getStylesheets().add(Objects.requireNonNull(getClass().getResource("/stylesheet/style.css")).toExternalForm());

        dialogStage.setScene(scene);
        dialogStage.setResizable(false);
        dialogStage.showAndWait();
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

    /**
     * Shows wizard for first run config
     */
    private void showFirstRunWizard(Stage primaryStage) {
        boolean success = FRW_welcome.show(primaryStage);

        refreshModsList();
        updateModCount();

        appendLog("Setup is complete, you are welcome to proceed");
        updateStatus("ModPack is ready");
    }
}
