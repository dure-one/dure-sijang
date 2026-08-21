# Installation

This guide walks you through installing Dure-Sijang and its prerequisites on Windows, macOS, and Linux. Dure-Sijang requires ADB (Android Debug Bridge) to communicate with your Android device for debloating, scanning, and installing apps.

## Overview

Before using Dure-Sijang, you need:

1. **ADB Platform-tools** - Required on all platforms to communicate with Android devices
2. **VC Redistributable** (Windows only) - Microsoft Visual C++ runtime libraries
3. **OpenGL/WebGL drivers** (Windows only) - Graphics libraries for the GUI

---

## Windows

### Prerequisites

| Component | Purpose | Required |
|-----------|---------|----------|
| ADB Platform-tools | Communicates with Android devices via USB/Wi-Fi | Yes |
| VC Redistributable | Microsoft C++ runtime for the application | Yes |
| OpenGL/WebGL (Mesa) | Graphics rendering for the GUI interface | Yes (if GPU drivers missing) |

### Step 1: Install VC Redistributable

The Visual C++ Redistributable installs runtime components required to run C++ applications built with Visual Studio.

Download the version matching your system architecture:

| Architecture | Download Link |
|-------------|---------------|
| ARM64 | https://aka.ms/vc14/vc_redist.arm64.exe |
| X86 (32-bit) | https://aka.ms/vc14/vc_redist.x86.exe |
| X64 (64-bit) | https://aka.ms/vc14/vc_redist.x64.exe |

Run the downloaded installer and follow the prompts to complete installation.

### Step 2: Install OpenGL/WebGL (Mesa) - If Needed

Mesa provides software-based OpenGL rendering. This is **only required** if:
- Your GPU drivers don't support OpenGL 3.3+
- You're running in a virtual machine
- You see graphics-related errors when launching Dure-Sijang

**Download:** https://github.com/pal1000/mesa-dist-win/releases/latest

Extract the archive and run the included deployment script.

### Step 3: Install ADB Platform-tools

ADB (Android Debug Bridge) is a command-line tool that lets you communicate with Android devices.

**Download:** https://dl.google.com/android/repository/platform-tools-latest-windows.zip

#### Automatic Installation (Recommended)

Open **PowerShell as Administrator** and run:

```powershell
# Download and extract platform-tools
$sdkPath = "$env:LOCALAPPDATA\Android\Sdk"
$ptPath = "$sdkPath\platform-tools"
New-Item -ItemType Directory -Force -Path $sdkPath | Out-Null
Invoke-WebRequest -Uri "https://dl.google.com/android/repository/platform-tools-latest-windows.zip" -OutFile "$env:TEMP\platform-tools.zip"
Expand-Archive -Path "$env:TEMP\platform-tools.zip" -DestinationPath $sdkPath -Force
Remove-Item "$env:TEMP\platform-tools.zip"

# Add to PATH permanently via registry
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$ptPath*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$ptPath", "User")
    Write-Host "Added $ptPath to user PATH. Restart your terminal."
} else {
    Write-Host "Path already contains platform-tools."
}
Write-Host "Installation complete. Run 'adb version' in a new terminal to verify."
```

#### Manual Installation

1. Download the ZIP file from the link above
2. Extract to `C:\Users\<YourUsername>\AppData\Local\Android\Sdk\platform-tools`
3. Add the folder to your system PATH:
   - Press `Win + X` and select "System"
   - Click "Advanced system settings"
   - Click "Environment Variables"
   - Under "User variables", select "Path" and click "Edit"
   - Click "New" and add the platform-tools path
   - Click OK to save

### Verification

Open a **new** terminal (Command Prompt or PowerShell) and run:

```
adb version
```

You should see output like:
```
Android Debug Bridge version 1.0.41
Version 35.0.0-11411520
```

---

## macOS

### Prerequisites

| Component | Purpose | Required |
|-----------|---------|----------|
| ADB Platform-tools | Communicates with Android devices via USB/Wi-Fi | Yes |

macOS includes all other necessary components (OpenGL support, C++ runtime).

### Install ADB Platform-tools

**Download:** https://dl.google.com/android/repository/platform-tools-latest-darwin.zip

#### Automatic Installation (Recommended)

Open **Terminal** and run:

```bash
# Download and extract platform-tools
SDK_PATH="$HOME/Library/Android/sdk"
PT_PATH="$SDK_PATH/platform-tools"
mkdir -p "$SDK_PATH"
curl -o /tmp/platform-tools.zip https://dl.google.com/android/repository/platform-tools-latest-darwin.zip
unzip -o /tmp/platform-tools.zip -d "$SDK_PATH"
rm /tmp/platform-tools.zip

# Add to PATH in shell config
SHELL_RC="$HOME/.zshrc"
if ! grep -q "platform-tools" "$SHELL_RC" 2>/dev/null; then
    echo "export PATH=\"\$PATH:$PT_PATH\"" >> "$SHELL_RC"
    echo "Added to $SHELL_RC. Run: source $SHELL_RC"
else
    echo "PATH already configured."
fi
echo "Run 'adb version' to verify installation."
```

