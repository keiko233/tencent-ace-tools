use image::ImageFormat;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OcrRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub region: OcrRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OcrResponse {
    pub results: Vec<OcrResult>,
    pub full_text: String,
    pub success: bool,
}

/// OCR screen region recognition
pub fn ocr_screen_region(region: OcrRegion) -> Result<OcrResponse, String> {
    tracing::debug!("OCR screen region: {:?}", region);

    // Capture full screen first
    let screenshot = crate::windows::screenshot::ScreenshotCapture::capture_display()
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    // Use PNG binary data directly
    let image_data = &screenshot.image_data;

    // Load image
    let img =
        image::load_from_memory(image_data).map_err(|e| format!("Failed to load image: {}", e))?;

    // Crop specified region
    let cropped = img.crop_imm(
        region.x as u32,
        region.y as u32,
        region.width as u32,
        region.height as u32,
    );

    // Convert to RGBA format and save as PNG in memory
    let rgba_img = cropped.to_rgba8();
    let mut png_data = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_data);
        rgba_img
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    // Create a temporary file path for oneocr
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("ocr_temp_{}.png", std::process::id()));
    std::fs::write(&temp_file, &png_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Perform OCR using oneocr
    let engine =
        oneocr_rs::OcrEngine::new().map_err(|e| format!("Failed to create OCR engine: {}", e))?;

    let ocr_result = engine
        .run(oneocr_rs::ImageInput::FilePath(temp_file.clone()))
        .map_err(|e| format!("OCR failed: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    // Convert result format
    let mut results = Vec::new();
    let mut full_text = String::new();

    for line in &ocr_result.lines {
        let text = line.text.clone();

        let bbox = &line.bounding_box;
        let region = OcrRegion {
            x: region.x + bbox.top_left.x as i32,
            y: region.y + bbox.top_left.y as i32,
            width: (bbox.top_right.x - bbox.top_left.x) as i32,
            height: (bbox.bottom_left.y - bbox.top_left.y) as i32,
        };

        if !text.is_empty() {
            if !full_text.is_empty() {
                full_text.push('\n');
            }
            full_text.push_str(&text);

            results.push(OcrResult {
                text,
                confidence: 1.0, // Default confidence since oneocr doesn't provide per-line confidence
                region,
            });
        }
    }

    tracing::debug!("OCR completed, found {} results", results.len());

    Ok(OcrResponse {
        results,
        full_text,
        success: true,
    })
}

/// OCR image region recognition (accepts PNG binary data)
pub fn ocr_image_region(image_data: &[u8], region: OcrRegion) -> Result<OcrResponse, String> {
    tracing::debug!("OCR image region: {:?}", region);

    // Load image from binary data
    let img =
        image::load_from_memory(image_data).map_err(|e| format!("Failed to load image: {}", e))?;

    // Check if region is within image bounds
    let img_width = img.width() as i32;
    let img_height = img.height() as i32;

    if region.x < 0
        || region.y < 0
        || region.x + region.width > img_width
        || region.y + region.height > img_height
    {
        return Err(format!(
            "Region out of bounds: image size {}x{}, requested region {}x{} at ({}, {})",
            img_width, img_height, region.width, region.height, region.x, region.y
        ));
    }

    // Crop specified region
    let cropped = img.crop_imm(
        region.x as u32,
        region.y as u32,
        region.width as u32,
        region.height as u32,
    );

    // Convert to RGBA format and save as PNG in memory
    let rgba_img = cropped.to_rgba8();
    let mut png_data = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_data);
        rgba_img
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    }

    // Create a temporary file path for oneocr
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("ocr_temp_{}.png", std::process::id()));
    std::fs::write(&temp_file, &png_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Perform OCR using oneocr
    let engine =
        oneocr_rs::OcrEngine::new().map_err(|e| format!("Failed to create OCR engine: {}", e))?;

    let ocr_result = engine
        .run(oneocr_rs::ImageInput::FilePath(temp_file.clone()))
        .map_err(|e| format!("OCR failed: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    // Convert result format
    let mut results = Vec::new();
    let mut full_text = String::new();

    for line in &ocr_result.lines {
        let text = line.text.clone();

        let bbox = &line.bounding_box;
        let region = OcrRegion {
            x: region.x + bbox.top_left.x as i32,
            y: region.y + bbox.top_left.y as i32,
            width: (bbox.top_right.x - bbox.top_left.x) as i32,
            height: (bbox.bottom_left.y - bbox.top_left.y) as i32,
        };

        if !text.is_empty() {
            if !full_text.is_empty() {
                full_text.push('\n');
            }
            full_text.push_str(&text);

            results.push(OcrResult {
                text,
                confidence: 1.0, // Default confidence since oneocr doesn't provide per-line confidence
                region,
            });
        }
    }

    tracing::debug!("OCR completed, found {} results", results.len());

    Ok(OcrResponse {
        results,
        full_text,
        success: true,
    })
}

/// OCR full screen recognition
pub fn ocr_full_screen() -> Result<OcrResponse, String> {
    tracing::debug!("OCR full screen");

    // Capture full screen
    let screenshot = crate::windows::screenshot::ScreenshotCapture::capture_display()
        .map_err(|e| format!("Screenshot failed: {}", e))?;

    // Use PNG binary data directly
    let image_data = &screenshot.image_data;

    // Create a temporary file path for oneocr
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("ocr_temp_{}.png", std::process::id()));
    std::fs::write(&temp_file, &image_data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Perform OCR using oneocr
    let engine =
        oneocr_rs::OcrEngine::new().map_err(|e| format!("Failed to create OCR engine: {}", e))?;

    let ocr_result = engine
        .run(oneocr_rs::ImageInput::FilePath(temp_file.clone()))
        .map_err(|e| format!("OCR failed: {}", e))?;

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    // Convert result format
    let mut results = Vec::new();
    let mut full_text = String::new();

    for line in &ocr_result.lines {
        let text = line.text.clone();

        let bbox = &line.bounding_box;
        let region = OcrRegion {
            x: bbox.top_left.x as i32,
            y: bbox.top_left.y as i32,
            width: (bbox.top_right.x - bbox.top_left.x) as i32,
            height: (bbox.bottom_left.y - bbox.top_left.y) as i32,
        };

        if !text.is_empty() {
            if !full_text.is_empty() {
                full_text.push('\n');
            }
            full_text.push_str(&text);

            results.push(OcrResult {
                text,
                confidence: 1.0, // Default confidence since oneocr doesn't provide per-line confidence
                region,
            });
        }
    }

    tracing::debug!("OCR completed, found {} results", results.len());

    Ok(OcrResponse {
        results,
        full_text,
        success: true,
    })
}
