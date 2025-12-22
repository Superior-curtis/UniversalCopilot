@echo off
REM Uninstall.bat - One-click uninstaller for Universal Copilot
REM Usage: Double-click to uninstall, or run from command line
REM This script stops the process and removes all traces

setlocal enabledelayedexpansion
set INSTALL_DIR=%LOCALAPPDATA%\UniversalCopilot

echo.
echo Uninstalling Universal Copilot...
echo.

if not exist "%INSTALL_DIR%" (
    echo Application is not installed (directory not found).
    pause
    exit /b 0
)

REM Stop the running process
echo Stopping process...
taskkill /IM universal_copilot.exe /F >nul 2>&1

REM Run the portable uninstaller PowerShell script for registry cleanup
echo Removing registry entries...
powershell -ExecutionPolicy Bypass -File "%INSTALL_DIR%\portable_uninstall.ps1"

if errorlevel 1 (
    echo Warning: Some cleanup may have failed.
)

echo.
echo Uninstallation complete!
echo %INSTALL_DIR% has been removed.
echo.
pause
