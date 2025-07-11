/**
 * ModBrowser - A component for browsing and displaying mods from CurseForge
 *
 * This class handles fetching, displaying, and searching for mods from the CurseForge API.
 * It provides a visually appealing UI to browse through available mods and view their details.
 *
 * @author Era (Sou1ence)
 */

package com.tmod.gui;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.tmod.cli.commands.CliBridge;
import javafx.application.Platform;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import javafx.scene.text.Text;
import javafx.scene.text.TextAlignment;
import javafx.scene.text.TextFlow;
import javafx.stage.Modality;
import javafx.stage.Stage;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

public class ModBrowser {
    // API Constants
    private static final String CURSEFORGE_API_URL = "https://api.curseforge.com/v1";
    private static final String API_KEY =
            "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm";

    // UI Components
    private Stage browserStage;
    private VBox rootLayout;
    private ScrollPane modListScrollPane;
    private VBox modListContainer;
    private TextField searchField;
    private ComboBox<String> categoryFilter;
    private ComboBox<String> sortOrder;
    private Pagination pagination;
    private ProgressIndicator loadingIndicator;
    private Label statusLabel;
    private Button installButton;

    // Data
    private final ObjectMapper mapper;
    private final HttpClient httpClient;
    private ObservableList<ModCard> displayedMods = FXCollections.observableArrayList();
    private List<Category> categories = new ArrayList<>();
    private int currentPage = 0;
    private int pageSize = 30;
    private int totalResults = 0;
    private String currentSearchTerm = "";
    private String currentCategory = "All";
    private String currentSortOrder = "Featured";
    private ModDetail currentModDetail = null;

    // Parent stage reference
    private final Stage parentStage;

    /**
     * Creates a new ModBrowser instance
     *
     * @param parentStage The parent stage
     */
    public ModBrowser(Stage parentStage) {
        this.parentStage = parentStage;
        this.mapper = new ObjectMapper();
        mapper.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        this.httpClient = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .build();

        // Initialize UI components
        initializeUI();

        // Load categories in background
        loadCategories();
    }

    /**
     * Shows the mod browser window
     */
    public void show() {
        if (browserStage == null) {
            browserStage = new Stage();
            browserStage.initOwner(parentStage);
            browserStage.initModality(Modality.NONE);
            browserStage.setTitle("TMod - Mod Browser");
            browserStage.setMinWidth(950);
            browserStage.setMinHeight(700);

            Scene scene = new Scene(rootLayout, 950, 700);
            scene.getStylesheets().add(Objects.requireNonNull(
                    getClass().getResource("/stylesheet/style.css")).toExternalForm());
            browserStage.setScene(scene);

            // Add icon
            browserStage.getIcons().add(
                    new Image(Objects.requireNonNull(
                            getClass().getResourceAsStream("/images/png/logo_tmod_bg(0)_zoomed.png")
                    ))
            );

            // Show stage and apply theme
            browserStage.show();

            Platform.runLater(() -> {
                TitleBarCustomizer.applyTheme(browserStage,
                        "#3a3a38",
                        "#f9f8f4",
                        "#5a5a58"
                );

                // Load initial mods
                searchMods("");
            });
        } else {
            browserStage.show();
            browserStage.toFront();
        }
    }

    /**
     * Initializes all UI components
     */
    private void initializeUI() {
        rootLayout = new VBox(10);
        rootLayout.setPadding(new Insets(20));
        rootLayout.getStyleClass().add("mod-browser-container");

        // Header with title
        Label titleLabel = new Label("Mod Browser");
        titleLabel.getStyleClass().add("app-title");

        Label subtitleLabel = new Label("Discover and install mods for Minecraft");
        subtitleLabel.getStyleClass().add("app-subtitle");

        VBox titleBox = new VBox(5, titleLabel, subtitleLabel);
        titleBox.getStyleClass().add("title-section");

        // Search and filter controls
        HBox searchBox = createSearchControls();

        // Mod list area
        modListContainer = new VBox(10);
        modListContainer.setPadding(new Insets(10, 0, 10, 0));
        modListScrollPane = new ScrollPane(modListContainer);
        modListScrollPane.setFitToWidth(true);
        modListScrollPane.setFitToHeight(true);
        modListScrollPane.getStyleClass().add("mod-list-scroll");
        VBox.setVgrow(modListScrollPane, Priority.ALWAYS);

        // Loading indicator
        loadingIndicator = new ProgressIndicator();
        loadingIndicator.setMaxSize(50, 50);
        loadingIndicator.setVisible(false);

        // Status label
        statusLabel = new Label("Ready to browse");
        statusLabel.getStyleClass().add("status-label");

        // Pagination controls
        pagination = new Pagination();
        pagination.setPageCount(1);
        pagination.setCurrentPageIndex(0);
        pagination.setMaxPageIndicatorCount(5);
        pagination.getStyleClass().add("mod-pagination");
        pagination.setPageFactory(this::createPage);

        // Footer with status and loading
        HBox footer = new HBox(10, statusLabel, loadingIndicator);
        footer.setAlignment(Pos.CENTER_LEFT);

        // Layout
        VBox content = new VBox(10);
        content.getChildren().addAll(modListScrollPane, pagination);
        VBox.setVgrow(content, Priority.ALWAYS);

        rootLayout.getChildren().addAll(titleBox, searchBox, content, footer);
    }

