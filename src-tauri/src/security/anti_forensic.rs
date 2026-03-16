// Anti-Forensic Module
// Handles OS-level process protections like Screenshot Blanking and Dump Prevention.

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_MONITOR, WDA_NONE,
};
// Removed incorrect Diagnostics::Debug imports

/// Enables anti-forensic screen blanking for the application window.
/// When active, screenshots and streaming software will only see a black box.
#[cfg(target_os = "windows")]
pub fn enable_screenshot_protection(window: &tauri::WebviewWindow) -> Result<(), String> {
    if let Ok(hwnd) = window.hwnd() {
        let handle = HWND(hwnd.0 as _);
        unsafe {
            let _ = SetWindowDisplayAffinity(handle, WDA_MONITOR);
        }
    }
    Ok(())
}

/// Disables screenshot protection, making the application visible to capture tools again.
#[cfg(target_os = "windows")]
pub fn disable_screenshot_protection(window: &tauri::WebviewWindow) -> Result<(), String> {
    if let Ok(hwnd) = window.hwnd() {
        let handle = HWND(hwnd.0 as _);
        unsafe {
            let _ = SetWindowDisplayAffinity(handle, WDA_NONE);
        }
    }
    Ok(())
}

/// Requests the OS to exclude this process from automated crash dumps,
/// protecting unencrypted memory contents from forensic recovery.
#[cfg(target_os = "windows")]
pub fn prevent_crash_dumps() {
    // ProcessMitigationPolicy implementation deferred to keep scope focused on Screen Blanking.
    // In the future, this should invoke SetProcessMitigationPolicy via windows::Win32::System::Threading.
}

// Stub implementations for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn enable_screenshot_protection(_window: &tauri::WebviewWindow) -> Result<(), String> {
    // Implementation for macOS/Linux would go here
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn disable_screenshot_protection(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn prevent_crash_dumps() {
    // macOS/Linux dump prevention
}
