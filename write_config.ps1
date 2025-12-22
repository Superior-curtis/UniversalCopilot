$src = 'c:\Users\huach\Downloads\biofit-dashboard-main1\biofit-dashboard-main\.env.local'
$dest = Join-Path $env:LOCALAPPDATA 'UniversalCopilot\config.ini'
if (-not (Test-Path (Split-Path $dest))) { New-Item -ItemType Directory -Path (Split-Path $dest) -Force | Out-Null }
Get-Content -LiteralPath $src | Where-Object { $_ -match 'VITE_OPENAI_API_KEY|VITE_OPENAI_API_URL' } | Set-Content -LiteralPath $dest
$acl = Get-Acl -LiteralPath $dest
$acl.SetAccessRuleProtection($true,$false)
$rule = New-Object System.Security.AccessControl.FileSystemAccessRule($env:USERNAME, 'FullControl', 'Allow')
$acl.SetAccessRule($rule)
Set-Acl -LiteralPath $dest -AclObject $acl
Write-Output "config.ini written to: $dest"
