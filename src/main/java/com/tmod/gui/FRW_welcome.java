/**
 * @author Era (Sou1ence)
 */

package com.tmod.gui;

import com.tmod.cli.commands.CliBridge;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.image.Image;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.scene.text.Text;
import javafx.scene.text.TextAlignment;
import javafx.stage.DirectoryChooser;
import javafx.stage.Modality;
import javafx.stage.Stage;

import java.util.Objects;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.prefs.Preferences;
import java.io.File;


public class FRW_welcome {
    private static final String PREF_INITIALIZED = "initialized";
    private static final String PREF_LAST_DIRECTORY = "lastDirectory";

    private static Preferences prefs = Preferences.userNodeForPackage(FRW_welcome.class);

    // WELCOME SCREEN STEPS
    private static final int STEP_WELCOME = 0;
    private static final int STEP_DIR = 1;
    private static final int STEP_LOAD = 2;
    private static final int STEP_VERSION = 3;
    private static final int STEP_FINISH = 4;


    private static int currentStep = STEP_WELCOME;  // default step is WELCOME
    private static Stage wizardStage;

    private static String selectedDir ="";
    private static String selectedLoader="forge"; // default loader is Forge ({"forge", "fabric","quilt"})
    private static String selectedVersion="1.20.1"; // default version is  1.20.1

    // <------- UI COMPS -------->
    private static Label stepLb;
    private static VBox contentAr;
    private static Button nextBtn;
    private static Button backBtn;


    /**
     * Need an initialization checker
     *
     * @return true or false flag
     */
    public static boolean needsInitialization() {
        return !prefs.getBoolean(PREF_INITIALIZED, false);
    }


    /**
     * Flag setter for initialization
     * This is used to mark that the wizard has been completed
     */
    public static void setInitialized() {
        prefs.putBoolean(PREF_INITIALIZED, true);
    }

    /**
     * Flag setter for initialization
     *
     */
    public static void setPrefInitialized() {
        prefs.putBoolean(PREF_INITIALIZED, true);
    }


    /**
     * Last directory setter
     *
     * @param directory the directory to set as last used
     */
    public static void setLastDirectory(String directory) {
        prefs.put(PREF_LAST_DIRECTORY, directory);
    }

    /**
     * Get the last used directory
     *
     * @return the last used directory or user home if not set
     */
    public static String getLastDirectory() {
        return prefs.get(PREF_LAST_DIRECTORY, System.getProperty("user.home"));
    }


    /**
     * Show the welcome screen
     *
     * @param welcome the stage to show the welcome screen on
     * @return true if the user completed the wizard, false otherwise
     */
    public static boolean show (Stage welcome) {
      AtomicBoolean success = new AtomicBoolean(false);

      // Lock the main GUI while the wizard is open
        wizardStage = new Stage();
        wizardStage.initOwner(welcome);
        wizardStage.initModality(Modality.APPLICATION_MODAL);
        wizardStage.setTitle("Welcome to TMod");

        VBox root = new VBox(10);
        root.setPadding(new Insets(20));
        root.getStyleClass().add("wizard-container");

        stepLb = new Label("Welcome to TMod Manager");
        stepLb.getStyleClass().add("wizard-title");

        contentAr = new VBox(15);
        contentAr.getStyleClass().add("wizard-content");
        VBox.setVgrow(contentAr, javafx.scene.layout.Priority.ALWAYS);

        // BTNs
        backBtn = TModGui.createStyledButton("", "wizard-button", "ARROW_LEFT");
        backBtn.setDisable(true);
        backBtn.getStyleClass().add("wizard-button");
        backBtn.setOnAction(e -> navigateToPreviosStep());

        nextBtn = TModGui.createStyledButton("Next", "wizard-button", "ARROW_RIGHT");
        nextBtn.getStyleClass().add("wizard-button");
        nextBtn.setDefaultButton(true);
        nextBtn.setOnAction(e -> navigateToNextStep());

        HBox buttonBar = new HBox(10, backBtn, nextBtn);
        buttonBar.setAlignment(javafx.geometry.Pos.CENTER_RIGHT);

        root.getChildren().addAll(stepLb,contentAr, buttonBar);

        updateStepContent();

        Scene welcomeScene = new Scene(root, 600, 370);
        welcomeScene.getStylesheets().add(Objects.requireNonNull(FRW_welcome.class.getResource("/stylesheet/wizard.css")).toExternalForm());

        wizardStage.setScene(welcomeScene);
        wizardStage.setResizable(false);


        wizardStage.setScene(welcomeScene);
        wizardStage.setResizable(false);
        wizardStage.show();


        wizardStage.getIcons().add(
                new Image(Objects.requireNonNull(
                        FRW_welcome.class.getResourceAsStream("/images/png/logo_tmod_bg(0)_zoomed.png")
                ))
        );

        Platform.runLater(() -> {
            TitleBarCustomizer.applyTheme(wizardStage,
                    "#3a3a38",
                    "#f9f8f4",
                    "#5a5a58"
            );
        });


        wizardStage.showAndWait();

        return success.get();
    }

