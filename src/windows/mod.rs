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
    info!("  Tencent ACE Tools v{}", env!("CARGO_PKG_VERSION"));
    info!("  Gaming Performance Optimizer");
    info!("==========================================");
    info!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
    info!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
    info!("");
    info!("This is an OPEN SOURCE tool that helps optimize gaming");
    info!("performance by managing ACE Guard process priority.");
    info!("What this tool does:");
    info!("✓ Finds Tencent ACE Guard processes");
    info!("✓ Lowers their priority to IDLE level");
    info!("✓ Limits them to use only the last CPU core");
    info!("✓ Improves gaming performance");
    info!("");
    info!("What this tool does NOT do:");
    info!("✗ Does not modify game files");
    info!("✗ Does not disable anti-cheat");
    info!("✗ Does not inject code into processes");
    info!("✗ Does not bypass security measures");
    info!("==========================================");

    if is_running_as_admin()? {
        info!("✓ Program is running with administrator privileges");
        info!("Starting ACE Guard optimization process...");

        limit_ace_guard_64_priority()?;
    } else {
        warn!("✗ Administrator privileges required");
        info!("This program needs administrator privileges to modify process priorities.");
        info!("Attempting to restart with administrator privileges...");

        // try to request admin privileges
        match request_admin_privileges() {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to restart with admin privileges: {:?}", e);
                info!("Please right-click the program and select 'Run as administrator'");
            }
        }
    }

    info!("==========================================");
    info!("Operation completed. Press any key to exit...");
    info!("==========================================");

    std::io::stdin().read_line(&mut String::new()).ok();

    Ok(())
}

/// limit the priority of ACE Guard 64 processes
fn limit_ace_guard_64_priority() -> Result<()> {
    // Try to enable multiple privileges first
    if let Err(e) = enable_required_privileges() {
        warn!("Failed to enable required privileges: {:?}", e);
        info!("Continuing without enhanced privileges, some processes may be inaccessible");
    }

    unsafe {
        // Create a snapshot of the process
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        debug!(
            "Starting ACE Guard priority limitation, snapshot handle: {:?}",
            snapshot
        );

        // Iterate over all processes
        if Process32FirstW(snapshot, &mut process_entry).is_ok() {
            loop {
                // Convert the process name to a string
                let process_name_raw = String::from_utf16_lossy(&process_entry.szExeFile);
                let process_name = process_name_raw.trim_end_matches('\0');

                // Get the process path with fallback permissions
                let process_path = get_process_path(process_entry.th32ProcessID)
                    .unwrap_or_else(|_| "Access Denied".to_string());

                // trace!(
                //     "Checking process: {} (PID: {}), path: {}",
                //     process_name,
                //     process_entry.th32ProcessID,
                //     process_path
                // );

                // Check if it is an ACE Guard 64 process
                if process_name.eq(constants::ACE_GUARD_64_PROCESS_NAME) {
                    debug!(
                        "Found ACE Guard process: {} (PID: {}), path: {})",
                        process_name, process_entry.th32ProcessID, process_path
                    );

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
                                debug!(
                                    "Permission level {} failed for process {}: {:?}",
                                    i, process_name, e
                                );
                                continue;
                            }
                        }
                    }

                    match process_handle {
                        Some(handle) => {
                            debug!(
                                "Successfully opened process {} with permission level {}",
                                process_name, used_permission
                            );

                            // set process priority to idle
                            let priority_result = SetPriorityClass(handle, IDLE_PRIORITY_CLASS);

                            if priority_result.is_ok() {
                                info!(
                                    "Successfully lowered priority for process: {}",
                                    process_name
                                );
                            } else {
                                warn!(
                                    "Failed to lower priority for process: {} (Error: {:?})",
                                    process_name,
                                    priority_result.err()
                                );
                            }

                            // Set CPU affinity to the last CPU core
                            // Get the number of processors using std::thread
                            let num_processors = std::thread::available_parallelism()
                                .map(|n| n.get())
                                .unwrap_or(1);

                            // Create affinity mask for the last CPU (bit position = num_processors - 1)
                            let last_cpu_mask = 1usize << (num_processors - 1);

                            let affinity_result = SetProcessAffinityMask(handle, last_cpu_mask);

                            if affinity_result.is_ok() {
                                info!(
                                    "Successfully set CPU affinity to last core (CPU {}) for process: {}",
                                    num_processors - 1, process_name
                                );
                            } else {
                                warn!(
                                    "Failed to set CPU affinity for process: {} (Error: {:?})",
                                    process_name,
                                    affinity_result.err()
                                );
                            }

                            CloseHandle(handle).ok();
                        }
                        None => {
                            // Only show debug information for access denied errors
                            // as these are common for protected processes
                            debug!(
                                "Failed to open process {} (PID: {}) with any permission level",
                                process_name, process_entry.th32ProcessID
                            );
                            info!(
                                "Skipping process {} (PID: {}) - access denied or protected process",
                                process_name,
                                process_entry.th32ProcessID
                            );
                        }
                    }
                }

                // get next process
                if Process32NextW(snapshot, &mut process_entry).is_err() {
                    break;
                }
            }
        }

        CloseHandle(snapshot).ok();

        tracing::info!("ACE Guard priority limitation completed");
        Ok(())
    }
}
