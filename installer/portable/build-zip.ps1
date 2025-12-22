# Build a portable ZIP for UniversalCopilot (optionally includes Ollama installer)
$ErrorActionPreference = 'Stop'

$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\\..')).Path
$staging = Join-Path $PSScriptRoot 'staging'
$dist = Join-Path $PSScriptRoot 'dist'
$zipName = 'UniversalCopilot-portable.zip'

if (Test-Path $staging) { Remove-Item -Recurse -Force $staging }
New-Item -ItemType Directory -Path $staging | Out-Null
if (-not (Test-Path $dist)) { New-Item -ItemType Directory -Path $dist | Out-Null }

Write-Host "Building UniversalCopilot (release)" -ForegroundColor Cyan
Push-Location $projectRoot
cargo build --release
Pop-Location

# Copy app binary
$appExe = Join-Path $projectRoot 'target/release/universal_copilot.exe'
Copy-Item $appExe -Destination (Join-Path $staging 'universal_copilot.exe')

# Copy helper scripts and docs
Copy-Item (Join-Path $PSScriptRoot 'first_run.ps1') -Destination (Join-Path $staging 'first_run.ps1')
Copy-Item (Join-Path $PSScriptRoot 'README-portable.md') -Destination (Join-Path $staging 'README-portable.md')

# Optional: include Ollama installer if present in installer/ollama
$ollamaSetup = Join-Path (Join-Path $projectRoot 'installer/ollama') 'OllamaSetup.exe'
if (Test-Path $ollamaSetup) {
  Write-Host "Including bundled Ollama installer" -ForegroundColor Cyan
  Copy-Item $ollamaSetup -Destination (Join-Path $staging 'OllamaSetup.exe')
}

# Create ZIP
$zipPath = Join-Path $dist $zipName
if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
Write-Host "Creating portable ZIP: $zipPath" -ForegroundColor Cyan
Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zipPath

Write-Host "Done. Output: $zipPath" -ForegroundColor Green
