<#
make_selfextract.ps1
Creates two self-extracting EXEs using IEXPRESS (built into Windows):
 - install.exe: extracts files to %LOCALAPPDATA%\UniversalCopilot and runs portable_install.ps1
 - uninstall.exe: extracts and runs portable_uninstall.ps1

Usage (developer machine):
  powershell -ExecutionPolicy Bypass -File .\make_selfextract.ps1 [-EmbedApiKey <string>] [-ApiUrl <string>]

Notes:
- IExpress is a built-in tool on Windows (`iexpress.exe`). This script generates SED files and calls iexpress.
- The resulting EXEs are simple SFX installers that run the included scripts. They do not require NSIS.
- To build a polished MSI, use WiX as documented in BUILD_INSTRUCTIONS.md.
#>
param(
    [string]$EmbedApiKey = $null,
    [string]$ApiUrl = $null
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$dist = Join-Path $root 'dist_build'
if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
New-Item -ItemType Directory -Path $dist | Out-Null

# Files to include in the installer (from root)
$files = @(
    'portable_install.ps1',
    'portable_uninstall.ps1',
    'README_INSTALL.md'
)

# Files from dist/ subfolder
$distFiles = @(
    'dist/run_with_env.ps1',
    'dist/start.bat',
    'dist/start-minimized.vbs'
)

# ensure release binary exists
$exeSource = Join-Path $root 'target\release\universal_copilot.exe'
if (-not (Test-Path $exeSource)) {
    Write-Error "Release binary not found at $exeSource. Build with: cargo build --release"
}

# Copy the release binary into dist_build
Copy-Item -Path $exeSource -Destination (Join-Path $dist 'universal_copilot.exe') -Force

# Copy required files into builder dir
foreach ($f in $files) {
    $src = Join-Path $root $f
    if (-not (Test-Path $src)) { Write-Error "Missing file: $src" }
    Copy-Item -Path $src -Destination $dist -Force
}

# Copy dist subfolder files
foreach ($f in $distFiles) {
    $src = Join-Path $root $f
    if (-not (Test-Path $src)) { Write-Error "Missing file: $src" }
    $filename = Split-Path $src -Leaf
    Copy-Item -Path $src -Destination (Join-Path $dist $filename) -Force
}

# Optionally embed config.ini with API key (only if developer explicitly passes it)
if ($EmbedApiKey) {
    $cfg = "[api]`nVITE_OPENAI_API_KEY=$EmbedApiKey"
    if ($ApiUrl) { $cfg += "`nVITE_OPENAI_API_URL=$ApiUrl" }
    $cfgPath = Join-Path $dist 'config.ini'
    $cfg | Out-File -FilePath $cfgPath -Encoding UTF8 -Force
    Write-Host "Embedded config.ini into SFX content (developer warned: do not distribute publicly if key is sensitive)"
}

# Create install launcher batch to call portable_install.ps1 (the SFX will extract here and run the command)
$installLauncher = @"
@echo off
powershell -ExecutionPolicy Bypass -File "%~dp0portable_install.ps1" -ApiKey `"$EmbedApiKey`" -ApiUrl `"$ApiUrl`"
"@
Set-Content -Path (Join-Path $dist 'install_launcher.bat') -Value $installLauncher -Encoding ASCII

# Create uninstall launcher batch
$uninstallLauncher = @"
@echo off
powershell -ExecutionPolicy Bypass -File "%~dp0portable_uninstall.ps1"
"@
Set-Content -Path (Join-Path $dist 'uninstall_launcher.bat') -Value $uninstallLauncher -Encoding ASCII

# Build SED for install.exe (use relative path for output)
$sedInstall = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=0
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCode=0
RebootMode=I
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=install.exe
FriendlyName=Universal Copilot Installer
AppLaunched=install_launcher.bat
PostInstallCmd=<None>
AdminChoose=No
ExtractDir=%LOCALAPPDATA%\\UniversalCopilot
[SourceFiles]
SourceFiles0=.
"@
$sedInstallPath = Join-Path $dist 'install.sed'
$sedInstall | Out-File -FilePath $sedInstallPath -Encoding ASCII -NoNewline

# Append file list
$filesInDir = Get-ChildItem -Path $dist -File | Where-Object { $_.Name -ne 'install.sed' -and $_.Name -ne 'uninstall.sed' } | ForEach-Object { $_.Name }
Add-Content -Path $sedInstallPath -Value ""
Add-Content -Path $sedInstallPath -Value "[SourceFiles0]"
foreach ($fn in $filesInDir) { 
    Add-Content -Path $sedInstallPath -Value "$fn=$fn" 
}

# Build install SFX (iexpress outputs to current dir when TargetName is relative)
Write-Host "Building install.exe..."
Push-Location $dist
& iexpress.exe /N /Q install.sed
if (Test-Path "install.exe") {
    Move-Item "install.exe" $root -Force
    Write-Host "Created: $(Join-Path $root 'install.exe')"
} else {
    Write-Warning "iexpress did not create install.exe"
}
Pop-Location

# Build uninstall SED
$sedUninst = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=1
HideExtractAnimation=0
UseLongFileName=1
InsideCompressed=0
CAB_FixedSize=0
CAB_ResvCode=0
RebootMode=I
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=uninstall.exe
FriendlyName=Universal Copilot Uninstaller
AppLaunched=uninstall_launcher.bat
PostInstallCmd=<None>
AdminChoose=No
ExtractDir=%TEMP%\\UniversalCopilot_uninstall
[SourceFiles]
SourceFiles0=.
"@
$sedUninstallPath = Join-Path $dist 'uninstall.sed'
$sedUninst | Out-File -FilePath $sedUninstallPath -Encoding ASCII -NoNewline
Add-Content -Path $sedUninstallPath -Value ""
Add-Content -Path $sedUninstallPath -Value "[SourceFiles0]"
foreach ($fn in $filesInDir) { 
    Add-Content -Path $sedUninstallPath -Value "$fn=$fn" 
}

Write-Host "Building uninstall.exe..."
Push-Location $dist
& iexpress.exe /N /Q uninstall.sed
if (Test-Path "uninstall.exe") {
    Move-Item "uninstall.exe" $root -Force
    Write-Host "Created: $(Join-Path $root 'uninstall.exe')"
} else {
    Write-Warning "iexpress did not create uninstall.exe"
}
Pop-Location

Write-Host "Self-extracting installers completed."
if (Test-Path (Join-Path $root 'install.exe')) { Write-Host "✓ install.exe: $((Get-Item (Join-Path $root 'install.exe')).Length) bytes" }
if (Test-Path (Join-Path $root 'uninstall.exe')) { Write-Host "✓ uninstall.exe: $((Get-Item (Join-Path $root 'uninstall.exe')).Length) bytes" }
