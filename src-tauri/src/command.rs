use crate::windows::{
    ace_tools::ProcessInfo,
    screenshot::{ScreenShot, ScreenshotCapture, WindowInfo},
    AceProcessControllerState,
};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn greet(name: &str) -> String {
    let message = format!("Hello, {}! You've been greeted from Rust!", name);
    tracing::debug!("greet called with name: {}", name);
    message
}

#[tauri::command]
#[specta::specta]
pub fn is_running_as_admin() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let result = crate::windows::utils::is_running_as_admin().map_err(|e| e.to_string());

        tracing::debug!("get is running as admin: {:?}", result);

        result
    }

    #[cfg(not(target_os = "windows"))]
    {
        tracing::debug!("get is running as admin: false (not implemented on this OS)");
        Ok(false)
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_all_ace_guard_processes(
    state: State<'_, AceProcessControllerState>,
) -> Result<Vec<ProcessInfo>, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire controller lock: {}", e))?;

    let result = guard.scan_ace_guard_processes();
    
    result
}

#[tauri::command]
#[specta::specta]
pub async fn optimize_all_ace_guard_processes(
    state: State<'_, AceProcessControllerState>,
) -> Result<String, String> {
    // Clone the controller to avoid holding the lock across await
    let mut controller = {
        let guard = state
            .0
            .lock()
            .map_err(|e| format!("Failed to acquire controller lock: {}", e))?;
        (*guard).clone()
    };

    let result = controller.optimize_ace_guard_processes().await;
    
    // Update the global state with the modified controller
    {
        let mut guard = state
            .0
            .lock()
            .map_err(|e| format!("Failed to acquire controller lock: {}", e))?;
        *guard = controller;
    }
    
    tracing::debug!("Optimization result: {:?}", result);
    result
}

#[tauri::command]
#[specta::specta]
pub fn get_controller_privileges_status(
    state: State<'_, AceProcessControllerState>,
) -> Result<bool, String> {
    let controller = state
        .0
        .lock()
        .map_err(|e| format!("Failed to acquire controller lock: {}", e))?;

    Ok(controller.get_privileges_enabled())
}

#[tauri::command]
#[specta::specta]
pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    ScreenshotCapture::get_all_windows()
}

#[tauri::command]
#[specta::specta]
pub fn try_capture_image_by_window_id(window_id: u32) -> Result<ScreenShot, String> {
    ScreenshotCapture::capture_by_window_id(window_id)
}