    /**
     * Creates search controls including search field, category filter, and sort order
     */
    private HBox createSearchControls() {
        // Search field
        searchField = new TextField();
        searchField.setPromptText("Search mods...");
//        searchField.setPrefWidth(100);
        searchField.setOnAction(e -> searchMods(searchField.getText()));
        searchField.getStyleClass().add("search-field");

        Button searchButton = TModGui.createStyledButton("Search", "search-button", "SEARCH");
        searchButton.setOnAction(e -> searchMods(searchField.getText()));
        searchButton.getStyleClass().add("search-button");

        // Category filter
        Label categoryLabel = new Label("Category:");
        categoryLabel.getStyleClass().add("filter-label");

        categoryFilter = new ComboBox<>();
        categoryFilter.getItems().add("All");
        categoryFilter.setValue("All");
        categoryFilter.getStyleClass().add("category-filter"); // Add this line
        categoryFilter.setOnAction(e -> {
            if (categoryFilter.getValue() != null) {
                currentCategory = categoryFilter.getValue();
                currentPage = 0;
                searchMods(currentSearchTerm);
            }
        });

        // Sort order
        Label sortLabel = new Label("Sort by:");
        sortLabel.getStyleClass().add("filter-label");

        sortOrder = new ComboBox<>();
        sortOrder.getItems().addAll("Featured", "Popularity", "Last Updated", "Name", "Author", "Total Downloads");
        sortOrder.setValue("Featured");
        sortOrder.getStyleClass().add("category-filter"); // Add this line
        sortOrder.setOnAction(e -> {
            if (sortOrder.getValue() != null) {
                currentSortOrder = sortOrder.getValue();
                currentPage = 0;
                searchMods(currentSearchTerm);
            }
        });
        sortOrder.getStyleClass().add("version-combo-box");

        // Create left and right sides
        HBox leftControls = new HBox(10, searchField, searchButton);
        leftControls.setAlignment(Pos.CENTER_LEFT);

        HBox rightControls = new HBox(10, categoryLabel, categoryFilter, sortLabel, sortOrder);
        rightControls.setAlignment(Pos.CENTER_RIGHT);

        // Main container
        HBox searchControls = new HBox();
        searchControls.setPadding(new Insets(10, 0, 10, 0));
        searchControls.getChildren().addAll(leftControls, new Region(), rightControls);
        HBox.setHgrow(leftControls, Priority.ALWAYS);
        HBox.setHgrow(rightControls, Priority.NEVER);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);
        searchControls.getChildren().add(1, spacer);

