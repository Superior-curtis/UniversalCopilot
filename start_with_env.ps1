$cfgPath = Join-Path $env:LOCALAPPDATA 'UniversalCopilot\config.ini'
if (-not (Test-Path $cfgPath)) { Write-Error "config.ini not found: $cfgPath"; exit 1 }
$cfgText = Get-Content -LiteralPath $cfgPath -Raw
$cfg = ConvertFrom-StringData $cfgText
foreach ($k in $cfg.Keys) { ${env:$k} = $cfg[$k] }
$exe = Join-Path $env:LOCALAPPDATA 'UniversalCopilot\universal_copilot.exe'
if (-not (Test-Path $exe)) { Write-Error "exe not found: $exe"; exit 1 }
Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe)
Write-Output "Started: $exe" 