    private static void navigateToNextStep() {
        if (currentStep == STEP_DIR && (selectedDir== null || selectedDir.trim().isEmpty())) {
            showAlert("Choose the folder", "Please select a folder for the modpack.");
            return;
        }

        if (currentStep < STEP_FINISH) {
            currentStep++;
            updateStepContent();
        } else {
            initializeRepository();
        }

    }

    /**
     * REPO INITIALIZATION
     */
    private static void initializeRepository() {
        // Disable buttons and show progress
        backBtn.setDisable(true);
        nextBtn.setDisable(true);

        // Create ProgressIndicator
        ProgressIndicator progress = new ProgressIndicator();
        progress.setPrefSize(50, 50);
        Label progressLabel = new Label("Initializing modpack...");

        VBox progressBox = new VBox(10, progress, progressLabel);
        progressBox.setAlignment(Pos.CENTER);
        contentAr.getChildren().add(progressBox);

        // Launch initialization in a background thread
        new Thread(() -> {
            try {
                // Run the init command with selected parameters
                String result = CliBridge.run("init",
                        "--repo", selectedDir,
                        "--loader", selectedLoader,
                        "--version", selectedVersion);

                Platform.runLater(() -> {
                    ///  SAVE INITIALIZATION stats
                    setInitialized();
                    wizardStage.close();
                });

            } catch (Exception e) {
                Platform.runLater(() -> {
                    contentAr.getChildren().remove(progressBox);
                    showAlert("Initialization error",
                            "Failed to initialize modpack: " + e.getMessage());
                    backBtn.setDisable(false);
                    nextBtn.setDisable(false);
                });
            }
        }).start();
    }

    private static void navigateToPreviosStep() {
        if ( currentStep > STEP_WELCOME) {
            currentStep--;
            updateStepContent();
        }
    }

    private static void updateStepContent() {
        contentAr.getChildren().clear();

        switch (currentStep) {
            case STEP_WELCOME -> createWelcomeStep();
            case STEP_DIR -> createDirectoryStep();
            case STEP_LOAD -> createLoaderStep();
            case STEP_VERSION -> createVersionStep();
            case STEP_FINISH -> createFinishStep();
        }

        backBtn.setDisable(currentStep == STEP_WELCOME);
        nextBtn.setText(currentStep == STEP_FINISH ? "Finish" : "Next");
    }
    /**
     * Step 1: Welcome
     */
    private static void createWelcomeStep() {
        stepLb.setText("Welcome to TMod Manager");

        Text welcomeText = new Text(
                "This wizard will help you set up your first Minecraft modpack\n" +
                        "I'll guide you through a few steps to get started with mods");
        welcomeText.getStyleClass().add("wizard-content");
        welcomeText.setTextAlignment(TextAlignment.CENTER);
        welcomeText.setWrappingWidth(600);
        // welcomeText.setWrapText(true);

        Text icon = FontAwesomeIcon.createIcon("GAMEPAD", "welcome-icon", 60);
        VBox content = new VBox(20, icon, welcomeText);
        content.setAlignment(Pos.CENTER);

        contentAr.getChildren().add(content);
    }

    /**
     * Step 2: Select Directory
     */
    private static void createDirectoryStep() {
        stepLb.setText("Choose a folder for your modpack");

        Text infoText = new Text("Select the folder where your modpack will be stored.\n" +
                "This is usually the .minecraft/mods folder or a separate folder for your modpack.");
        infoText.getStyleClass().add("wizard-content");


        TextField directoryField = new TextField(selectedDir);
        directoryField.setEditable(false);
        directoryField.setPrefWidth(400);
        directoryField.setPromptText("Select a folder for your modpack");
        directoryField.getStyleClass().add("directory-field");

        Button browseButton = new Button("Browse...");
        browseButton.getStyleClass().add("browse-button");
        browseButton.setOnAction(e -> {
            DirectoryChooser chooser = new DirectoryChooser();
            chooser.setTitle("Choose folder for your modpack");

            // Set initial directory
            String initialDir = getLastDirectory();
            if (!initialDir.isEmpty()) {
                File dir = new File(initialDir);
                if (dir.exists()) {
                    chooser.setInitialDirectory(dir);
                }
            }

            File directory = chooser.showDialog(wizardStage);
            if (directory != null) {
                selectedDir= directory.getAbsolutePath();
                directoryField.setText(selectedDir);
                setLastDirectory(directory.getParent()); // Save parent directory (!)
            }
        });

        HBox directoryBox = new HBox(10, directoryField, browseButton);
        HBox.setHgrow(directoryField, Priority.ALWAYS);

        Label reminderLabel = new Label("* Folder will be created if it doesn't exist");
        reminderLabel.getStyleClass().add("reminder-text");

        contentAr.getChildren().addAll(infoText, directoryBox, reminderLabel);
    }

