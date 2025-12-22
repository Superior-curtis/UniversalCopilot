# Builds the project, starts the binary with logs redirected, runs the Notepad demo, and prints last_run.log
Set-StrictMode -Version Latest
$proj = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $proj
Write-Output "Building release..."
cargo build --release

$exe = Join-Path (Get-Location) 'target\release\universal_copilot.exe'
$log = Join-Path (Get-Location) 'last_run.log'
if (-not (Test-Path $exe)) {
    Write-Error "Executable not found: $exe"
    exit 1
}
Remove-Item -LiteralPath $log -ErrorAction SilentlyContinue

Write-Output "Starting binary (no stdout redirection)"
try {
    # If the user has a per-user config.ini in %LOCALAPPDATA%\UniversalCopilot, load its API key into the environment
    $cfgPath = Join-Path $env:LOCALAPPDATA 'UniversalCopilot\config.ini'
    if (Test-Path $cfgPath) {
        $cfg = Get-Content $cfgPath -Raw
        if ($cfg -match 'VITE_OPENAI_API_KEY=(.+?)(\r?\n|$)') {
            $rawKey = $matches[1]
            # Remove all whitespace and control characters, keep only printable ASCII
            $cleanKey = $rawKey -replace '\s+', '' -replace '[^\x20-\x7E]', ''
            $env:VITE_OPENAI_API_KEY = $cleanKey
            Write-Output "Loaded API key from per-user config (raw: $($rawKey.Length) chars, clean: $($cleanKey.Length) chars)"
        }
    }

    Start-Process -FilePath $exe -WindowStyle Hidden -PassThru | Out-Null
} catch {
    Write-Error "Failed to start process: $_"
    exit 1
}

# Give the process a moment to initialize
Start-Sleep -Milliseconds 500

Write-Output "Running focused Notepad demo (will open Notepad and type)..."
& "$proj\focus_and_simulate.ps1"

# Allow some time for app to initialize and write its per-user log
Start-Sleep -Seconds 2
$userLog = Join-Path $env:LOCALAPPDATA 'UniversalCopilot\last_run.log'
if (Test-Path $userLog) {
    Write-Output "--- last_run.log (%LOCALAPPDATA%\\UniversalCopilot) ---"
    Get-Content -LiteralPath $userLog -Raw
    Write-Output "--- end log ---"
} else {
    Write-Output "No per-user log file generated ($userLog)"
}
