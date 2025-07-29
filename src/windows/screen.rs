use std::mem;
use anyhow::{anyhow, Result};
use tracing::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
        UI::HiDpi::*,
    },
};

/// Window screenshot result
#[derive(Debug)]
pub struct ScreenshotResult {
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: HWND,
    pub title: String,
    pub process_id: u32,
    pub class_name: String,
}

/// Window screenshot utility
pub struct WindowScreenshot;

impl WindowScreenshot {
    /// Initialize DPI awareness for high-DPI displays
    fn init_dpi_awareness() {
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
    }

    /// Capture screenshot of a window by handle
    pub fn capture_window(hwnd: HWND) -> Result<ScreenshotResult> {
        unsafe {
            // Initialize DPI awareness
            Self::init_dpi_awareness();

            // Check if window is valid
            if !IsWindow(Some(hwnd)).as_bool() {
                return Err(anyhow!("Invalid window handle"));
            }

            // Get window device context
            let window_dc = GetDC(Some(hwnd));
            if window_dc.is_invalid() {
                return Err(anyhow!("Failed to get window device context"));
            }

            // Create compatible device context
            let memory_dc = CreateCompatibleDC(Some(window_dc));
            if memory_dc.is_invalid() {
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err(anyhow!("Failed to create compatible device context"));
            }

            // Get window rect (use GetWindowRect for full window including borders)
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect)?;

            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            if width <= 0 || height <= 0 {
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err(anyhow!("Invalid window dimensions: {}x{}", width, height));
            }

            // Create compatible bitmap
            let bitmap = CreateCompatibleBitmap(window_dc, width, height);
            if bitmap.is_invalid() {
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(Some(hwnd), window_dc);
                return Err(anyhow!("Failed to create compatible bitmap"));
            }

            // Select bitmap into memory device context
            let old_bitmap = SelectObject(memory_dc, HGDIOBJ(bitmap.0));

            // Copy window content to memory device context
            BitBlt(memory_dc, 0, 0, width, height, Some(window_dc), 0, 0, SRCCOPY)?;

            // Get bitmap data
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Negative for top-down bitmap (correct orientation)
                    biPlanes: 1,
                    biBitCount: 32, // 32-bit RGBA
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            // Calculate image data size
            let data_size = (width * height * 4) as usize;
            let mut image_data = vec![0u8; data_size];

            // Get bitmap data
            let result = GetDIBits(
                memory_dc,
                bitmap,
                0,
                height as u32,
                Some(image_data.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );

            // Clean up resources
            let _ = SelectObject(memory_dc, old_bitmap);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(Some(hwnd), window_dc);

            if result == 0 {
                return Err(anyhow!("Failed to get bitmap data"));
            }

            info!("Successfully captured window screenshot: {}x{}", width, height);

            Ok(ScreenshotResult {
                width,
                height,
                data: image_data,
            })
        }
    }

    /// Capture screenshot by window title
    pub fn capture_window_by_title(window_title: &str) -> Result<ScreenshotResult> {
        let hwnd = Self::find_window_by_title(window_title)?;
        Self::capture_window(hwnd)
    }

    /// Capture screenshot by process ID
    pub fn capture_window_by_pid(process_id: u32) -> Result<ScreenshotResult> {
        let hwnd = Self::find_main_window_by_pid(process_id)?;
        Self::capture_window(hwnd)
    }