    /**
     * Step 3: Select Mod Loader
     */
    private static void createLoaderStep() {
        stepLb.setText("Choose a mod loader");

        Text infoText = new Text("A mod loader allows Minecraft to run with mods.\n" +
                "Choose your preferred loader:");
        infoText.getStyleClass().add("wizard-content");

        ToggleGroup loaderGroup = new ToggleGroup();

        RadioButton forgeButton = new RadioButton("Forge");
        forgeButton.setToggleGroup(loaderGroup);
        forgeButton.setUserData("forge");
        forgeButton.setSelected(selectedLoader.equals("forge"));

        RadioButton fabricButton = new RadioButton("Fabric");
        fabricButton.setToggleGroup(loaderGroup);
        fabricButton.setUserData("fabric");
        fabricButton.setSelected(selectedLoader.equals("fabric"));

        RadioButton quiltButton = new RadioButton("Quilt");
        quiltButton.setToggleGroup(loaderGroup);
        quiltButton.setUserData("quilt");
        quiltButton.setSelected(selectedLoader.equals("quilt"));


        forgeButton.getStyleClass().add("radio-button");
        fabricButton.getStyleClass().add("radio-button");
        quiltButton.getStyleClass().add("radio-button");

        // Description for each loader
        Text descriptionText = new Text("Forge - the classic loader with a huge mod library");
        descriptionText.getStyleClass().add("loader-description");

        loaderGroup.selectedToggleProperty().addListener((obs, oldVal, newVal) -> {
            if (newVal != null) {
                selectedLoader = newVal.getUserData().toString();
                switch (selectedLoader) {
                    case "forge" -> descriptionText.setText(
                            "Forge - the classic loader with a huge mod library");
                    case "fabric" -> descriptionText.setText(
                            "Fabric - modern, lightweight, and frequently updated");
                    case "quilt" -> descriptionText.setText(
                            "Quilt - a fork of Fabric with extra features");
                }
            }
        });

        VBox loaderBox = new VBox(10, forgeButton, fabricButton, quiltButton);

        contentAr.getChildren().addAll(infoText, loaderBox, descriptionText);
    }

    /**
     * Step 4: Select Minecraft Version
     */
    private static void createVersionStep() {
        stepLb.setText("Choose Minecraft version");

        Label infoLabel = new Label("Select the Minecraft version for your modpack:");
        infoLabel.getStyleClass().add("wizard-content-label");
        infoLabel.setWrapText(true);

        ComboBox<String> versionComboBox = new ComboBox<>();
        versionComboBox.getItems().addAll(
                // 1.20
                "1.20.4", "1.20.2", "1.20.1", "1.20",

                // 1.19
                "1.19.4", "1.19.3", "1.19.2", "1.19.1", "1.19",

                // 1.18
                "1.18.2", "1.18.1", "1.18",

                // 1.17
                "1.17.1", "1.17",

                // 1.16
                "1.16.5", "1.16.4", "1.16.3", "1.16.2", "1.16.1", "1.16",

                // 1.15
                "1.15.2", "1.15.1", "1.15",

                // 1.14
                "1.14.4", "1.14.3", "1.14.2", "1.14.1", "1.14",

                // 1.13
                "1.13.2", "1.13.1", "1.13",

                // 1.12
                "1.12.2", "1.12.1", "1.12",

                // 1.11
                "1.11.2", "1.11.1", "1.11",

                // 1.10
                "1.10.2", "1.10.1", "1.10",

                // 1.9
                "1.9.4", "1.9.3", "1.9.2", "1.9.1", "1.9",

                // 1.8
                "1.8.9", "1.8.8", "1.8.7", "1.8.6", "1.8.5", "1.8.4", "1.8.3", "1.8.2", "1.8.1", "1.8",

                // 1.7
                "1.7.10", "1.7.9", "1.7.8", "1.7.7", "1.7.6", "1.7.5", "1.7.4", "1.7.2",

                // 1.6
                "1.6.4", "1.6.2", "1.6.1",

                // 1.5
                "1.5.2", "1.5.1", "1.5",

                // 1.4
                "1.4.7", "1.4.6", "1.4.5", "1.4.4", "1.4.3", "1.4.2", "1.4.1", "1.4",

                // 1.3
                "1.3.2", "1.3.1",

                // 1.2
                "1.2.5", "1.2.4", "1.2.3", "1.2.2", "1.2.1",

                // 1.1
                "1.1",

                // 1.0
                "1.0.0"
        );

        versionComboBox.setValue(selectedVersion);
        versionComboBox.getStyleClass().add("version-combo-box");
        versionComboBox.valueProperty().addListener((obs, oldVal, newVal) -> {
            if (newVal != null) {
                selectedVersion = newVal;
            }
        });

        Label compatibilityLabel = new Label(
                "Make sure the selected version is compatible with the mods you plan to use.");
        compatibilityLabel.setWrapText(true);
        compatibilityLabel.getStyleClass().add("reminder-text");

        contentAr.getChildren().addAll(infoLabel, versionComboBox, compatibilityLabel);
    }

