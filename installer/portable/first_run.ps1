# First-run helper for UniversalCopilot portable
# - Checks for Ollama, installs if bundled, pulls default model, then starts app.

$ErrorActionPreference = 'Stop'

function Get-OllamaExe {
  $paths = @(
    "$Env:LOCALAPPDATA\Programs\Ollama\ollama.exe",
    "$Env:ProgramFiles\Ollama\ollama.exe",
    "$Env:ProgramFiles(x86)\Ollama\ollama.exe"
  )
  foreach ($p in $paths) { if (Test-Path $p) { return $p } }
  # Try PATH
  $cmd = Get-Command ollama -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Path }
  return $null
}

function Install-OllamaIfBundled {
  $bundled = Join-Path $PSScriptRoot 'OllamaSetup.exe'
  if (Test-Path $bundled) {
    Write-Host "Installing Ollama (bundled) ..." -ForegroundColor Cyan
    Start-Process -FilePath $bundled -ArgumentList '/S' -Wait -WindowStyle Hidden
  }
}

function Ensure-Model {
  param([string]$ModelName = 'mistral')
  $ollama = Get-OllamaExe
  if (-not $ollama) { return }
  Write-Host "Preparing Ollama model '$ModelName' ..." -ForegroundColor Cyan
  # Try run (pulls if needed), fallback to pull
  try {
    Start-Process -FilePath $ollama -ArgumentList @('run', $ModelName) -Wait -WindowStyle Hidden
  } catch {
    Start-Process -FilePath $ollama -ArgumentList @('pull', $ModelName) -Wait -WindowStyle Hidden
  }
}

# Main
$ollamaPath = Get-OllamaExe
if (-not $ollamaPath) {
  Install-OllamaIfBundled
  $ollamaPath = Get-OllamaExe
  if (-not $ollamaPath) {
    Write-Warning "Ollama not found and no bundled installer present. Please install Ollama manually from https://ollama.com/download"
  }
}

Ensure-Model -ModelName 'mistral'

# Start the app
$app = Join-Path $PSScriptRoot 'universal_copilot.exe'
if (-not (Test-Path $app)) { throw "App binary not found at $app" }
Write-Host "Starting UniversalCopilot ..." -ForegroundColor Green
Start-Process -FilePath $app -WorkingDirectory $PSScriptRoot