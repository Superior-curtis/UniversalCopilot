using System;
using System.Text;
using System.Runtime.InteropServices;

public class WinEnum {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    public static string FindOverlayWindows(string targetClass) {
        StringBuilder result = new StringBuilder();
        EnumWindows(delegate(IntPtr hWnd, IntPtr lParam) {
            var cls = new StringBuilder(256);
            GetClassName(hWnd, cls, cls.Capacity);
            if (cls.ToString() == targetClass) {
                var title = new StringBuilder(256);
                GetWindowText(hWnd, title, title.Capacity);
                result.AppendLine("HWND=" + hWnd.ToInt64() + " Class=" + cls.ToString() + " Title=" + title.ToString());
            }
            return true;
        }, IntPtr.Zero);
        return result.ToString();
    }
}
