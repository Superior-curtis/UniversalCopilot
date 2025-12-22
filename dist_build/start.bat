@echo off
REM Start Universal Copilot in this folder (expects universal_copilot.exe in same folder)
set EXE=%~dp0universal_copilot.exe
if not exist "%EXE%" (
  echo Cannot find universal_copilot.exe in %~dp0
  pause
  exit /b 1
)
start "Universal Copilot" "%EXE%"
exit /b 0
