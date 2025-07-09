package com.tmod.gui;



import javafx.scene.text.Font;
import javafx.scene.text.Text;

import java.util.HashMap;
import java.util.Map;
import java.util.Objects;
import java.util.logging.Logger;

/**
 * Utility class for generating FontAwesome icons as {@link Text} nodes in JavaFX applications.
 * <p>
 * This class loads FontAwesome TTF font and provides a map of icon names to their Unicode values.
 * It allows the creation of styled icons with customizable size and CSS style classes.
 * </p>
 * <p>
 * Note: This class assumes that the FontAwesome font file is located at <code>/fonts/fa-solid-900.ttf</code>
 * in the application's resources.
 * </p>
 */
public class FontAwesomeIcon {
    private static final Logger logger = Logger.getLogger(FontAwesomeIcon.class.getName());

    /**
     * A mapping between icon name keys and their corresponding FontAwesome Unicode characters.
     */
    private static final Map<String, String> ICON_MAP = new HashMap<>();

    /**
     * A cache of loaded fonts indexed by size, used to avoid reloading fonts for each icon size.
     */
    private static final Map<Double, Font> FONT_CACHE = new HashMap<>();

    /**
     * Loaded FontAwesome font instance.
     */
    private static Font fontAwesome;

    // Static initialization block to populate the ICON_MAP with commonly used icon names
    static {
        ICON_MAP.put("COG", "\uf013");              // General settings
        ICON_MAP.put("PALETTE", "\uf53f");          // Appearance
        ICON_MAP.put("INFO_CIRCLE", "\uf05a");      // About
        ICON_MAP.put("QUESTION_CIRCLE", "\uf059");  // Default fallback
        ICON_MAP.put("CUBE", "\uf1b2");             // Download
        ICON_MAP.put("SEARCH", "\uf002");
        ICON_MAP.put("UPLOAD", "\uf093");
        ICON_MAP.put("TIMES", "\uf00d");
        ICON_MAP.put("FOLDER", "\uf07b");
        ICON_MAP.put("IMAGE", "\uf03e");
        ICON_MAP.put("FILM", "\uf008");
        ICON_MAP.put("CODE", "\uf121");
        ICON_MAP.put("ARCHIVE", "\uf187");

        ICON_MAP.put("CLOCK", "\uf017");
        ICON_MAP.put("SIGN_IN_ALT", "\uf2f6");
        ICON_MAP.put("USER_CLOCK", "\uf4fc");

        ICON_MAP.put("STAR", "\uf005");
        ICON_MAP.put("SHARE_ALT", "\uf1e0");
        ICON_MAP.put("TH", "\uf00a");
        ICON_MAP.put("LIST", "\uf03a");
        ICON_MAP.put("FILE_CODE", "\uf1c9");
        ICON_MAP.put("FILE", "\uf15b");
        ICON_MAP.put("DOWNLOAD", "\uf019");
        ICON_MAP.put("TRASH_ALT", "\uf2ed");
        ICON_MAP.put("SHIELD_ALT", "\uf3ed");
        ICON_MAP.put("LINK", "\uf0c1");
        ICON_MAP.put("PROFILE", "\uf007");
        ICON_MAP.put("SECURITY", "\uf3ed");
        ICON_MAP.put("BILLING", "\uf09d");
        ICON_MAP.put("ACTIVITY", "\uf080");
        ICON_MAP.put("NOTIFICATION", "\uf0f3");
        ICON_MAP.put("LOGOUT", "\uf08b");
        ICON_MAP.put("REFRESH", "\uf021");

        // UPLOAD
        ICON_MAP.put("CLOUD_UPLOAD_ALT", "\uf382");
        ICON_MAP.put("FILE_PDF", "\uf1c1");
        ICON_MAP.put("FILE_WORD", "\uf1c2");
        ICON_MAP.put("FILE_EXCEL", "\uf1c3");
        ICON_MAP.put("FILE_POWERPOINT", "\uf1c4");
        ICON_MAP.put("FILE_IMAGE", "\uf1c5");
        ICON_MAP.put("FILE_AUDIO", "\uf1c7");
        ICON_MAP.put("FILE_VIDEO", "\uf1c8");
        ICON_MAP.put("FILE_ARCHIVE", "\uf1c6");
        ICON_MAP.put("FILE_ALT", "\uf15b");
        ICON_MAP.put("CLOSE", "\uf00d");
        ICON_MAP.put("FILE_UPLOAD", "\uf574");
        ICON_MAP.put("ARROW_LEFT", "\u2190");
        ICON_MAP.put("TRASH", "\uf1f8");
        ICON_MAP.put("FILE_LIST", "\uf022");

    }

    /**
     * Creates a FontAwesome icon with the default size (16px) and specified style class.
     *
     * @param iconName   the name of the icon (e.g., "SEARCH", "UPLOAD")
     * @param styleClass the CSS class to apply to the icon
     * @return a {@link Text} node representing the FontAwesome icon
     */
    public static Text createIcon(String iconName, String styleClass) {
        return createIcon(iconName, styleClass, 16);
    }

    /**
     * Creates a FontAwesome icon with a specified size and style class.
     * If the FontAwesome font is not yet loaded, this method attempts to load it from resources.
     *
     * @param iconName   the name of the icon to render (must exist in ICON_MAP)
     * @param styleClass the CSS class to apply to the icon
     * @param size       the font size in pixels
     * @return a styled {@link Text} node representing the requested icon
     */
    public static Text createIcon(String iconName, String styleClass, double size) {
        if (fontAwesome == null) {
            try {
                fontAwesome = Font.loadFont(
                        Objects.requireNonNull(FontAwesomeIcon.class.getResourceAsStream("/fonts/fa-solid-900.ttf")),
                        16);

                if (fontAwesome == null) {
                    logger.severe("Failed to load FontAwesome font!");
                }
            } catch (Exception e) {
                e.printStackTrace();
            }
        }

        Text icon = new Text(ICON_MAP.getOrDefault(iconName, "\uf128")); // \uf128 = default (info-circle)

        icon.setStyle(String.format("-fx-font-family: '%s'; -fx-font-size: %.1fpx;",
                fontAwesome.getFamily(), size));

        icon.getStyleClass().add(styleClass);

        return icon;
    }
}
