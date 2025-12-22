if (Test-Path "$env:LOCALAPPDATA\UniversalCopilot\config.ini") {
    $c = Get-Content "$env:LOCALAPPDATA\UniversalCopilot\config.ini" -Raw
    if ($c -match 'VITE_OPENAI_API_KEY=') { Write-Output 'API key: present' } else { Write-Output 'API key: missing' }
} else {
    Write-Output 'config.ini: not found'
}