#### Using Homebrew (Alternative)

If you have Homebrew installed:

```bash
brew install android-platform-tools
```

### Verification

Open a **new** terminal window (or run `source ~/.zshrc`) and run:

```bash
adb version
```

---

## Linux

### Prerequisites

| Component | Purpose | Required |
|-----------|---------|----------|
| ADB Platform-tools | Communicates with Android devices via USB/Wi-Fi | Yes |

Most Linux distributions include OpenGL and C++ runtime libraries by default.

### Install ADB Platform-tools

**Download:** https://dl.google.com/android/repository/platform-tools-latest-linux.zip

#### Option 1: Package Manager (Recommended)

Most distributions provide ADB through their package manager:

**Debian/Ubuntu:**
```bash
sudo apt update && sudo apt install android-tools-adb
```

**Fedora:**
```bash
sudo dnf install android-tools
```

**Arch Linux:**
```bash
sudo pacman -S android-tools
```

#### Option 2: Manual Installation

Open **Terminal** and run:

```bash
# Download and extract platform-tools
SDK_PATH="$HOME/Android/Sdk"
PT_PATH="$SDK_PATH/platform-tools"
mkdir -p "$SDK_PATH"
curl -o /tmp/platform-tools.zip https://dl.google.com/android/repository/platform-tools-latest-linux.zip
unzip -o /tmp/platform-tools.zip -d "$SDK_PATH"
rm /tmp/platform-tools.zip

# Add to PATH in shell config
SHELL_RC="$HOME/.bashrc"
[ -f "$HOME/.zshrc" ] && SHELL_RC="$HOME/.zshrc"
if ! grep -q "platform-tools" "$SHELL_RC" 2>/dev/null; then
    echo "export PATH=\"\$PATH:$PT_PATH\"" >> "$SHELL_RC"
    echo "Added to $SHELL_RC. Run: source $SHELL_RC"
else
    echo "PATH already configured."
fi
echo "Run 'adb version' to verify installation."
```

### USB Permissions (Linux Only)

On Linux, you may need to configure udev rules to access Android devices without root:

```bash
# Create udev rules for Android devices
sudo tee /etc/udev/rules.d/51-android.rules << 'EOF'
SUBSYSTEM=="usb", ATTR{idVendor}=="*", MODE="0666", GROUP="plugdev"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Add your user to plugdev group
sudo usermod -aG plugdev $USER
```

Log out and log back in for group changes to take effect.

### Verification

Open a **new** terminal window (or run `source ~/.bashrc`) and run:

```bash
adb version
```

---

## Android

Dure-Sijang is available as a native Android APK for browsing mycart stores on your mobile device.

### Installation

Download and install the APK from one of these sources:

**Google Play Store:**
https://play.google.com/store/apps/details?id=app.dure.sijang

**GitHub Releases:**
https://github.com/dure-one/dure-sijang/releases/latest

### First Launch

1. Open **Dure-Sijang** app
2. Grant storage permissions if prompted (for caching product images)
3. The app will sync the store directory from dure.one
4. Browse available mycart stores from the directory

### Using the App

**Store Directory:**
- View list of available mycart stores
- Tap a store to open it in a new tab
- Stores sync automatically from dure.one

**Browsing Modes:**
- **WebView Mode**: Full website with all features
- **API Mode**: Native product grid and cart (faster, uses less data)
- Switch modes using the mode toggle button

**Tab Management:**
- Create new tabs with the + button
- Swipe tabs to close them
- Tabs persist between app sessions

**Bookmarks:**
- Tap the bookmark icon to save current page
- Access bookmarks from the menu
- Bookmarks sync across app restarts

### Storage Locations

- **Database**: `/data/data/app.dure.sijang/dbs/`
- **Settings**: `/data/data/app.dure.sijang/files/settings.txt`
- **Image Cache**: Internal cache directory (managed automatically)

### Troubleshooting

**App crashes on startup:**
- Clear app data from Android Settings
- Reinstall the app
- Check available storage space

**Images not loading:**
- Check internet connection
- Grant storage permissions
- Clear app cache from Android Settings

**Store directory not syncing:**
- Verify internet connection
- Check that dure.one is accessible
- Try pull-to-refresh in the directory view

---

## Troubleshooting

### ADB not found after installation

- **Windows:** Open a new terminal after installation. The PATH changes require a new session.
- **macOS/Linux:** Run `source ~/.zshrc` or `source ~/.bashrc` to reload your shell configuration.

### Device not detected

1. Enable **USB Debugging** on your Android device:
   - Go to Settings > About phone
   - Tap "Build number" 7 times to enable Developer options
   - Go to Settings > Developer options
   - Enable "USB debugging"

2. Run `adb devices` to check if your device is listed

3. If the device shows as "unauthorized", check your phone for the authorization prompt

### Graphics issues on Windows

If Dure-Sijang shows graphics errors or a blank window:
1. Update your GPU drivers
2. If that doesn't help, install Mesa from the link above
