package com.tmod.gui;

import com.sun.jna.Native;
import com.sun.jna.platform.win32.User32;
import com.sun.jna.platform.win32.WinDef.HWND;
import javafx.stage.Stage;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * TitleBarCustomizer is a utility class that provides methods to customize the title bar appearance
 * of a JavaFX stage on Windows operating systems using the DWM (Desktop Window Manager) API.
 * <p>
 * This class allows setting the title bar caption color, text color, and border color, enabling a
 * visually appealing and professional user interface. It is designed for Windows 10 (version 1809 and later).
 * </p>
 */
public class TitleBarCustomizer {
    private static final Logger logger = Logger.getLogger(TitleBarCustomizer.class.getName());
    // Interface for DWM API
    public interface Dwmapi extends com.sun.jna.Library {
        Dwmapi INSTANCE = Native.load("dwmapi", Dwmapi.class);

        int DWMWA_USE_IMMERSIVE_DARK_MODE = 20; // Enable dark mode
        int DWMWA_CAPTION_COLOR = 35; // Custom title bar color
        int DWMWA_TEXT_COLOR = 36; // Title bar text color
        int DWMWA_BORDER_COLOR = 34; // Window border color

        int DwmSetWindowAttribute(HWND hwnd, int dwAttribute, int[] pvAttribute, int cbAttribute);
    }

    /**
     * Sets the title bar colors for caption, text, and border.
     *
     * @param stage       The stage whose title bar is to be customized.
     * @param rCaption    Red component of the caption color (0-255).
     * @param gCaption    Green component of the caption color (0-255).
     * @param bCaption    Blue component of the caption color (0-255).
     * @param rText       Red component of the text color (0-255).
     * @param gText       Green component of the text color (0-255).
     * @param bText       Blue component of the text color (0-255).
     * @param rBorder     Red component of the border color (0-255).
     * @param gBorder     Green component of the border color (0-255).
     * @param bBorder     Blue component of the border color (0-255).
     */
    public static void setTitleBarColors(Stage stage, int rCaption, int gCaption, int bCaption,
                                         int rText, int gText, int bText,
                                         int rBorder, int gBorder, int bBorder) {
        try {
            // Get the window handle using the stage title
            String windowTitle = stage.getTitle();
            HWND hwnd = User32.INSTANCE.FindWindow(null, windowTitle);
            if (hwnd == null) {
                logger.severe("Window not found!");
                return;
            }

            // Enable immersive dark mode
            int[] darkMode = new int[]{1};
            Dwmapi.INSTANCE.DwmSetWindowAttribute(
                    hwnd,
                    Dwmapi.DWMWA_USE_IMMERSIVE_DARK_MODE,
                    darkMode,
                    darkMode.length * 4
            );

            // Set caption color
            int captionColor = (rCaption & 0xFF) | ((gCaption & 0xFF) << 8) | ((bCaption & 0xFF) << 16);
            int[] captionColorArray = new int[]{captionColor};
            Dwmapi.INSTANCE.DwmSetWindowAttribute(
                    hwnd,
                    Dwmapi.DWMWA_CAPTION_COLOR,
                    captionColorArray,
                    captionColorArray.length * 4
            );

            // Set text color
            int textColor = (rText & 0xFF) | ((gText & 0xFF) << 8) | ((bText & 0xFF) << 16);
            int[] textColorArray = new int[]{textColor};
            Dwmapi.INSTANCE.DwmSetWindowAttribute(
                    hwnd,
                    Dwmapi.DWMWA_TEXT_COLOR,
                    textColorArray,
                    textColorArray.length * 4
            );

            // Set border color
            int borderColor = (rBorder & 0xFF) | ((gBorder & 0xFF) << 8) | ((bBorder & 0xFF) << 16);
            int[] borderColorArray = new int[]{borderColor};
            Dwmapi.INSTANCE.DwmSetWindowAttribute(
                    hwnd,
                    Dwmapi.DWMWA_BORDER_COLOR,
                    borderColorArray,
                    borderColorArray.length * 4
            );

            logger.info("Title bar customized: caption RGB(" + rCaption + ", " + gCaption + ", " + bCaption +
                    "), text RGB(" + rText + ", " + gText + ", " + bText +
                    "), border RGB(" + rBorder + ", " + gBorder + ", " + bBorder + ")");
        } catch (Exception e) {
            logger.log(Level.SEVERE, "Error setting title bar colors: " + e.getMessage(), e);
            logger.severe("Ensure you are running on Windows 10 (version 1809 or later) with DWM enabled.");
        }
    }


    public static void applyTheme(Stage stage,
                                  String captionHex,
                                  String textHex,
                                  String borderHex) {
        int[] caption = hexToRgb(captionHex);
        int[] text    = hexToRgb(textHex);
        int[] border  = hexToRgb(borderHex);

        setTitleBarColors(stage,
                caption[0], caption[1], caption[2],
                text[0], text[1], text[2],
                border[0], border[1], border[2]);
    }


    private static int[] hexToRgb(String hex) {
        String clean = hex.replace("#", "");
        int rgb = Integer.parseInt(clean, 16);
        int r = (rgb >> 16) & 0xFF;
        int g = (rgb >> 8)  & 0xFF;
        int b = rgb & 0xFF;
        return new int[]{r, g, b};
    }
}
