Universal Copilot — Install & Uninstall Guide

What this app does
- Provides inline, per-caret "ghost text" suggestions from a remote LLM (opt-in).
- Local process runs on Windows and renders a transparent overlay with suggestions.
- Suggestions are accepted with TAB, dismissed with ESC.

What this app does NOT do
- This is NOT a keylogger. It does not send your keystrokes to a remote server.
- It does NOT persist typed text or store content on disk by default.
- Remote inference is opt-in: you must set an API key to enable network calls.

Privacy & Permissions
- Why it asks for input/access: to display inline suggestions near the caret and accept TAB to insert suggestions, this app needs to inspect the caret position and use a low-level keyboard hook so it can accept/dismiss suggestions. This is necessary to integrate across different Windows applications.
- No admin rights required by default. The portable installer writes only to user-scoped locations (e.g., `%LOCALAPPDATA%`) and registry keys under HKCU so no elevation is needed.
- Elevated or UIAccess-required scenarios: to interact with elevated applications (processes running as Administrator), either run Universal Copilot elevated or grant the app UIAccess by signing and placing it in `Program Files` with a suitable manifest—this is advanced and not required for normal usage.

Binary & Process naming
- Binary/process name: `universal_copilot.exe` (keeps a clear readable name)
- Window class used by the overlay: `UniversalCopilotOverlay`

Quick portable install (1 minute)
1. Unzip the `UniversalCopilot.zip` to any folder you own (e.g., `C:\Users\<you>\AppData\Local\UniversalCopilot`).
2. Run `portable_install.ps1` (included) or double-click `start.bat` to launch.
3. Optionally set your API key in an environment variable `VITE_OPENAI_API_KEY` or set a small config file (instructions below).

Quick portable uninstall (under 1 minute)
1. Run `portable_uninstall.ps1` from within the installation folder or via Explorer. This will:
   - Stop the running `universal_copilot.exe` process (graceful stop, then forced if needed).
   - Remove the installation folder if requested.
   - Remove the "Add or Remove Programs" entry created in the installer step (HKCU uninstall key).
   - Remove any Start Menu / Startup shortcut created by the installer script.
2. Verify the process is no longer running in Task Manager.

Add/Remove Programs entry (user-scoped)
- For a simple single-click experience we create an HKCU uninstall key so the app appears in Settings → Apps → Installed apps for the current user. This does NOT require admin rights.
- The uninstall record points to `portable_uninstall.ps1` so the user can uninstall from the system UI.

Startup behavior
- Default: the app does NOT auto-start.
- Optional: the portable installer can create a shortcut in the current user's Startup folder (`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`) if the user opts in during install.
- Deferred advanced option: a registry-based Run key under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` can be used instead (documented below) but will be implemented later if requested.

Trust & transparency
- No keylogging, no persistent text storage by default.
- Remote inference (sending context to an LLM) is strictly opt-in and requires setting an API key: `VITE_OPENAI_API_KEY`.

Packaging recommendations
- Portable ZIP (recommended for quick trials): Create a `UniversalCopilot.zip` containing the single `universal_copilot.exe` plus `portable_install.ps1`, `portable_uninstall.ps1`, and `README_INSTALL.md`.
- MSI via WiX (recommended for a polished installer experience): Use WiX to build an MSI that installs to `%ProgramFiles(x86)%` or `%LocalAppData%` and registers Add/Remove metadata, Start Menu shortcuts and an Uninstall command. See `installer.wxs` example in this repo.

Future: code signing
- For UIAccess and to reduce SmartScreen warnings, code signing is recommended in production. This repo does not include signing; signing would be part of a later release process.

Support
- If you have questions about removal, run `portable_uninstall.ps1` or delete the folder if you used a portable install and the process is not running.

Files included for install helpers
- `portable_install.ps1` — copies files to `%LOCALAPPDATA%` and registers a HKCU uninstall entry; optional startup shortcut.
- `portable_uninstall.ps1` — stops the running process, removes files and registry entries.
- `installer.wxs` — minimal WiX example for building an MSI (developer-facing).
- `BUILD_INSTRUCTIONS.md` — step-by-step packaging/build commands.

If you want, I can produce a one-click packaged ZIP and MSI build script (developer steps) next.

One-click `install.exe` / `uninstall.exe` (SFX) using IExpress
- This repository includes `make_selfextract.ps1` which can produce two simple self-extracting EXEs (`install.exe` and `uninstall.exe`) using the built-in Windows `iexpress.exe`.
- `install.exe` extracts contents to a temporary folder and runs an internal `install_launcher.bat` which invokes `portable_install.ps1` to copy files to `%LOCALAPPDATA%` and optionally write your API key into `config.ini`.
- `uninstall.exe` extracts and runs `uninstall_launcher.bat` which runs `portable_uninstall.ps1` to stop the process and remove files/registry entries.

Security note: embedding an API key into the SFX bundle will write the key into the user's install folder. Only do this for local demos and do NOT distribute installers containing private keys publicly.