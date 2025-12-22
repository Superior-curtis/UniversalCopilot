# Open Notepad, focus it, then run the simulate demo
Start-Process notepad.exe
Start-Sleep -Milliseconds 800
$p = Get-Process -Name notepad | Select-Object -First 1
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
}
"@
[Win]::SetForegroundWindow($p.MainWindowHandle)
Start-Sleep -Milliseconds 300
# Run the simulator to type into the focused window and accept suggestion
& "C:\Users\huach\Downloads\Code\UniversalCopilot\simulate_demo.ps1" -Text "Earlier line: This is the setup sentence. Now continue with an additional sentence to see context-aware suggestions." -CharDelayMs 60 -PauseBeforeAcceptMs 800 -Accept
Write-Output "Focus-and-simulate completed." 
