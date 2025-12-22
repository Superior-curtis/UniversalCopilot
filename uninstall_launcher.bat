@echo off
REM Runs portable_uninstall.ps1 from extracted SFX folder
powershell -ExecutionPolicy Bypass -File "%~dp0portable_uninstall.ps1"
