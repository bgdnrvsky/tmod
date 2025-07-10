# TMod Manager

A modern GUI application for managing Minecraft modpacks with dependency resolution and compatibility checking
<br/><br/>
<img width="998" height="864" alt="изображение" src="https://github.com/user-attachments/assets/5355097a-9d36-4783-8d27-b9a43c57809d" />


## Features

- **Intuitive Interface**: Clean, modern dark theme with easy-to-use controls
- **Dependency Management**: Automatically resolves and installs mod dependencies
- **Real-time Logging**: View operation progress and results in an integrated log panel
- **Mod Overview**: List view of all installed mods with count display
- **Batch Operations**: Install all mods at once or manage them individually
- **Cross-platform**: Built with JavaFX for Windows, macOS, and Linux support

## Interface

The application features a split-panel layout:
- **Left Panel**: Displays your installed mods with selection and count information
- **Right Panel**: Shows real-time activity logs and operation feedback
- **Toolbar**: Quick access buttons for adding, removing, installing, and refreshing mods
- **Status Bar**: Progress indication and current operation status

## Usage

### Adding Mods
Click "Add Mod" and select a mod directory. The application will automatically detect dependencies and add them to your modpack.

### Removing Mods
Select a mod from the list and click "Remove Mod". A confirmation dialog will appear before removal.

### Installing Mods
Click "Install All" to download and install all mods in your list along with their dependencies.

### Refreshing
Use the "Refresh" button to update the mod list and sync with the current state of your modpack.

## Colorscheme

| Color         | Badge                                                                                                               | Hex Code   |
| ------------- | --------------------------------------------------------------------------------------------------------------------- | ---------- |
| 🔶 **Orange** | ![Orange](https://img.shields.io/badge/Accent_Orange-ba694b?style=flat&logo=materialdesignicons&logoColor=f9f8f4)   | `#ba694b`  |
| 🔷 **Blue**   | ![Blue](https://img.shields.io/badge/Accent_Blue-3d6cac?style=flat&logo=materialdesignicons&logoColor=f9f8f4)       | `#3d6cac`  |
| 🔴 **Red**    | ![Red](https://img.shields.io/badge/Accent_Red-d67762?style=flat&logo=materialdesignicons&logoColor=f9f8f4)         | `#d67762`  |
| 🟣 **Purple** | ![Purple](https://img.shields.io/badge/Accent_Purple-9b7aa6?style=flat&logo=materialdesignicons&logoColor=f9f8f4)   | `#9b7aa6`  |
| 🟡 **Yellow** | ![Yellow](https://img.shields.io/badge/Accent_Yellow-c9a96e?style=flat&logo=materialdesignicons&logoColor=f9f8f4)   | `#c9a96e`  |
| 🟢 **Aqua**   | ![Aqua](https://img.shields.io/badge/Accent_Aqua-6b9b9b?style=flat&logo=materialdesignicons&logoColor=f9f8f4)       | `#6b9b9b`  |

## Technical Details

- Built with JavaFX for cross-platform compatibility
- Integrates with TMod CLI backend for mod management operations
- Features custom styling with a cohesive dark theme
- Includes progress indicators and status animations
- Supports background task execution to maintain UI responsiveness

## Requirements

- Java 11 or higher
- JavaFX runtime
- TMod CLI backend
