use base64::Engine;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::io::Cursor;
use win_screenshot::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ScreenShot {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WindowInfo {
    pub title: String,
    pub process_id: u32,
}

pub struct ScreenshotCapture;

impl ScreenshotCapture {
    /// Get all window information
    pub fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
        let windows = window_list().map_err(|e| format!("Failed to get windows: {:?}", e))?;

        let window_infos: Vec<WindowInfo> = windows
            .iter()
            .filter(|w| !w.window_name.is_empty())
            .map(|w| WindowInfo {
                title: w.window_name.clone(),
                process_id: w.hwnd as u32, // Using hwnd as identifier since it's unique
            })
            .collect();

        tracing::debug!("Found {} valid windows", window_infos.len());
        Ok(window_infos)
    }

    /// Capture entire screen
    pub fn capture_display() -> Result<ScreenShot, String> {
        let buf = capture_display()
            .map_err(|e| format!("Failed to capture display: {:?}", e))?;
        
        Self::encode_buffer_to_base64(buf)
    }

    /// Capture window by process ID (hwnd)
    pub fn capture_by_window_id(window_id: u32) -> Result<ScreenShot, String> {
        let buf = capture_window(window_id as isize)
            .map_err(|e| format!("Failed to capture window {}: {:?}", window_id, e))?;
        
        Self::encode_buffer_to_base64(buf)
    }

    /// Find and capture window by name (exact match)
    pub fn capture_by_window_name(window_name: &str) -> Result<ScreenShot, String> {
        let hwnd = find_window(window_name)
            .map_err(|e| format!("Failed to find window '{}': {:?}", window_name, e))?;
        
        let buf = capture_window(hwnd)
            .map_err(|e| format!("Failed to capture window '{}': {:?}", window_name, e))?;
        
        Self::encode_buffer_to_base64(buf)
    }

    /// Find and capture window by regex pattern
    pub fn capture_by_window_pattern(pattern: &str) -> Result<ScreenShot, String> {
        use regex::Regex;
        
        let re = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
        
        let windows = window_list()
            .map_err(|e| format!("Failed to get window list: {:?}", e))?;
        
        let window = windows
            .iter()
            .find(|w| re.is_match(&w.window_name))
            .ok_or_else(|| format!("No window found matching pattern '{}'", pattern))?;
        
        let buf = capture_window(window.hwnd)
            .map_err(|e| format!("Failed to capture window matching '{}': {:?}", pattern, e))?;
        
        Self::encode_buffer_to_base64(buf)
    }

    /// Advanced window capture with fine-tuning options
    pub fn capture_window_advanced(
        window_id: u32,
        use_bitblt: bool,
        client_only: bool,
        crop_xy: Option<[i32; 2]>,
        crop_wh: Option<[i32; 2]>,
    ) -> Result<ScreenShot, String> {
        let using = if use_bitblt { Using::BitBlt } else { Using::PrintWindow };
        let area = if client_only { Area::ClientOnly } else { Area::Full };
        
        let buf = capture_window_ex(
            window_id as isize,
            using,
            area,
            crop_xy,
            crop_wh,
        ).map_err(|e| format!("Failed to capture window with advanced options: {:?}", e))?;
        
        Self::encode_buffer_to_base64(buf)
    }

    /// Encode screenshot buffer to base64
    fn encode_buffer_to_base64(buf: RgbBuf) -> Result<ScreenShot, String> {
        let width = buf.width;
        let height = buf.height;
        
        // Use the original pixels directly without color channel conversion
        let rgba_image = RgbaImage::from_raw(width, height, buf.pixels)
            .ok_or_else(|| "Failed to create RGBA image from buffer".to_string())?;

        let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);

        // Convert image to PNG bytes
        let mut png_bytes = Vec::new();
        dynamic_image
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;

        // Convert to base64
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        Ok(ScreenShot {
            image_base64,
            width,
            height,
            format: "png".to_string(),
        })
    }

    /// Create a demo screenshot (for testing purposes)
    pub fn create_demo_screenshot() -> Result<ScreenShot, String> {
        // Create a simple 100x100 red rectangle as a demo
        let width = 100;
        let height = 100;
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        
        // Fill with red color (RGBA)
        for _ in 0..(width * height) {
            data.extend_from_slice(&[255, 0, 0, 255]); // Red with full alpha
        }
        
        Self::encode_data_to_base64(&data, width, height)
    }

    /// Encode raw data to base64
    fn encode_data_to_base64(data: &[u8], width: u32, height: u32) -> Result<ScreenShot, String> {
        let rgba_image = image::ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or_else(|| "Failed to create image from buffer".to_string())?;

        let dynamic_image = image::DynamicImage::ImageRgba8(rgba_image);

        // Convert image to PNG bytes
        let mut png_bytes = Vec::new();
        dynamic_image
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image as PNG: {}", e))?;

        // Convert to base64
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

        Ok(ScreenShot {
            image_base64,
            width,
            height,
            format: "png".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_windows() {
        let result = ScreenshotCapture::get_all_windows();
        assert!(result.is_ok());
        let windows = result.unwrap();
        println!("Found {} windows", windows.len());
        
        // Print first few windows for debugging
        for (i, window) in windows.iter().take(5).enumerate() {
            println!("Window {}: {} (ID: {})", i + 1, window.title, window.process_id);
        }
    }

    #[test]
    fn test_capture_display() {
        let result = ScreenshotCapture::capture_display();
        assert!(result.is_ok());
        let screenshot = result.unwrap();
        
        assert!(screenshot.width > 0);
        assert!(screenshot.height > 0);
        assert_eq!(screenshot.format, "png");
        assert!(!screenshot.image_base64.is_empty());
        println!("Display screenshot captured: {}x{} {}", 
                screenshot.width, screenshot.height, screenshot.format);
    }

    #[test]
    fn test_demo_screenshot() {
        let result = ScreenshotCapture::create_demo_screenshot();
        assert!(result.is_ok());
        let screenshot = result.unwrap();
        
        assert_eq!(screenshot.width, 100);
        assert_eq!(screenshot.height, 100);
        assert_eq!(screenshot.format, "png");
        assert!(!screenshot.image_base64.is_empty());
        println!("Demo screenshot created successfully: {}x{} {}", 
                screenshot.width, screenshot.height, screenshot.format);
    }

    #[test]
    fn test_window_info_serialization() {
        let window_info = WindowInfo {
            title: "Test Window".to_string(),
            process_id: 1234,
        };
        
        let json = serde_json::to_string(&window_info).unwrap();
        println!("WindowInfo JSON: {}", json);
        
        let parsed: WindowInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Test Window");
        assert_eq!(parsed.process_id, 1234);
    }

    #[test]
    fn test_capture_by_window_name() {
        // This test will likely fail if no notepad window is open, but it tests the function signature
        match ScreenshotCapture::capture_by_window_name("Notepad") {
            Ok(screenshot) => {
                println!("Successfully captured Notepad window: {}x{}", 
                        screenshot.width, screenshot.height);
            }
            Err(e) => {
                println!("Expected error when Notepad is not open: {}", e);
            }
        }
    }

    #[test] 
    fn test_capture_advanced_options() {
        // Test advanced capture options with a fake window ID
        match ScreenshotCapture::capture_window_advanced(
            12345, // fake window ID
            true,  // use BitBlt
            false, // capture full window
            None,  // no crop xy
            None,  // no crop wh
        ) {
            Ok(screenshot) => {
                println!("Advanced capture successful: {}x{}", 
                        screenshot.width, screenshot.height);
            }
            Err(e) => {
                println!("Expected error with fake window ID: {}", e);
            }
        }
    }
}
