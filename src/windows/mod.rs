use tracing::*;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Diagnostics::ToolHelp::*, System::Threading::*},
};

use crate::constants;

mod utils;
use utils::*;

pub async fn run() -> Result<()> {
    // check if the program is running in a terminal environment
    let is_terminal = atty::is(atty::Stream::Stdout);
    let is_windows_terminal = std::env::var("WT_SESSION").is_ok();

    // Configure tracing subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            Level::TRACE
        } else {
            Level::INFO
        })
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    // Configure colors based on terminal type
    if is_terminal && (is_windows_terminal || console::colors_enabled()) {
        // Windows Terminal or a terminal that supports colors
        subscriber.with_ansi(true).init();
    } else {
        // CMD or a terminal that does not support colors
        subscriber.with_ansi(false).init();
    }

    // Show terminal information
    detect_terminal_environment();

    info!("==========================================");
    info!("  Tencent ACE Gaming Performance Optimizer v{}", env!("CARGO_PKG_VERSION"));
    info!("  Open Source Gaming Performance Tool");
    info!("==========================================");
    info!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    info!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    info!("");
    info!("This is an open source gaming performance optimization tool that legally optimizes anti-cheat process priority:");
    info!("✓ Finds Tencent ACE Guard anti-cheat processes (SGuard64.exe)");
    info!("✓ Lowers their priority to IDLE level");
    info!("✓ Limits them to use only the last CPU core");
    info!("✓ Improves gaming performance and reduces stuttering");
    info!("");
    info!("This tool will NEVER:");
    info!("✗ Modify game files or inject code");
    info!("✗ Disable or bypass anti-cheat systems");
    info!("✗ Affect game security or account safety");
    info!("✗ Perform any malicious operations");
    info!("");
    info!("This is an OPEN SOURCE gaming optimization tool that:");
    info!("✓ Finds Tencent ACE Guard anti-cheat processes");
    info!("✓ Lowers their priority to IDLE level");
    info!("✓ Limits them to use only the last CPU core");  
    info!("✓ Improves gaming performance without compromising security");
    info!("==========================================");

    if is_running_as_admin()? {
        info!("✓ Program is running with administrator privileges");
        
        // Add user confirmation for safety
        info!("");
        info!("⚠️  Safety Confirmation");
        info!("==========================================");
        info!("This program will modify system process priority and CPU affinity.");
        info!("This is a completely safe and reversible operation.");
        info!("Enter 'y' or 'yes' to continue, any other input will exit the program.");
        info!("==========================================");
        
        print!("Continue? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let input = input.trim().to_lowercase();
        
        if input != "y" && input != "yes" {
            info!("Operation cancelled by user");
            info!("Exiting program");
            std::process::exit(0);
        }
        
        info!("User confirmed to continue");
        info!("Starting ACE Guard optimization...");
        
        info!("");
        info!("Performing the following operations:");
        info!("1. Scanning system for ACE Guard processes");
        info!("2. Attempting to lower process priority to IDLE level");
        info!("3. Setting CPU affinity to the last core");
        info!("4. These operations are completely legal and safe");
        info!("");

        limit_ace_guard_64_priority()?;
    } else {
        warn!("✗ Administrator privileges required to modify process priorities");
        info!("");
        info!("==========================================");
        info!("           IMPORTANT NOTICE");
        info!("==========================================");
        info!("This program requires administrator privileges to modify system process priorities.");
        info!("This is a normal Windows security requirement.");
        info!("");
        info!("Please follow these steps:");
        info!("1. Close this program");
        info!("2. Right-click on the program icon");
        info!("3. Select 'Run as administrator'");
        info!("4. Click 'Yes' in the UAC prompt");
        info!("==========================================");
        info!("");
        info!("Program will exit in 10 seconds...");
        
        // Wait for 10 seconds before exiting
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        std::process::exit(1);
    }

    info!("==========================================");
    info!("Operation completed. Press any key to exit...");
    info!("==========================================");

    std::io::stdin().read_line(&mut String::new()).ok();

    Ok(())
}