    /**
     * Step 5: Finish
     */
    private static void createFinishStep() {
        stepLb.setText("All set!");

        stepLb.setStyle("-fx-padding: 20 0 0 0;");

        Text icon = FontAwesomeIcon.createIcon("CHECK_CIRCLE", "success-icon", 30);

        Text summaryText = new Text("Your modpack has been configured with the following settings:");
        summaryText.getStyleClass().add("wizard-content");

        GridPane summaryGrid = new GridPane();
        summaryGrid.setHgap(15);
        summaryGrid.setVgap(10);
        summaryGrid.getStyleClass().add("wizard-summary");

        // MODPACK FOLDER
        Label folderLabel = new Label("Modpack Folder:");
        folderLabel.getStyleClass().add("wizard-summary-label");

        Label folderValue = new Label(selectedDir);
        folderValue.getStyleClass().add("wizard-summary-value");

        // LOADER
        Label loaderLabel = new Label("Mod Loader:");
        loaderLabel.getStyleClass().add("wizard-summary-label");

        Label loaderValue = new Label(selectedLoader);
        loaderValue.getStyleClass().add("wizard-summary-value");

        // VERSION
        Label versionLabel = new Label("Minecraft Version:");
        versionLabel.getStyleClass().add("wizard-summary-label");

        Label versionValue = new Label(selectedVersion);
        versionValue.getStyleClass().add("wizard-summary-value");


        summaryGrid.add(folderLabel, 0, 0);
        summaryGrid.add(folderValue, 1, 0);
        summaryGrid.add(loaderLabel, 0, 1);
        summaryGrid.add(loaderValue, 1, 1);
        summaryGrid.add(versionLabel, 0, 2);
        summaryGrid.add(versionValue, 1, 2);


        summaryGrid.getStyleClass().add("wizard-summary");

        Label finishLabel = new Label(
                "Click 'Finish' to create the modpack and start adding your mods.");
        finishLabel.getStyleClass().add("wizard-content-label");

        VBox content = new VBox(3, icon, summaryText, summaryGrid, finishLabel);
        content.setAlignment(Pos.CENTER);

        contentAr.getChildren().add(content);
    }


    /**
     * Show an alert dialog
     *
     * @param title
     * @param message
     */
    private static void showAlert(String title, String message) {
        Alert alert = new Alert(Alert.AlertType.ERROR);
        alert.setTitle(title);
        alert.setHeaderText(null);
        alert.setContentText(message);

        Stage alertStage = (Stage) alert.getDialogPane().getScene().getWindow();

        alertStage.getIcons().add(
                new Image(Objects.requireNonNull(
                        FRW_welcome.class.getResourceAsStream("/images/png/logo_tmod_bg(0)_zoomed.png")
                ))
        );

        Scene scene = alert.getDialogPane().getScene();
        if (scene != null) {
            scene.getStylesheets().add(
                    Objects.requireNonNull(FRW_welcome.class.getResource("/stylesheet/wizard.css")).toExternalForm()
            );
        }

        alert.show();

        Platform.runLater(() -> {
            TitleBarCustomizer.applyTheme(alertStage,
                    "#3a3a38",
                    "#f9f8f4",
                    "#5a5a58"
            );
        });

        alert.showAndWait();
    }
}
