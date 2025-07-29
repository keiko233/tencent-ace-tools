use anyhow::Result;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tencent_ace_tools::windows::screen::WindowScreenshot;

/// Launch Windows Notepad and wait for it to appear
fn launch_notepad() -> Result<()> {
    println!("Launching Windows Notepad...");

    match Command::new("notepad").spawn() {
        Ok(_) => println!("Notepad launch command executed"),
        Err(e) => return Err(anyhow::anyhow!("Failed to launch notepad: {}", e)),
    }

    // Wait for notepad to appear
    println!("Waiting for Notepad window to appear...");
    for attempt in 1..=10 {
        thread::sleep(Duration::from_millis(500));

        let search_terms = ["notepad", "记事本"];
        for term in &search_terms {
            match WindowScreenshot::find_windows_by_partial_title(term) {
                Ok(windows) => {
                    if !windows.is_empty() {
                        println!("✓ Notepad window found after {} attempts", attempt);
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
        }

        println!("Attempt {}/10: Notepad not found yet...", attempt);
    }

    Err(anyhow::anyhow!(
        "Notepad window did not appear within 5 seconds"
    ))
}

/// Screenshot example program
fn main() -> Result<()> {
    println!("Windows Screenshot Tool Example");

    // Example 1: Capture entire screen
    println!("Capturing entire screen...");
    match WindowScreenshot::capture_screen() {
        Ok(screenshot) => {
            println!(
                "✓ Screen capture successful! Size: {}x{}",
                screenshot.width, screenshot.height
            );

            // Save as BMP file
            if let Err(e) = WindowScreenshot::save_to_bmp(&screenshot, "screen_capture.bmp") {
                println!("❌ Failed to save screen capture: {}", e);
            } else {
                println!("✓ Screen capture saved as screen_capture.bmp");
            }
        }
        Err(e) => println!("❌ Screen capture failed: {}", e),
    }

    // Example 2: List all windows to find Notepad
    println!("\nListing all visible windows...");
    match WindowScreenshot::list_windows() {
        Ok(windows) => {
            println!("Found {} windows:", windows.len());
            for (i, window) in windows.iter().enumerate() {
                if i < 10 {
                    // Show first 10 windows
                    println!(
                        "  {}: '{}' (PID: {}, Class: {})",
                        i + 1,
                        window.title,
                        window.process_id,
                        window.class_name
                    );
                }
            }
            if windows.len() > 10 {
                println!("  ... and {} more windows", windows.len() - 10);
            }
        }
        Err(e) => println!("❌ Failed to list windows: {}", e),
    }

    // Example 3: Find windows containing "notepad" in the title
    println!("\nSearching for windows containing 'notepad'...");
    match WindowScreenshot::find_windows_by_partial_title("notepad") {
        Ok(notepad_windows) => {
            if notepad_windows.is_empty() {
                println!("No Notepad windows found");
            } else {
                println!("Found {} Notepad windows:", notepad_windows.len());
                for window in &notepad_windows {
                    println!(
                        "  '{}' (PID: {}, Class: {})",
                        window.title, window.process_id, window.class_name
                    );
                }

                // Try to capture the first Notepad window
                println!("\nCapturing first Notepad window...");
                match WindowScreenshot::capture_window_by_partial_title("notepad") {
                    Ok(screenshot) => {
                        println!(
                            "✓ Notepad capture successful! Size: {}x{}",
                            screenshot.width, screenshot.height
                        );

                        if let Err(e) =
                            WindowScreenshot::save_to_bmp(&screenshot, "notepad_capture.bmp")
                        {
                            println!("❌ Failed to save Notepad capture: {}", e);
                        } else {
                            println!("✓ Notepad capture saved as notepad_capture.bmp");
                        }
                    }
                    Err(e) => println!("❌ Notepad capture failed: {}", e),
                }
            }
        }
        Err(e) => println!("❌ Failed to search for Notepad windows: {}", e),
    }

    // Example 4: Search for other common applications and try to capture them
    let apps_to_search = [
        "chrome",
        "vscode",
        "cursor",
        "explorer",
        "calculator",
        "cmd",
    ];

    for app in &apps_to_search {
        match WindowScreenshot::find_windows_by_partial_title(app) {
            Ok(windows) => {
                if !windows.is_empty() {
                    println!("\nFound {} window(s) for '{}':", windows.len(), app);
                    for window in windows.iter().take(3) {
                        // Show first 3 matches
                        println!("  '{}' (PID: {})", window.title, window.process_id);
                    }

                    // Try to capture the first window
                    println!("Attempting to capture {} window...", app);
                    match WindowScreenshot::capture_window_by_partial_title(app) {
                        Ok(screenshot) => {
                            println!(
                                "✓ {} capture successful! Size: {}x{}",
                                app, screenshot.width, screenshot.height
                            );

                            let filename = format!("{}_capture.bmp", app);
                            if let Err(e) = WindowScreenshot::save_to_bmp(&screenshot, &filename) {
                                println!("❌ Failed to save {} capture: {}", app, e);
                            } else {
                                println!("✓ {} capture saved as {}", app, filename);
                            }
                        }
                        Err(e) => println!("❌ {} capture failed: {}", app, e),
                    }
                    break; // Only capture first matching app
                }
            }
            Err(_) => {} // Ignore errors for this search
        }
    }

    // Example 5: Launch Notepad and capture it
    println!("\n=== Notepad Screenshot Example ===");
    match launch_notepad() {
        Ok(_) => {
            // Try to capture notepad window
            let notepad_terms = ["notepad", "记事本"];

            for term in &notepad_terms {
                match WindowScreenshot::find_windows_by_partial_title(term) {
                    Ok(windows) => {
                        if !windows.is_empty() {
                            println!("Found {} Notepad windows:", windows.len());
                            for window in &windows {
                                println!(
                                    "  '{}' (PID: {}, Class: {})",
                                    window.title, window.process_id, window.class_name
                                );
                            }

                            // Capture the first notepad window
                            match WindowScreenshot::capture_window_by_partial_title(term) {
                                Ok(screenshot) => {
                                    println!(
                                        "✓ Notepad capture successful! Size: {}x{}",
                                        screenshot.width, screenshot.height
                                    );

                                    let filename = "notepad_capture.bmp";
                                    if let Err(e) =
                                        WindowScreenshot::save_to_bmp(&screenshot, filename)
                                    {
                                        println!("❌ Failed to save Notepad capture: {}", e);
                                    } else {
                                        println!("✓ Notepad capture saved as {}", filename);
                                    }
                                }
                                Err(e) => println!("❌ Notepad capture failed: {}", e),
                            }
                            break;
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(e) => println!("❌ Failed to launch notepad: {}", e),
    }

    println!("\nScreenshot example completed!");
    Ok(())
}
