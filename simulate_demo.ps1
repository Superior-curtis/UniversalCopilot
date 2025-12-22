param(
    [string]$Text = "In this paper, we propose a novel method for",
    [int]$CharDelayMs = 60,
    [int]$PauseBeforeAcceptMs = 800,
    [switch]$Accept
)

$src = @"
using System;
using System.Runtime.InteropServices;

public static class InputHelper {
    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT {
        public UInt32 type;
        public InputUnion u;
    }
    [StructLayout(LayoutKind.Explicit)]
    public struct InputUnion {
        [FieldOffset(0)] public KEYBDINPUT ki;
    }
    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT {
        public UInt16 wVk;
        public UInt16 wScan;
        public UInt32 dwFlags;
        public UInt32 time;
        public IntPtr dwExtraInfo;
    }

    [DllImport("user32.dll", SetLastError=true)]
    public static extern UInt32 SendInput(UInt32 nInputs, INPUT[] pInputs, Int32 cbSize);

    public const UInt32 INPUT_KEYBOARD = 1;
    public const UInt32 KEYEVENTF_KEYUP = 0x0002;
    public const UInt32 KEYEVENTF_UNICODE = 0x0004;

    public static void SendUnicodeChar(char ch) {
        INPUT down = new INPUT();
        down.type = INPUT_KEYBOARD;
        down.u.ki.wVk = 0;
        down.u.ki.wScan = (UInt16)ch;
        down.u.ki.dwFlags = KEYEVENTF_UNICODE;
        down.u.ki.time = 0;
        down.u.ki.dwExtraInfo = IntPtr.Zero;

        INPUT up = down;
        up.u.ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

        INPUT[] inputs = new INPUT[] { down, up };
        SendInput((UInt32)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)));
    }

    public static void SendVk(ushort vk) {
        INPUT down = new INPUT();
        down.type = INPUT_KEYBOARD;
        down.u.ki.wVk = vk;
        down.u.ki.wScan = 0;
        down.u.ki.dwFlags = 0;
        down.u.ki.time = 0;
        down.u.ki.dwExtraInfo = IntPtr.Zero;

        INPUT up = down;
        up.u.ki.dwFlags = KEYEVENTF_KEYUP;

        INPUT[] inputs = new INPUT[] { down, up };
        SendInput((UInt32)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)));
    }
}
"@

Add-Type -TypeDefinition $src -Language CSharp -ErrorAction Stop

Write-Output "Demo will start in 5 seconds. Focus the target window now and save work."
for ($i=5; $i -ge 1; $i--) { Write-Host "$i..."; Start-Sleep -Milliseconds 1000 }

foreach ($ch in $Text.ToCharArray()) {
    [InputHelper]::SendUnicodeChar($ch)
    Start-Sleep -Milliseconds $CharDelayMs
}

if ($Accept) {
    Start-Sleep -Milliseconds $PauseBeforeAcceptMs
    # send TAB (VK_TAB = 0x09)
    [InputHelper]::SendVk(0x09)
}

Write-Output "Simulation complete."