/// limit the priority of ACE Guard 64 processes
fn limit_ace_guard_64_priority() -> Result<()> {
    info!("Starting system process scan...");
    
    // Try to enable multiple privileges first
    if let Err(e) = enable_required_privileges() {
        warn!("Failed to enable enhanced privileges, some protected processes may be inaccessible: {:?}", e);
        info!("Continuing with basic privileges");
    } else {
        info!("✓ Enhanced privileges enabled successfully");
    }

    let mut found_processes = 0;
    let mut modified_processes = 0;

    unsafe {
        // Create a snapshot of the process
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        info!("Enumerating system processes...");

        // Iterate over all processes
        if Process32FirstW(snapshot, &mut process_entry).is_ok() {
            loop {
                // Convert the process name to a string
                let process_name_raw = String::from_utf16_lossy(&process_entry.szExeFile);
                let process_name = process_name_raw.trim_end_matches('\0');

                // Check if it is an ACE Guard 64 process
                if process_name.eq(constants::ACE_GUARD_64_PROCESS_NAME) {
                    found_processes += 1;
                    
                    // Get the process path with fallback permissions
                    let process_path = get_process_path(process_entry.th32ProcessID)
                        .unwrap_or_else(|_| "Access Denied".to_string());

                    info!("Found ACE Guard process:");
                    info!("  Name: {}", process_name);
                    info!("  PID: {}", process_entry.th32ProcessID);
                    info!("  Path: {}", process_path);

                    // Try different permission levels to open the process
                    let permissions = [
                        PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
                        PROCESS_SET_INFORMATION,
                        PROCESS_ALL_ACCESS,
                        PROCESS_QUERY_INFORMATION,
                        PROCESS_QUERY_LIMITED_INFORMATION,
                    ];

                    let mut process_handle = None;
                    let mut used_permission = 0;

                    for (i, &permission) in permissions.iter().enumerate() {
                        match OpenProcess(permission, false, process_entry.th32ProcessID) {
                            Ok(handle) => {
                                process_handle = Some(handle);
                                used_permission = i;
                                break;
                            }
                            Err(e) => {
                                debug!("Permission level {} failed: {:?}", i, e);
                                continue;
                            }
                        }
                    }

                    match process_handle {
                        Some(handle) => {
                            info!("  ✓ Successfully opened process handle (permission level: {})", used_permission);

                            let mut operation_success = false;

                            // set process priority to idle
                            info!("  Setting process priority to IDLE...");
                            let priority_result = SetPriorityClass(handle, IDLE_PRIORITY_CLASS);

                            if priority_result.is_ok() {
                                info!("  ✓ Successfully lowered process priority");
                                operation_success = true;
                            } else {
                                warn!("  ✗ Failed to set priority: {:?}", priority_result.err());
                            }

                            // Set CPU affinity to the last CPU core
                            info!("  Setting CPU affinity...");
                            
                            // Get the number of processors using std::thread
                            let num_processors = std::thread::available_parallelism()
                                .map(|n| n.get())
                                .unwrap_or(1);

                            info!("  Detected {} CPU cores", num_processors);

                            // Create affinity mask for the last CPU (bit position = num_processors - 1)
                            let last_cpu_mask = 1usize << (num_processors - 1);
                            info!("  Limiting process to CPU core {}", num_processors - 1);

                            let affinity_result = SetProcessAffinityMask(handle, last_cpu_mask);

                            if affinity_result.is_ok() {
                                info!("  ✓ Successfully set CPU affinity");
                                operation_success = true;
                            } else {
                                warn!("  ✗ Failed to set CPU affinity: {:?}", affinity_result.err());
                            }

                            if operation_success {
                                modified_processes += 1;
                                info!("  ✓ Process optimization completed");
                            } else {
                                warn!("  ✗ Process optimization failed");
                            }

                            CloseHandle(handle).ok();
                        }
                        None => {
                            warn!("  ✗ Cannot open process handle - may be protected process");
                            info!("  This is usually normal, some system processes are protected");
                        }
                    }
                    
                    info!(""); // Add blank line for readability
                }

                // get next process
                if Process32NextW(snapshot, &mut process_entry).is_err() {
                    break;
                }
            }
        }

        CloseHandle(snapshot).ok();

        info!("==========================================");
        info!("Scan Results Summary:");
        info!("Found ACE Guard processes: {}", found_processes);
        info!("Successfully optimized processes: {}", modified_processes);
        
        if found_processes == 0 {
            info!("No ACE Guard processes found, may not be running Tencent games currently");
        } else if modified_processes > 0 {
            info!("✓ Gaming performance optimization completed!");
            info!("ACE Guard process priority lowered, CPU usage limited");
        }
        info!("==========================================");

        Ok(())
    }
}
