@echo off
setlocal

REM One-click launcher for UniversalCopilot
REM - Checks Ollama availability and attempts to start the server if needed
REM - Launches the app from root or falls back to target\release

cd /d "%~dp0"

REM Check Ollama CLI presence
where ollama >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
  echo [Info] Ollama CLI not found on PATH. Skipping server start.
) else (
  REM Probe Ollama server by listing models
  ollama list >nul 2>nul
  if %ERRORLEVEL% NEQ 0 (
    echo [Info] Ollama server not responding. Starting background server...
    start "" /min ollama serve
    timeout /t 2 /nobreak >nul
  ) else (
    echo [Info] Ollama server is running.
  )
)

REM Determine app executable path
set "APP_EXE=%~dp0universal_copilot.exe"
if not exist "%APP_EXE%" (
  set "APP_EXE=%~dp0target\release\universal_copilot.exe"
)

if not exist "%APP_EXE%" (
  echo [Error] UniversalCopilot executable not found.
  echo         Expected at: %~dp0universal_copilot.exe
  echo         or fallback: %~dp0target\release\universal_copilot.exe
  echo         Build with: cargo build --release
  pause
  exit /b 1
)

echo [Info] Starting UniversalCopilot...
start "" "%APP_EXE%"
echo [Done] App launched. Press any key to close this window.
pause
