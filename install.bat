@echo off
REM UniversalCopilot One-Click Installer with Ollama Setup
REM Downloads Ollama + Mistral model, installs app, and launches setup

setlocal enabledelayedexpansion

echo.
echo ============================================================
echo   UniversalCopilot Setup (with Ollama + Mistral-7B)
echo ============================================================
echo.

set INSTALL_DIR=%LOCALAPPDATA%\UniversalCopilot
set APP_EXE=%~dp0universal_copilot.exe

if not exist "%APP_EXE%" (
    echo ERROR: universal_copilot.exe not found in %~dp0
    pause
    exit /b 1
)

echo [1] Creating installation directory...
mkdir "%INSTALL_DIR%" 2>nul
copy "%APP_EXE%" "%INSTALL_DIR%\universal_copilot.exe" >nul
echo   [OK] App copied to %INSTALL_DIR%

echo.
echo [2] Checking for Ollama installation...
ollama --version >nul 2>&1
if errorlevel 1 (
    echo   [!] Ollama not found. Downloading installer...
    echo   [i] Opening https://ollama.ai in your browser...
    start https://ollama.ai
    echo.
    echo   Please:
    echo   1. Download and install Ollama from the page that opened
    echo   2. Run this installer again after Ollama installation is complete
    pause
    exit /b 1
) else (
    echo   [OK] Ollama is installed
)

echo.
echo [3] Pulling Mistral-7B model (this may take 5-10 minutes)...
echo   [i] If this is your first time, grab a coffee!
echo.
ollama pull mistral
if errorlevel 1 (
    echo   [!] Failed to pull Mistral model
    echo   [i] Try manually: ollama pull mistral
    pause
    exit /b 1
)
echo   [OK] Mistral model downloaded

echo.
echo [4] Creating desktop shortcut...
set DESKTOP=%USERPROFILE%\Desktop

REM Create VBS script for shortcut
set VBS_FILE=%TEMP%\create_shortcut.vbs
(
    echo Set oWS = WScript.CreateObject("WScript.Shell"^)
    echo sLinkFile = "%DESKTOP%\UniversalCopilot.lnk"
    echo Set oLink = oWS.CreateShortcut(sLinkFile^)
    echo oLink.TargetPath = "%INSTALL_DIR%\universal_copilot.exe"
    echo oLink.WorkingDirectory = "%INSTALL_DIR%"
    echo oLink.Description = "UniversalCopilot - Local AI Inline Suggestions"
    echo oLink.Save
) > "%VBS_FILE%"

cscript "%VBS_FILE%" >nul 2>&1
del "%VBS_FILE%"
echo   [OK] Desktop shortcut created

echo.
echo ============================================================
echo   Installation Complete!
echo ============================================================
echo.
echo Next Steps:
echo   1. Open a new PowerShell window and run: ollama serve
echo      (leave this running in the background)
echo.
echo   2. Click the "UniversalCopilot" shortcut on your desktop
echo.
echo   3. Open Notepad and start typing!
echo      - Suggestions appear as ghost text
echo      - Press TAB to accept
echo      - Press ESC to dismiss
echo.
echo Tip: Set ollama serve to auto-start on boot!
echo.
echo ============================================================
echo.

REM Option to start Ollama now
set /p START_OLLAMA="Start Ollama now? (y/n): "
if /i "%START_OLLAMA%"=="y" (
    start cmd /k "ollama serve"
    timeout /t 2
)

REM Launch UniversalCopilot
echo.
echo Starting UniversalCopilot...
start "" "%INSTALL_DIR%\universal_copilot.exe"

pausesetlocal enabledelayedexpansion
set SCRIPT_DIR=%~dp0
set INSTALL_DIR=%LOCALAPPDATA%\UniversalCopilot

echo.
echo Installing Universal Copilot...
echo.

REM Create install directory
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Copy only the necessary files (explicitly list them)
echo Copying files to %INSTALL_DIR%...
for %%F in (
    "universal_copilot.exe"
    "portable_install.ps1"
    "portable_uninstall.ps1"
    "README_INSTALL.md"
) do (
    if exist "%SCRIPT_DIR%%%F" (
        copy "%SCRIPT_DIR%%%F" "%INSTALL_DIR%\" /Y >nul 2>&1
    ) else (
        echo Warning: Missing file %%F
    )
)

REM Copy optional files from dist subfolder
for %%F in (
    "dist\run_with_env.ps1"
    "dist\start.bat"
    "dist\start-minimized.vbs"
) do (
    if exist "%SCRIPT_DIR%%%F" (
        copy "%SCRIPT_DIR%%%F" "%INSTALL_DIR%\" /Y >nul 2>&1
    )
)

if not exist "%INSTALL_DIR%\universal_copilot.exe" (
    echo Error: Failed to copy universal_copilot.exe
    pause
    exit /b 1
)

REM Run the portable installer PowerShell script for registry registration, startup shortcuts, etc.
echo Registering application...
powershell -ExecutionPolicy Bypass -File "%INSTALL_DIR%\portable_install.ps1"

if errorlevel 1 (
    echo Warning: Registration may have failed, but files were copied.
    pause
)

echo.
echo Installation complete!
echo The app is installed to: %INSTALL_DIR%
echo.
echo You can now:
echo  - Run: %INSTALL_DIR%\start.bat
echo  - Or uninstall by running: uninstall.bat from this folder
echo.
pause
