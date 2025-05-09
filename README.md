# Expedition 33 UnnieModManager

A modern, user-friendly tool for managing UE4SS and mods for Unreal Engine games, with both a powerful GUI and CLI.

---

## Features

### GUI (Graphical User Interface)
- **Beautiful, Responsive Design:**
  - Dark theme, large readable fonts, and accent-colored buttons for clarity.
  - Remembers window size between runs.
- **Game Directory Selection:**
  - Easily select your game's `Win64` directory with a file dialog.
  - Example path shown for clarity: `Expedition 33\Sandfall\Binaries\Win64`
- **Install UE4SS:**
  - One-click download and install of the latest UE4SS into your selected game directory.
- **Mod Management:**
  - Install mods from `.zip` files directly into the game's `Mods` folder.
  - View a list of installed mods.
  - Open the `Mods` folder in your system's file explorer.
- **Debug Output:**
  - Toggle debug mode to see detailed logs of operations.
  - Output area shows clear, up-to-date status and error messages.
- **UI Scale:**
  - Adjust the UI scale for accessibility and comfort.

### CLI (Command Line Interface)
You can also use UnnieModManager from the command line for scripting or automation:

#### Install or Update UE4SS
```
UnnieModManager.exe install-ue4ss --target-dir <Win64 directory>
```
- Example:
  ```
  UnnieModManager.exe install-ue4ss --target-dir "C:\Program Files (x86)\Steam\steamapps\common\Expedition 33\Sandfall\Binaries\Win64"
  ```

#### Install a Mod from a Zip File
```
UnnieModManager.exe install-mod --zip-path <mod zip file> --target-dir <Win64 directory>
```
- Example:
  ```
  UnnieModManager.exe install-mod --zip-path "C:\Downloads\MyCoolMod.zip" --target-dir "C:\Program Files (x86)\Steam\steamapps\common\Expedition 33\Sandfall\Binaries\Win64"
  ```

#### List Installed Mods
```
UnnieModManager.exe list-mods --target-dir <Win64 directory>
```
- Example:
  ```
  UnnieModManager.exe list-mods --target-dir "C:\Program Files (x86)\Steam\steamapps\common\Expedition 33\Sandfall\Binaries\Win64"
  ```

#### Launch the GUI
```
UnnieModManager.exe gui
```
Or simply double-click the `.exe` to launch the GUI by default.

---

## Building

1. Install [Rust](https://rustup.rs/)
2. Clone this repository and open a terminal in the project directory.
3. Build the release executable:
   ```
   cargo build --release
   ```
4. The `.exe` will be in `target/release/UnnieModManager.exe`

---

## Notes
- The GUI and CLI are both included in the same `.exe`.
- The program will remember your last used game directory, installed mods, debug output, and window size.
- For best results, run as a user with write access to your game directory.

---

## License
MIT