    /// Capture entire screen
    pub fn capture_screen() -> Result<ScreenshotResult> {
        unsafe {
            // Initialize DPI awareness
            Self::init_dpi_awareness();

            // Get screen dimensions
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);

            // Get desktop window
            let desktop_dc = GetDC(None);
            if desktop_dc.is_invalid() {
                return Err(anyhow!("Failed to get desktop device context"));
            }

            // Create compatible device context
            let memory_dc = CreateCompatibleDC(Some(desktop_dc));
            if memory_dc.is_invalid() {
                let _ = ReleaseDC(None, desktop_dc);
                return Err(anyhow!("Failed to create compatible device context"));
            }

            // Create compatible bitmap
            let bitmap = CreateCompatibleBitmap(desktop_dc, screen_width, screen_height);
            if bitmap.is_invalid() {
                let _ = DeleteDC(memory_dc);
                let _ = ReleaseDC(None, desktop_dc);
                return Err(anyhow!("Failed to create compatible bitmap"));
            }

            // Select bitmap into memory device context
            let old_bitmap = SelectObject(memory_dc, HGDIOBJ(bitmap.0));

            // Copy screen to memory device context
            BitBlt(memory_dc, 0, 0, screen_width, screen_height, Some(desktop_dc), 0, 0, SRCCOPY)?;

            // Get bitmap data
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: screen_width,
                    biHeight: -screen_height, // Negative for top-down bitmap (correct orientation)
                    biPlanes: 1,
                    biBitCount: 32, // 32-bit RGBA
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            // Calculate image data size
            let data_size = (screen_width * screen_height * 4) as usize;
            let mut image_data = vec![0u8; data_size];

            // Get bitmap data
            let result = GetDIBits(
                memory_dc,
                bitmap,
                0,
                screen_height as u32,
                Some(image_data.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );

            // Clean up resources
            let _ = SelectObject(memory_dc, old_bitmap);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
            let _ = ReleaseDC(None, desktop_dc);

            if result == 0 {
                return Err(anyhow!("Failed to get bitmap data"));
            }

            info!("Successfully captured screen screenshot: {}x{}", screen_width, screen_height);

            Ok(ScreenshotResult {
                width: screen_width,
                height: screen_height,
                data: image_data,
            })
        }
    }

    /// Find window by title
    fn find_window_by_title(window_title: &str) -> Result<HWND> {
        unsafe {
            let title_wide: Vec<u16> = window_title.encode_utf16().chain(std::iter::once(0)).collect();
            let hwnd = FindWindowW(PCWSTR::null(), PCWSTR::from_raw(title_wide.as_ptr()))?;
            
            if hwnd.0.is_null() {
                return Err(anyhow!("Window not found: {}", window_title));
            }

            Ok(hwnd)
        }
    }

    /// Find main window by process ID
    fn find_main_window_by_pid(process_id: u32) -> Result<HWND> {
        unsafe {
            let mut result = HWND::default();
            let mut data = FindWindowData {
                process_id,
                hwnd: &mut result,
                found: false,
            };

            EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut data as *mut _ as isize),
            )?;

            if !data.found {
                return Err(anyhow!("Main window not found for process ID: {}", process_id));
            }

            Ok(result)
        }
    }

    /// List all visible windows with their titles and process IDs
    pub fn list_windows() -> Result<Vec<WindowInfo>> {
        unsafe {
            let mut windows = Vec::new();
            let mut data = EnumWindowsData {
                windows: &mut windows,
            };

            EnumWindows(
                Some(enum_all_windows_proc),
                LPARAM(&mut data as *mut _ as isize),
            )?;

            Ok(windows)
        }
    }

    /// Find windows by partial title match
    pub fn find_windows_by_partial_title(partial_title: &str) -> Result<Vec<WindowInfo>> {
        let all_windows = Self::list_windows()?;
        let matching_windows: Vec<WindowInfo> = all_windows
            .into_iter()
            .filter(|window| window.title.to_lowercase().contains(&partial_title.to_lowercase()))
            .collect();
        
        Ok(matching_windows)
    }

    /// Capture screenshot by partial window title match (captures the first match)
    pub fn capture_window_by_partial_title(partial_title: &str) -> Result<ScreenshotResult> {
        let matching_windows = Self::find_windows_by_partial_title(partial_title)?;
        
        if matching_windows.is_empty() {
            return Err(anyhow!("No windows found containing title: {}", partial_title));
        }
        
        let window = &matching_windows[0];
        info!("Found window: '{}' (PID: {})", window.title, window.process_id);
        Self::capture_window(window.hwnd)
    }
    pub fn save_to_bmp(screenshot: &ScreenshotResult, file_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(file_path)?;

        // BMP file header
        let file_size = 14 + 40 + screenshot.data.len(); // File header + Info header + Image data
        let bmp_file_header = [
            0x42, 0x4D, // "BM"
            (file_size & 0xFF) as u8,
            ((file_size >> 8) & 0xFF) as u8,
            ((file_size >> 16) & 0xFF) as u8,
            ((file_size >> 24) & 0xFF) as u8,
            0, 0, 0, 0, // Reserved fields
            54, 0, 0, 0, // Image data offset
        ];

        // BMP info header
        let bmp_info_header = [
            40, 0, 0, 0, // Info header size
            (screenshot.width & 0xFF) as u8,
            ((screenshot.width >> 8) & 0xFF) as u8,
            ((screenshot.width >> 16) & 0xFF) as u8,
            ((screenshot.width >> 24) & 0xFF) as u8,
            (screenshot.height & 0xFF) as u8,
            ((screenshot.height >> 8) & 0xFF) as u8,
            ((screenshot.height >> 16) & 0xFF) as u8,
            ((screenshot.height >> 24) & 0xFF) as u8,
            1, 0, // Color planes
            32, 0, // Bits per pixel
            0, 0, 0, 0, // Compression type
            (screenshot.data.len() & 0xFF) as u8,
            ((screenshot.data.len() >> 8) & 0xFF) as u8,
            ((screenshot.data.len() >> 16) & 0xFF) as u8,
            ((screenshot.data.len() >> 24) & 0xFF) as u8,
            0, 0, 0, 0, // X pixels per meter
            0, 0, 0, 0, // Y pixels per meter
            0, 0, 0, 0, // Color indices used
            0, 0, 0, 0, // Important color indices
        ];

        file.write_all(&bmp_file_header)?;
        file.write_all(&bmp_info_header)?;
        file.write_all(&screenshot.data)?;

        info!("Screenshot saved to: {}", file_path);
        Ok(())
    }
}

