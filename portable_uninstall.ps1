<#
portable_uninstall.ps1
Stops running process, removes installation folder and HKCU uninstall entry.
Run this as user who installed the app. No admin rights required (HKCU only).
#>
param(
    [switch]$RemoveFiles = $true
)

$ErrorActionPreference = 'Continue'

$exeName = 'universal_copilot.exe'
$installDir = Join-Path -Path $env:LOCALAPPDATA -ChildPath 'UniversalCopilot'

Write-Host "Attempting to stop process: $exeName"
try {
    $procs = Get-Process -Name 'universal_copilot' -ErrorAction SilentlyContinue
    if ($procs) {
        foreach ($p in $procs) {
            try {
                Write-Host "Stopping PID $($p.Id)"
                $p.CloseMainWindow() | Out-Null
                Start-Sleep -Milliseconds 300
                if (-Not $p.HasExited) {
                    Write-Host "Forcing stop PID $($p.Id)"
                    Stop-Process -Id $p.Id -Force
                }
            } catch {
                Write-Host "Error stopping process: $_"
            }
        }
    } else {
        Write-Host "Process not running."
    }
} catch {
    Write-Host "Failed to query process: $_"
}

# Remove StartUp shortcut if present
$startupLnk = Join-Path ([Environment]::GetFolderPath('Startup')) 'Universal Copilot.lnk'
if (Test-Path $startupLnk) {
    Remove-Item $startupLnk -Force -ErrorAction SilentlyContinue
    Write-Host "Removed startup shortcut: $startupLnk"
}

# Remove registry uninstall entry
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\UniversalCopilot'
if (Test-Path $uninstallKey) {
    Remove-Item $uninstallKey -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "Removed HKCU uninstall key"
}

# Optionally remove installation files
if ($RemoveFiles -and (Test-Path $installDir)) {
    Write-Host "Removing files at $installDir"
    try {
        Remove-Item $installDir -Recurse -Force -ErrorAction Stop
        Write-Host "Files removed"
    } catch {
        Write-Host "Failed to remove files: $_"
    }
}

Write-Host "Uninstall complete."