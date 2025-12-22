Packaging & Build Instructions (developer)

1) Build the release binary (MSVC toolchain recommended on Windows)

```powershell
# from repo root
cargo build --release
# resulting binary: target\release\universal_copilot.exe
```

2) Create a portable ZIP (recommended for quick trials)

```powershell
# from repo root (Windows PowerShell)
$dist = "$PWD\dist"
Remove-Item $dist -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $dist | Out-Null
Copy-Item -Path target\release\universal_copilot.exe -Destination $dist -Force
Copy-Item -Path portable_install.ps1,portable_uninstall.ps1,README_INSTALL.md -Destination $dist -Force
Compress-Archive -Path $dist\* -DestinationPath UniversalCopilot.zip -Force
```

3) Build an MSI with WiX (optional, requires WiX Toolset installed)

- Install WiX (https://wixtoolset.org/) and add `candle.exe` and `light.exe` to PATH.
- Edit `installer.wxs` to point to the built `universal_copilot.exe` and set UpgradeCode.

```powershell
# from repo root
candle installer.wxs -o installer.wixobj
light installer.wixobj -o UniversalCopilot.msi
```

Notes & future steps
- For per-user MSI (no admin), set `InstallScope="perUser"` and adjust directories to LocalAppDataFolder.
- To provide a polished installer you should sign the binary and the MSI (recommended later).
- The WiX file here is an example only. Use `heat.exe` to harvest files for a full app dir.

User install/uninstall acceptance tests
- Portable: unzip, run `portable_install.ps1`, run the exe, then run `portable_uninstall.ps1` and confirm process terminates and folder/registry keys removed.
- MSI: run MSI installer (may require elevation), verify app appears in Settings→Apps, run uninstall from there and verify process stops and files removed.
