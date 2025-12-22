<#
portable_install.ps1
Usage: Run from the folder that contains `universal_copilot.exe`, `portable_uninstall.ps1` and this script.
This script copies the current folder to `%LOCALAPPDATA%\UniversalCopilot`, creates a per-user Uninstall registry entry under HKCU, and optionally adds a Startup shortcut.
No admin rights required (writes to HKCU and %LOCALAPPDATA%).
#>
param(
    [switch]$CreateStartupShortcut,
    [string]$ApiKey,
    [string]$ApiUrl
)

$ErrorActionPreference = 'Stop'

$exeName = 'universal_copilot.exe'
$installDir = Join-Path -Path $env:LOCALAPPDATA -ChildPath 'UniversalCopilot'

Write-Host "Installing to: $installDir"

# Interactive prompt if no flag provided
function Ask-StartupOptInGui {
    Add-Type -AssemblyName System.Windows.Forms
    $form = New-Object System.Windows.Forms.Form
    $form.Text = 'Universal Copilot Setup'
    $form.Size = New-Object System.Drawing.Size(420,160)
    $form.StartPosition = 'CenterScreen'

    $lbl = New-Object System.Windows.Forms.Label
    $lbl.Text = 'Create a Startup shortcut so Universal Copilot starts with Windows?'
    $lbl.Size = New-Object System.Drawing.Size(380,30)
    $lbl.Location = New-Object System.Drawing.Point(10,10)
    $form.Controls.Add($lbl)

    $chk = New-Object System.Windows.Forms.CheckBox
    $chk.Text = 'Start with Windows (create per-user Startup shortcut)'
    $chk.Size = New-Object System.Drawing.Size(380,20)
    $chk.Location = New-Object System.Drawing.Point(10,45)
    $form.Controls.Add($chk)

    $ok = New-Object System.Windows.Forms.Button
    $ok.Text = 'OK'
    $ok.Location = New-Object System.Drawing.Point(220,80)
    $ok.Add_Click({ $form.Tag = $chk.Checked; $form.Close() })
    $form.Controls.Add($ok)

    $cancel = New-Object System.Windows.Forms.Button
    $cancel.Text = 'Cancel'
    $cancel.Location = New-Object System.Drawing.Point(300,80)
    $cancel.Add_Click({ $form.Tag = $false; $form.Close() })
    $form.Controls.Add($cancel)

    $form.ShowDialog() | Out-Null
    return [bool]$form.Tag
}

if (-not $PSBoundParameters.ContainsKey('CreateStartupShortcut')) {
    try {
        $CreateStartupShortcut = Ask-StartupOptInGui
    } catch {
        # fallback to console prompt
        Write-Host "Create a Startup shortcut so the app starts with Windows? (Y/N) [N]: " -NoNewline
        $ans = Read-Host
        if ($ans -match '^[Yy]') { $CreateStartupShortcut = $true } else { $CreateStartupShortcut = $false }
    }
}

# Copy files to install dir
if (-Not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

Get-ChildItem -Path . -File | ForEach-Object {
    # Do not copy development-only files
    if ($_.Name -in @('README.md','README_INSTALL.md','BUILD_INSTRUCTIONS.md','.gitignore')) { return }
    Copy-Item -Path $_.FullName -Destination (Join-Path $installDir $_.Name) -Force
}

# If ApiKey provided, write config.ini (restricted to current user)
if ($ApiKey) {
    $cfgPath = Join-Path $installDir 'config.ini'
    $content = @()
    $content += '[api]'
    $content += "VITE_OPENAI_API_KEY=$ApiKey"
    if ($ApiUrl) { $content += "VITE_OPENAI_API_URL=$ApiUrl" }
    $content | Out-File -FilePath $cfgPath -Encoding UTF8 -Force

    # Restrict file permissions to current user only
    try {
        icacls $cfgPath /inheritance:r | Out-Null
        icacls $cfgPath /grant:r "$($env:USERNAME):(R,W)" | Out-Null
    } catch {
        Write-Host "Warning: failed to restrict permissions on config file: $_"
    }
    Write-Host "Wrote config file to: $cfgPath (permissions restricted to the current user)"
}

# Create a simple Uninstall entry under HKCU so the app shows in Add/Remove (current user)
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\UniversalCopilot'
if (-Not (Test-Path $uninstallKey)) {
    New-Item -Path $uninstallKey | Out-Null
}
Set-ItemProperty -Path $uninstallKey -Name 'DisplayName' -Value 'Universal Copilot'
Set-ItemProperty -Path $uninstallKey -Name 'DisplayVersion' -Value '0.1.0'
Set-ItemProperty -Path $uninstallKey -Name 'Publisher' -Value 'UniversalCopilot'
Set-ItemProperty -Path $uninstallKey -Name 'UninstallString' -Value "`"$installDir\portable_uninstall.ps1`""
Set-ItemProperty -Path $uninstallKey -Name 'QuietUninstallString' -Value "`"$installDir\portable_uninstall.ps1`""

# Optional: create a Startup shortcut in the current user's Startup folder
if ($CreateStartupShortcut) {
    $startupFolder = [Environment]::GetFolderPath('Startup')
    $lnkPath = Join-Path $startupFolder 'Universal Copilot.lnk'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($lnkPath)
    $shortcut.TargetPath = Join-Path $installDir $exeName
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Save()
    Write-Host "Startup shortcut created: $lnkPath"
}

Write-Host "Installation complete. To run: $installDir\$exeName"