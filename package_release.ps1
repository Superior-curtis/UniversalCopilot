<#
package_release.ps1
Builds a release binary and packages a portable zip in ./dist
Run from repo root. Requires cargo in PATH.
#>
set -e

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $root

Write-Host "Building release..."
cargo build --release

$dist = Join-Path $root 'dist'
if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
New-Item -ItemType Directory -Path $dist | Out-Null

# Copy runtime files
Copy-Item -Path target\release\universal_copilot.exe -Destination $dist -Force
Copy-Item -Path portable_install.ps1,portable_uninstall.ps1,README_INSTALL.md -Destination $dist -Force
Copy-Item -Path dist\start.bat,dist\start-minimized.vbs,dist\run_with_env.ps1 -Destination $dist -Force

# Create zip
$zip = Join-Path $root 'UniversalCopilot.zip'
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $dist '*') -DestinationPath $zip -Force

Write-Host "Packaged: $zip"
Pop-Location
