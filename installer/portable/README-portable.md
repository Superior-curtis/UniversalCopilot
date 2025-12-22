# UniversalCopilot Portable

This ZIP contains UniversalCopilot as a portable app with a first-run helper.

## Contents
- `universal_copilot.exe` — the app
- `first_run.ps1` — checks Ollama, installs if bundled, pulls `mistral` model, then starts the app
- `OllamaSetup.exe` (optional) — include this to enable offline one-click setup

## One-click Start (recommended)
Extract the ZIP to a folder and run:

```powershell
# From extracted folder
./first_run.ps1
```

This will:
1. Detect Ollama
2. Install Ollama silently if `OllamaSetup.exe` is present
3. Prepare the default model (`mistral`)
4. Launch UniversalCopilot

## Manual Start
If you already have Ollama installed and the model pulled, you can start the app directly:

```powershell
./universal_copilot.exe
```

## Changing the default model
Open `first_run.ps1` and change `Ensure-Model -ModelName 'mistral'` to your preferred model.

## Notes
- Admin privileges may be required to install Ollama silently.
- If Ollama isn’t bundled, install it from https://ollama.com/download and rerun `first_run.ps1`.
- To create this ZIP yourself, see `installer/portable/build-zip.ps1`.
