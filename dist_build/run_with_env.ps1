<#
run_with_env.ps1
Reads config.ini in the same folder (if present), sets environment variables in this process, and starts universal_copilot.exe.
Do NOT commit your API key to the repo. This script reads the per-install `config.ini` written by the installer.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$cfg = Join-Path $root 'config.ini'
if (Test-Path $cfg) {
    $lines = Get-Content $cfg | Where-Object { $_ -match '=' }
    foreach ($l in $lines) {
        $parts = $l -split '=',2
        if ($parts.Length -eq 2) {
            $k = $parts[0].Trim()
            $v = $parts[1].Trim()
            if ($k -ne '[api]') {
                $env:$k = $v
            }
        }
    }
}
$exe = Join-Path $root 'universal_copilot.exe'
if (-Not (Test-Path $exe)) { Write-Host "Cannot find $exe"; exit 1 }
Start-Process -FilePath $exe -WorkingDirectory $root
