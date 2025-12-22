# Build UniversalCopilot release and compile installer via Inno Setup
param(
    [string]$InnoSetupPath = "$Env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe"
)

$ErrorActionPreference = 'Stop'

Push-Location "$PSScriptRoot\.."

Write-Host "Building UniversalCopilot (release)" -ForegroundColor Cyan
cargo build --release

Pop-Location

if (-not (Test-Path $InnoSetupPath)) {
    Write-Error "Inno Setup compiler not found at $InnoSetupPath. Install Inno Setup 6 and re-run."
}

$iss = Join-Path $PSScriptRoot 'UniversalCopilot.iss'
$dist = Join-Path $PSScriptRoot 'dist'

if (-not (Test-Path $dist)) { New-Item -ItemType Directory -Path $dist | Out-Null }

Write-Host "Compiling installer with Inno Setup" -ForegroundColor Cyan
& $InnoSetupPath $iss

Write-Host "Done. Output: $(Join-Path $dist 'install.exe')" -ForegroundColor Green
