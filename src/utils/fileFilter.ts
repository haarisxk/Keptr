/**
 * Client-side file extension blocklist.
 * Mirrors the server-side BLOCKED_EXTENSIONS in src-tauri/src/commands/files.rs.
 * Prevents users from importing potentially dangerous file types into the vault.
 */

const BLOCKED_EXTENSIONS = new Set([
    // Windows executables & installers
    "exe", "msi", "bat", "cmd", "com", "scr", "pif",
    // Script files
    "ps1", "psm1", "psd1", "vbs", "vbe", "js", "jse", "ws", "wsf", "wsc", "wsh",
    // Shell / Unix
    "sh", "bash", "csh", "ksh",
    // Compiled / bytecode
    "dll", "sys", "drv", "ocx", "cpl",
    // Macro-enabled Office (can contain malicious macros)
    "docm", "xlsm", "pptm", "dotm", "xltm", "potm",
    // Java / .NET
    "jar", "class", "msp", "mst",
    // Shortcuts & links
    "lnk", "url", "scf",
    // Registry & config
    "reg", "inf",
    // Disk images (can contain anything)
    "iso", "img", "vhd", "vhdx",
    // Other risky
    "appx", "msix", "appxbundle", "cab",
]);

/**
 * Returns true if the given filename has a blocked (dangerous) extension.
 */
export function isBlockedExtension(fileName: string): boolean {
    const ext = fileName.split(".").pop()?.toLowerCase() ?? "";
    return BLOCKED_EXTENSIONS.has(ext);
}