        return searchControls;
    }

    /**
     * Creates a page for the pagination control
     */
    private Node createPage(int pageIndex) {
        if (pageIndex != currentPage) {
            currentPage = pageIndex;
            searchMods(currentSearchTerm);
        }
        return new VBox(); // Empty VBox, content is actually in modListContainer
    }

    /**
     * Loads categories from the CurseForge API
     */
    private void loadCategories() {
        Task<Void> task = new Task<>() {
            @Override
            protected Void call() throws Exception {
                try {
                    HttpRequest request = HttpRequest.newBuilder()
                            .uri(URI.create(CURSEFORGE_API_URL + "/categories?gameId=432"))
                            .header("x-api-key", API_KEY)
                            .header("Accept", "application/json")
                            .GET()
                            .build();

                    HttpResponse<String> response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
                    if (response.statusCode() == 200) {
                        JsonNode rootNode = mapper.readTree(response.body());
                        JsonNode dataNode = rootNode.get("data");

                        if (dataNode != null && dataNode.isArray()) {
                            categories = mapper.readValue(
                                    dataNode.toString(),
                                    new TypeReference<List<Category>>() {}
                            );

                            Platform.runLater(() -> {
                                // Add all mod categories
                                List<String> categoryNames = new ArrayList<>();
                                categoryNames.add("All");

                                // Only add categories that are class 6 (mods)
                                categories.stream()
                                        .filter(c -> c.classId == 6)
                                        .map(c -> c.name)
                                        .sorted()
                                        .forEach(categoryNames::add);

                                categoryFilter.getItems().setAll(categoryNames);
                            });
                        }
                    }
                } catch (Exception e) {
                    e.printStackTrace();
                }
                return null;
            }
        };

        new Thread(task).start();
    }

    /**
     * Searches for mods using the CurseForge API
     */
    private void searchMods(String searchTerm) {
        currentSearchTerm = searchTerm;
        showLoading(true);

        Task<Void> task = new Task<>() {
            @Override
            protected Void call() throws Exception {
                try {
                    // Build search parameters
                    StringBuilder urlBuilder = new StringBuilder(CURSEFORGE_API_URL + "/mods/search?gameId=432");

                    // Add search term if provided
                    if (!searchTerm.isEmpty()) {
                        urlBuilder.append("&searchFilter=").append(
                                URLEncoder.encode(searchTerm, StandardCharsets.UTF_8));
                    }

                    // Add category filter if not "All"
                    if (!currentCategory.equals("All")) {
                        Optional<Category> selectedCategory = categories.stream()
                                .filter(c -> c.name.equals(currentCategory))
                                .findFirst();

                        selectedCategory.ifPresent(category ->
                                urlBuilder.append("&categoryId=").append(category.id));
                    }

                    // Add sorting
                    String sortParam = "Featured";
                    switch (currentSortOrder) {
                        case "Popularity" -> sortParam = "Popularity";
                        case "Last Updated" -> sortParam = "LastUpdated";
                        case "Name" -> sortParam = "Name";
                        case "Author" -> sortParam = "Author";
                        case "Total Downloads" -> sortParam = "TotalDownloads";
                    }
                    urlBuilder.append("&sortField=").append(sortParam);

                    // Add pagination
                    urlBuilder.append("&index=").append(currentPage * pageSize);
                    urlBuilder.append("&pageSize=").append(pageSize);

                    // Send request
                    HttpRequest request = HttpRequest.newBuilder()
                            .uri(URI.create(urlBuilder.toString()))
                            .header("x-api-key", API_KEY)
                            .header("Accept", "application/json")
                            .GET()
                            .build();

                    HttpResponse<String> response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
                    if (response.statusCode() == 200) {
                        JsonNode rootNode = mapper.readTree(response.body());
                        JsonNode dataNode = rootNode.get("data");
                        JsonNode paginationNode = rootNode.get("pagination");

                        if (dataNode != null && dataNode.isArray()) {
                            List<ModInfo> mods = mapper.readValue(
                                    dataNode.toString(),
                                    new TypeReference<List<ModInfo>>() {}
                            );

                            // Get pagination info
                            if (paginationNode != null) {
                                totalResults = paginationNode.get("totalCount").asInt();
                            }

                            // Update UI on JavaFX thread
                            Platform.runLater(() -> {
                                updateModListDisplay(mods);
                                updatePagination();
                                showLoading(false);
                                statusLabel.setText(mods.size() + " mods found" +
                                        (searchTerm.isEmpty() ? "" : " for '" + searchTerm + "'"));
                            });
                            return null;
                        }
                    }

                    // Handle errors
                    Platform.runLater(() -> {
                        modListContainer.getChildren().clear();
                        Label errorLabel = new Label("Error loading mods. Please try again.");
                        errorLabel.getStyleClass().add("error-message");
                        modListContainer.getChildren().add(errorLabel);
                        showLoading(false);
                        statusLabel.setText("Error occurred while searching");
                    });

                } catch (Exception e) {
                    e.printStackTrace();
                    Platform.runLater(() -> {
                        modListContainer.getChildren().clear();
                        Label errorLabel = new Label("Error: " + e.getMessage());
                        errorLabel.getStyleClass().add("error-message");
                        modListContainer.getChildren().add(errorLabel);
                        showLoading(false);
                        statusLabel.setText("Error occurred while searching");
                    });
                }
                return null;
            }
        };

        new Thread(task).start();
    }

    /**
     * Updates the pagination control based on search results
     */
    private void updatePagination() {
        int pageCount = (int) Math.ceil((double) totalResults / pageSize);
        pagination.setPageCount(Math.max(1, pageCount));
        pagination.setCurrentPageIndex(currentPage);
    }

    /**
     * Updates the mod list display with search results
     */
    private void updateModListDisplay(List<ModInfo> mods) {
        modListContainer.getChildren().clear();

        if (mods.isEmpty()) {
            Label noResultsLabel = new Label("No mods found. Try adjusting your search.");
            noResultsLabel.getStyleClass().add("no-results-message");
            modListContainer.getChildren().add(noResultsLabel);
            return;
        }

        for (ModInfo mod : mods) {
            ModCard modCard = new ModCard(mod);
            modListContainer.getChildren().add(modCard);
        }
    }

    /**
     * Shows or hides the loading indicator
     */
    private void showLoading(boolean show) {
        loadingIndicator.setVisible(show);
    }

    /**
     * Fetches detailed information about a specific mod
     */
    private void fetchModDetails(int modId, ModCard card) {
        showLoading(true);

        Task<Void> task = new Task<>() {
            @Override
            protected Void call() throws Exception {
                try {
                    HttpRequest request = HttpRequest.newBuilder()
                            .uri(URI.create(CURSEFORGE_API_URL + "/mods/" + modId))
                            .header("x-api-key", API_KEY)
                            .header("Accept", "application/json")
                            .GET()
                            .build();

                    HttpResponse<String> response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
                    if (response.statusCode() == 200) {
                        JsonNode rootNode = mapper.readTree(response.body());
                        JsonNode dataNode = rootNode.get("data");

                        if (dataNode != null) {
                            ModDetail modDetail = mapper.readValue(
                                    dataNode.toString(),
                                    ModDetail.class
                            );

                            Platform.runLater(() -> {
                                showModDetailDialog(modDetail, card);
                                showLoading(false);
                            });
                            return null;
                        }
                    }

                    Platform.runLater(() -> {
                        Alert alert = new Alert(Alert.AlertType.ERROR,
                                "Failed to load mod details. Please try again.");
                        alert.showAndWait();
                        showLoading(false);
                    });

                } catch (Exception e) {
                    e.printStackTrace();
                    Platform.runLater(() -> {
                        Alert alert = new Alert(Alert.AlertType.ERROR,
                                "Error: " + e.getMessage());
                        alert.showAndWait();
                        showLoading(false);
                    });
                }
                return null;
            }
        };

        new Thread(task).start();
    }

    /**
     * Shows the mod detail dialog
     */
    private void showModDetailDialog(ModDetail modDetail, ModCard card) {
        currentModDetail = modDetail;

        Stage detailStage = new Stage();
        detailStage.initOwner(browserStage);
        detailStage.initModality(Modality.NONE);
        detailStage.setTitle(modDetail.name);
        detailStage.setMinWidth(800);
        detailStage.setMinHeight(600);

        BorderPane detailRoot = new BorderPane();
        detailRoot.setPadding(new Insets(20));
        detailRoot.getStyleClass().add("mod-detail-container");

        // Header with mod name and basic info
        VBox header = createModDetailHeader(modDetail);
        detailRoot.setTop(header);

        // Center content with description and screenshots
        TabPane tabPane = new TabPane();
        tabPane.setTabClosingPolicy(TabPane.TabClosingPolicy.UNAVAILABLE);
        tabPane.getStyleClass().add("mod-detail-tabs");

        // Description tab
        Tab descriptionTab = new Tab("Description");
        descriptionTab.setContent(createDescriptionTab(modDetail));
        descriptionTab.getStyleClass().add("mod-tab");

        // Screenshots tab
        Tab screenshotsTab = new Tab("Screenshots");
        screenshotsTab.setContent(createScreenshotsTab(modDetail));
        screenshotsTab.getStyleClass().add("mod-tab");

        // Files tab
        Tab filesTab = new Tab("Files");
        filesTab.setContent(createFilesTab(modDetail));
        filesTab.getStyleClass().add("mod-tab");

        tabPane.getTabs().addAll(descriptionTab, screenshotsTab, filesTab);
        detailRoot.setCenter(tabPane);

        // Bottom buttons
        HBox buttonBar = createModDetailButtonBar(modDetail, card, detailStage);
        detailRoot.setBottom(buttonBar);

        Scene detailScene = new Scene(detailRoot, 800, 600);
        detailScene.getStylesheets().add(Objects.requireNonNull(
                getClass().getResource("/stylesheet/mod_details.css")).toExternalForm());

        detailStage.setScene(detailScene);

        // Add icon
        detailStage.getIcons().add(
                new Image(Objects.requireNonNull(
                        getClass().getResourceAsStream("/images/png/logo_tmod_bg(0)_zoomed.png")
                ))
        );

        // Show the dialog
        detailStage.show();

        Platform.runLater(() -> {
            TitleBarCustomizer.applyTheme(detailStage,
                    "#3a3a38",
                    "#f9f8f4",
                    "#5a5a58"
            );
        });
    }

    /**
     * Creates the mod detail header section
     */
    private VBox createModDetailHeader(ModDetail modDetail) {
        // Mod title
        Label titleLabel = new Label(modDetail.name);
        titleLabel.getStyleClass().add("mod-detail-title");

        // Author and basic stats
        HBox statsBox = new HBox(15);
        statsBox.setAlignment(Pos.CENTER_LEFT);

        // Authors
        String authors = modDetail.authors.stream()
                .map(a -> a.name)
                .collect(Collectors.joining(", "));

        Label authorLabel = new Label("By: " + authors);
        authorLabel.getStyleClass().add("mod-detail-author");

        // Downloads
        Label downloadsLabel = new Label(formatNumber(modDetail.downloadCount) + " Downloads");
        downloadsLabel.getStyleClass().add("mod-detail-downloads");

        // Last update
        LocalDateTime updateTime = LocalDateTime.parse(
                modDetail.dateModified.replace("Z", ""),
                DateTimeFormatter.ISO_LOCAL_DATE_TIME);
        String formattedDate = updateTime.format(DateTimeFormatter.ofPattern("MMM d, yyyy"));
        Label updatedLabel = new Label("Updated: " + formattedDate);
        updatedLabel.getStyleClass().add("mod-detail-updated");

        statsBox.getChildren().addAll(authorLabel, downloadsLabel, updatedLabel);

        // Categories
        FlowPane categoriesPane = new FlowPane(10, 10);
        categoriesPane.setAlignment(Pos.CENTER_LEFT);

        for (Category category : modDetail.categories) {
            Label categoryChip = new Label(category.name);
            categoryChip.getStyleClass().add("category-chip");
            categoriesPane.getChildren().add(categoryChip);
        }

        // Summary
        Label summaryLabel = new Label(modDetail.summary);
        summaryLabel.getStyleClass().add("mod-detail-summary");
        summaryLabel.setWrapText(true);

        VBox header = new VBox(10, titleLabel, statsBox, categoriesPane, summaryLabel);
        header.setPadding(new Insets(0, 0, 15, 0));
        header.setAlignment(Pos.TOP_LEFT);

        return header;
    }

    /**
     * Creates the description tab content
     */
    private ScrollPane createDescriptionTab(ModDetail modDetail) {
        VBox content = new VBox(15);
        content.setPadding(new Insets(15));

        // Convert HTML description to formatted text (basic implementation)
        String description = modDetail.description != null ? modDetail.description : "No description available";
        // Replace some basic HTML tags
        description = description.replaceAll("<br/?", "\n")
                .replaceAll("<p>", "\n")
                .replaceAll("</p>", "\n")
                .replaceAll("<[^>]*>", ""); // Remove other HTML tags

        Text descriptionText = new Text(description);
        descriptionText.getStyleClass().add("mod-description-text");
        descriptionText.setWrappingWidth(760);

        content.getChildren().add(descriptionText);

        ScrollPane scrollPane = new ScrollPane(content);
        scrollPane.setFitToWidth(true);
        scrollPane.getStyleClass().add("description-scroll");

        return scrollPane;
    }

    /**
     * Creates the screenshots tab content
     */
    private ScrollPane createScreenshotsTab(ModDetail modDetail) {
        VBox content = new VBox(20);
        content.setPadding(new Insets(15));
        content.setAlignment(Pos.TOP_CENTER);

        if (modDetail.screenshots == null || modDetail.screenshots.isEmpty()) {
            Label noScreenshotsLabel = new Label("No screenshots available");
            noScreenshotsLabel.getStyleClass().add("no-screenshots-message");
            content.getChildren().add(noScreenshotsLabel);
        } else {
            for (Screenshot screenshot : modDetail.screenshots) {
                VBox screenshotBox = new VBox(5);
                screenshotBox.setAlignment(Pos.CENTER);

                // Create image view
                ImageView imageView = new ImageView();
                imageView.setPreserveRatio(true);
                imageView.setFitWidth(700);

                // Load image asynchronously
                CompletableFuture.runAsync(() -> {
                    try {
                        Image image = new Image(screenshot.url);
                        Platform.runLater(() -> {
                            imageView.setImage(image);
                        });
                    } catch (Exception e) {
                        Platform.runLater(() -> {
                            Label errorLabel = new Label("Failed to load image");
                            errorLabel.getStyleClass().add("error-message");
                            screenshotBox.getChildren().add(errorLabel);
                        });
                    }
                });

                // Caption
                Label captionLabel = new Label(screenshot.title);
                captionLabel.getStyleClass().add("screenshot-caption");
                captionLabel.setWrapText(true);
                captionLabel.setTextAlignment(TextAlignment.CENTER);

                screenshotBox.getChildren().addAll(imageView, captionLabel);
                content.getChildren().add(screenshotBox);
            }
        }

        ScrollPane scrollPane = new ScrollPane(content);
        scrollPane.setFitToWidth(true);
        scrollPane.getStyleClass().add("screenshots-scroll");

        return scrollPane;
    }

    /**
     * Creates the files tab content with available downloads
     */
    private ScrollPane createFilesTab(ModDetail modDetail) {
        VBox content = new VBox(15);
        content.setPadding(new Insets(15));

        if (modDetail.latestFiles == null || modDetail.latestFiles.isEmpty()) {
            Label noFilesLabel = new Label("No files available");
            noFilesLabel.getStyleClass().add("no-files-message");
            content.getChildren().add(noFilesLabel);
        } else {
            // Sort files by date
            List<ModFile> sortedFiles = new ArrayList<>(modDetail.latestFiles);
            sortedFiles.sort((a, b) -> b.fileDate.compareTo(a.fileDate));

            // Create a table view
            TableView<ModFile> filesTable = new TableView<>();
            filesTable.getStyleClass().add("files-table");

            // File name column
            TableColumn<ModFile, String> nameColumn = new TableColumn<>("File Name");
            nameColumn.setCellValueFactory(data -> new SimpleStringProperty(data.getValue().fileName));
            nameColumn.setPrefWidth(300);

            // Game version column
            TableColumn<ModFile, String> versionColumn = new TableColumn<>("Game Version");
            versionColumn.setCellValueFactory(data -> {
                String versions = String.join(", ", data.getValue().gameVersions);
                return new SimpleStringProperty(versions);
            });
            versionColumn.setPrefWidth(150);

            // Release type column
            TableColumn<ModFile, String> releaseColumn = new TableColumn<>("Release Type");
            releaseColumn.setCellValueFactory(data -> {
                String releaseType;
                switch (data.getValue().releaseType) {
                    case 1 -> releaseType = "Release";
                    case 2 -> releaseType = "Beta";
                    case 3 -> releaseType = "Alpha";
                    default -> releaseType = "Unknown";
                }
                return new SimpleStringProperty(releaseType);
            });
            releaseColumn.setPrefWidth(100);

            // Date column
            TableColumn<ModFile, String> dateColumn = new TableColumn<>("Date");
            dateColumn.setCellValueFactory(data -> {
                LocalDateTime fileDate = LocalDateTime.parse(
                        data.getValue().fileDate.replace("Z", ""),
                        DateTimeFormatter.ISO_LOCAL_DATE_TIME);
                String formattedDate = fileDate.format(DateTimeFormatter.ofPattern("MMM d, yyyy"));
                return new SimpleStringProperty(formattedDate);
            });
            dateColumn.setPrefWidth(120);

            // Download column
            TableColumn<ModFile, String> downloadColumn = new TableColumn<>("Download");
            downloadColumn.setCellValueFactory(data -> new SimpleStringProperty(""));
            downloadColumn.setCellFactory(col -> new TableCell<ModFile, String>() {
                private Button downloadBtn;

                @Override
                protected void updateItem(String item, boolean empty) {
                    super.updateItem(item, empty);
                    if (empty || getTableRow() == null) {
                        setGraphic(null);
                    } else {
                        ModFile file = getTableRow().getItem();
                        if (file != null) {
                            if (downloadBtn == null) {
                                downloadBtn = TModGui.createStyledButton("Download", "dw-button", "DOWNLOAD");
                            }
                            downloadBtn.setOnAction(e -> {
                                try {
                                    installMod(modDetail, file);
                                } catch (Exception ex) {
                                    System.err.println("Error installing mod: " + ex.getMessage());
                                }
                            });
                            setGraphic(downloadBtn);
                        } else {
                            setGraphic(null);
                        }
                    }
                }
            });
            downloadColumn.setPrefWidth(100);

            filesTable.getColumns().addAll(nameColumn, versionColumn, releaseColumn, dateColumn, downloadColumn);
            filesTable.setItems(FXCollections.observableArrayList(sortedFiles));

            content.getChildren().add(filesTable);
            VBox.setVgrow(filesTable, Priority.ALWAYS);
        }

        ScrollPane scrollPane = new ScrollPane(content);
        scrollPane.setFitToWidth(true);
        scrollPane.getStyleClass().add("files-scroll");

        return scrollPane;
    }

    /**
     * Creates the button bar for the mod detail dialog
     */
    private HBox createModDetailButtonBar(ModDetail modDetail, ModCard card, Stage detailStage) {
        HBox buttonBar = new HBox(10);
        buttonBar.setPadding(new Insets(15, 0, 0, 0));
        buttonBar.setAlignment(Pos.CENTER_RIGHT);

        // Install latest button
        installButton = TModGui.createStyledButton("Install Latest Version", "install-button", "DOWNLOAD");
        installButton.setOnAction(e -> {
            // Find the latest release file
            Optional<ModFile> latestFile = modDetail.latestFiles.stream()
                    .filter(f -> f.releaseType == 1) // Release type
                    .findFirst();

            if (latestFile.isPresent()) {
                installMod(modDetail, latestFile.get());
            } else if (!modDetail.latestFiles.isEmpty()) {
                // If no release, use the first file
                installMod(modDetail, modDetail.latestFiles.get(0));
            } else {
                Alert alert = new Alert(Alert.AlertType.ERROR,
                        "No files available for download.");
                alert.showAndWait();
            }
        });

        // Visit website button
        Button websiteButton = TModGui.createStyledButton("Visit Website", "website-button", "LINK");
        websiteButton.setOnAction(e -> {
            String url = modDetail.links.websiteUrl;
            if (url != null && !url.isEmpty()) {
                try {
                    java.awt.Desktop.getDesktop().browse(new URI(url));
                } catch (Exception ex) {
                    Alert alert = new Alert(Alert.AlertType.ERROR,
                            "Failed to open website: " + ex.getMessage());
                    alert.showAndWait();
                }
            }
        });

        // Close button
        Button closeButton = TModGui.createStyledButton("Close", "close-button", "TIMES");
        closeButton.setOnAction(e -> detailStage.close());

        // Add buttons
        buttonBar.getChildren().addAll(websiteButton, installButton, closeButton);

        return buttonBar;
    }

    /**
     * Installs a mod using the CliBridge
     */
    private void installMod(ModDetail modDetail, ModFile file) {
        Stage progressStage = new Stage();
        progressStage.initOwner(browserStage);
        progressStage.initModality(Modality.APPLICATION_MODAL);
        progressStage.setTitle("Installing Mod");
        progressStage.setResizable(false);

        VBox progressBox = new VBox(15);
        progressBox.setPadding(new Insets(20));
        progressBox.setAlignment(Pos.CENTER);
        progressBox.setMinWidth(400);
        progressBox.setMinHeight(200);

        Label titleLabel = new Label("Installing " + modDetail.name);
        titleLabel.getStyleClass().add("install-title");

        ProgressIndicator progressIndicator = new ProgressIndicator();
        progressIndicator.setPrefSize(50, 50);

        Label statusLabel = new Label("Starting installation...");
        statusLabel.getStyleClass().add("install-status");

        progressBox.getChildren().addAll(titleLabel, progressIndicator, statusLabel);

        Scene progressScene = new Scene(progressBox);
        progressScene.getStylesheets().add(Objects.requireNonNull(
                getClass().getResource("/stylesheet/style.css")).toExternalForm());

        progressStage.setScene(progressScene);
        progressStage.show();

        Platform.runLater(() -> {
            TitleBarCustomizer.applyTheme(progressStage,
                    "#3a3a38",
                    "#f9f8f4",
                    "#5a5a58"
            );
        });

        // Execute the installation in a background thread
        Task<String> installTask = new Task<>() {
            @Override
            protected String call() throws Exception {
                // Use the URL from the mod file
                String url = file.downloadUrl;
                if (url == null || url.isEmpty()) {
                    // Fallback to the project URL
                    url = modDetail.links.websiteUrl;
                }

                updateMessage("Adding mod from URL: " + url);
                return CliBridge.run("add", "--url", url);
            }
        };

        installTask.messageProperty().addListener((obs, oldMsg, newMsg) -> {
            statusLabel.setText(newMsg);
        });

        installTask.setOnSucceeded(e -> {
            String result = installTask.getValue();
            Platform.runLater(() -> {
                progressStage.close();

                if (result.toLowerCase().contains("error") || result.toLowerCase().contains("failed")) {
                    Alert alert = new Alert(Alert.AlertType.ERROR);
                    alert.setTitle("Installation Failed");
                    alert.setHeaderText("Failed to install " + modDetail.name);
                    alert.setContentText(result);
                    alert.showAndWait();
                } else {
                    Alert alert = new Alert(Alert.AlertType.INFORMATION);
                    alert.setTitle("Installation Complete");
                    alert.setHeaderText(modDetail.name + " installed successfully");
                    alert.setContentText("The mod and its dependencies have been added to your modpack.");
                    alert.showAndWait();
                }
            });
        });

        installTask.setOnFailed(e -> {
            Platform.runLater(() -> {
                progressStage.close();
                Alert alert = new Alert(Alert.AlertType.ERROR);
                alert.setTitle("Installation Failed");
                alert.setHeaderText("Failed to install " + modDetail.name);
                alert.setContentText(installTask.getException().getMessage());
                alert.showAndWait();
            });
        });

        new Thread(installTask).start();
    }

    /**
     * Formats a number with K, M suffixes for display
     */
    private String formatNumber(int number) {
        if (number < 1000) {
            return String.valueOf(number);
        } else if (number < 1000000) {
            return String.format("%.1fK", number / 1000.0);
        } else {
            return String.format("%.1fM", number / 1000000.0);
        }
    }

    /**
     * ModCard class for displaying a mod in the search results
     */
    private class ModCard extends HBox {
        private final ModInfo mod;

        public ModCard(ModInfo mod) {
            this.mod = mod;
            this.setSpacing(15);
            this.setPadding(new Insets(10));
            this.getStyleClass().add("mod-card");
            this.setOnMouseClicked(e -> fetchModDetails(mod.id, this));

            // Logo
            ImageView logoView = new ImageView();
            logoView.setFitWidth(64);
            logoView.setFitHeight(64);
            logoView.setPreserveRatio(true);

            // Placeholder until image loads
            logoView.setImage(new Image(
                    Objects.requireNonNull(getClass().getResourceAsStream("/images/png/logo_tmod.png"))
            ));

            // Load actual logo asynchronously if available
            if (mod.logo != null && mod.logo.url != null && !mod.logo.url.isEmpty()) {
                CompletableFuture.runAsync(() -> {
                    try {
                        Image logo = new Image(mod.logo.url, true);
                        Platform.runLater(() -> logoView.setImage(logo));
                    } catch (Exception e) {
                        // Keep placeholder on error
                    }
                });
            }

            // Content
            VBox contentBox = new VBox(5);
            HBox.setHgrow(contentBox, Priority.ALWAYS);

            // Title and author
            Label titleLabel = new Label(mod.name);
            titleLabel.getStyleClass().add("mod-card-title");

            String authors = mod.authors.stream()
                    .map(a -> a.name)
                    .collect(Collectors.joining(", "));
            Label authorLabel = new Label("By " + authors);
            authorLabel.getStyleClass().add("mod-card-author");

            // Summary
            Label summaryLabel = new Label(mod.summary);
            summaryLabel.getStyleClass().add("mod-card-summary");
            summaryLabel.setWrapText(true);
            summaryLabel.setMaxHeight(40);

            // Stats
            HBox statsBox = new HBox(15);
            statsBox.setAlignment(Pos.CENTER_LEFT);

            Label downloadsLabel = new Label(formatNumber(mod.downloadCount) + " Downloads");
            downloadsLabel.getStyleClass().add("mod-card-downloads");

            // Create category pills
            FlowPane categoryPane = new FlowPane(5, 5);
            categoryPane.setAlignment(Pos.CENTER_LEFT);

            // Show up to 3 categories
            int categoryLimit = Math.min(mod.categories.size(), 3);
            for (int i = 0; i < categoryLimit; i++) {
                Label categoryLabel = new Label(mod.categories.get(i).name);
                categoryLabel.getStyleClass().add("category-pill");
                categoryPane.getChildren().add(categoryLabel);
            }

            statsBox.getChildren().addAll(downloadsLabel, categoryPane);

            contentBox.getChildren().addAll(titleLabel, authorLabel, summaryLabel, statsBox);

            // Install button
            Button installButton = TModGui.createStyledButton("", "mod-card-install", "DOWNLOAD");
            installButton.setMinHeight(40);
            installButton.setOnAction(e -> {
                e.consume(); // Prevent card click
                fetchModDetails(mod.id, this);
            });

            this.getChildren().addAll(logoView, contentBox, installButton);
        }
    }

    /**
     * ModInfo class with basic mod information for listings
     */
    private static class ModInfo {
        public int id;
        public String name;
        public String slug;
        public Links links;
        public String summary;
        public int downloadCount;
        public List<Author> authors = new ArrayList<>();
        public Logo logo;
        public List<Category> categories = new ArrayList<>();
    }

    /**
     * ModDetail class with detailed mod information
     */
    private static class ModDetail {
        public int id;
        public String name;
        public String slug;
        public Links links;
        public String summary;
        public String description;
        public int downloadCount;
        public List<Author> authors = new ArrayList<>();
        public Logo logo;
        public List<Screenshot> screenshots = new ArrayList<>();
        public List<ModFile> latestFiles = new ArrayList<>();
        public List<Category> categories = new ArrayList<>();
        public String dateCreated;
        public String dateModified;
        public String dateReleased;
    }

    /**
     * Links class for mod URLs
     */
    private static class Links {
        public String websiteUrl;
        public String wikiUrl;
        public String issuesUrl;
        public String sourceUrl;
    }

    /**
     * Author class for mod authors
     */
    private static class Author {
        public int id;
        public String name;
        public String url;
    }

    /**
     * Logo class for mod logos
     */
    private static class Logo {
        public int id;
        public String url;
        public String thumbnailUrl;
    }

    /**
     * Screenshot class for mod screenshots
     */
    private static class Screenshot {
        public int id;
        public String title;
        public String description;
        public String url;
        public String thumbnailUrl;
    }

    /**
     * Category class for mod categories
     */
    private static class Category {
        public int id;
        public int gameId;
        public String name;
        public String slug;
        public String url;
        public int classId;
    }

    /**
     * ModFile class for mod files
     */
    private static class ModFile {
        public int id;
        public String fileName;
        public String displayName;
        public String fileDate;
        public int fileLength;
        public int releaseType;
        public int fileStatus;
        public String downloadUrl;
        public List<String> gameVersions = new ArrayList<>();
    }
}