// Helper structure for finding windows
struct FindWindowData {
    process_id: u32,
    hwnd: *mut HWND,
    found: bool,
}

// Helper structure for enumerating all windows
struct EnumWindowsData {
    windows: *mut Vec<WindowInfo>,
}

// Callback function for enumerating windows
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut FindWindowData);
    
    let mut window_process_id = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut window_process_id));
    
    if window_process_id == data.process_id {
        // Check if it's a main window (visible and has a title)
        if IsWindowVisible(hwnd).as_bool() {
            let title_length = GetWindowTextLengthW(hwnd);
            if title_length > 0 {
                // Additional check: make sure it's not a child window
                match GetParent(hwnd) {
                    Ok(parent) if parent.0.is_null() => {
                        *data.hwnd = hwnd;
                        data.found = true;
                        return FALSE; // Stop enumerating
                    }
                    Err(_) => {
                        // No parent, this is a top-level window
                        *data.hwnd = hwnd;
                        data.found = true;
                        return FALSE; // Stop enumerating
                    }
                    _ => {} // Has parent, continue searching
                }
            }
        }
    }
    
    TRUE // Continue enumerating
}

// Callback function for enumerating all windows
unsafe extern "system" fn enum_all_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumWindowsData);
    let windows = &mut *data.windows;
    
    // Only include visible windows
    if IsWindowVisible(hwnd).as_bool() {
        let title_length = GetWindowTextLengthW(hwnd);
        if title_length > 0 {
            // Get window title
            let mut title_buffer = vec![0u16; (title_length + 1) as usize];
            let actual_length = GetWindowTextW(hwnd, &mut title_buffer);
            
            if actual_length > 0 {
                // Convert to String, removing null terminator
                title_buffer.truncate(actual_length as usize);
                let title = String::from_utf16_lossy(&title_buffer);
                
                // Get process ID
                let mut process_id = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut process_id));
                
                // Get class name
                let mut class_buffer = vec![0u16; 256];
                let class_length = GetClassNameW(hwnd, &mut class_buffer);
                let class_name = if class_length > 0 {
                    class_buffer.truncate(class_length as usize);
                    String::from_utf16_lossy(&class_buffer)
                } else {
                    String::new()
                };
                
                // Only add top-level windows (no parent)
                match GetParent(hwnd) {
                    Ok(parent) if parent.0.is_null() => {
                        windows.push(WindowInfo {
                            hwnd,
                            title,
                            process_id,
                            class_name,
                        });
                    }
                    Err(_) => {
                        // No parent, this is a top-level window
                        windows.push(WindowInfo {
                            hwnd,
                            title,
                            process_id,
                            class_name,
                        });
                    }
                    _ => {} // Has parent, skip
                }
            }
        }
    }
    
    TRUE // Continue enumerating
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_screen() {
        let result = WindowScreenshot::capture_screen();
        assert!(result.is_ok());
        
        let screenshot = result.unwrap();
        assert!(screenshot.width > 0);
        assert!(screenshot.height > 0);
        assert!(!screenshot.data.is_empty());
    }
}