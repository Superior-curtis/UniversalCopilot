# UniversalCopilot Installer

This folder contains a Windows installer for UniversalCopilot and (optionally) Ollama.

## What you get
- `install.exe`: One-click installer for UniversalCopilot.
- Installs Ollama silently if bundled.
- Adds a Start Menu shortcut and an uninstaller entry. Uninstaller attempts to remove Ollama if found.

## Prerequisites
- Windows 10/11 x64.
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) installed (for building the installer).
- Rust toolchain installed to build the app.

## Bundle Ollama (offline)
If you want the installer to include Ollama without downloading:
1. Download the Ollama Windows installer (e.g., `OllamaSetup.exe`).
2. Place it at `installer/ollama/OllamaSetup.exe`.
3. The installer will install Ollama silently (`/S`) during setup.

If you skip this step, the installer will not install Ollama. You can install Ollama separately and re-run UniversalCopilot.

The installer will try to pre-pull the `mistral` model if Ollama is present. This requires internet access and may take time. You can change the model by editing `UniversalCopilot.iss`.

## Build the installer
From a PowerShell terminal:

```powershell
# Build app and compile installer
cd C:\Users\huach\Downloads\Code\UniversalCopilot\installer
./build.ps1
```

Outputs:
- `installer/dist/install.exe`

## Install & Uninstall
- Run `install.exe` to install UniversalCopilot (and Ollama if bundled).
- Start Menu: UniversalCopilot.
- Uninstall: Use "Uninstall UniversalCopilot" shortcut or Windows Apps & features.
- The uninstaller will attempt to uninstall Ollama if its uninstaller is found in common locations.

## Notes
- If you see build access denied errors, stop any running `universal_copilot.exe` before rebuilding:
  ```powershell
  Get-Process -Name universal_copilot | Stop-Process -Force
  ```
- To change the default model pre-pull, edit the `Run` section of `UniversalCopilot.iss`.
- If Ollama isn’t found in the typical install paths, the installer will skip model pre-pull/uninstall steps.
