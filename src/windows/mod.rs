use tracing::*;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Diagnostics::ToolHelp::*, System::Threading::*},
};

use crate::constants;

mod utils;
use utils::*;

// Re-export the function for public access
pub use utils::is_running_as_admin;

pub async fn run_optimization() -> anyhow::Result<(String, Vec<ProcessInfo>)> {
    let mut controller = ProcessController::new()?;
    let result = controller.optimize_ace_guard_processes().await?;
    let processes = controller.get_processes().to_vec();
    Ok((result, processes))
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub process_name: String,
    pub process_path: String,
    pub priority_modified: bool,
    pub affinity_modified: bool,
    pub current_priority: String,
    pub current_affinity: String,
}

pub struct ProcessController {
    processes: Vec<ProcessInfo>,
    privileges_enabled: bool,
}

impl ProcessController {
    pub fn new() -> Result<Self> {
        // Try to enable privileges first
        let privileges_enabled = enable_required_privileges().is_ok();

        if privileges_enabled {
            info!("✓ Enhanced privileges enabled successfully");
        } else {
            warn!("Failed to enable enhanced privileges, some protected processes may be inaccessible");
            info!("Continuing with basic privileges");
        }

        Ok(Self {
            processes: Vec::new(),
            privileges_enabled,
        })
    }

    pub async fn optimize_ace_guard_processes(&mut self) -> anyhow::Result<String> {
        info!("Starting system process scan...");

        self.scan_processes()?;

        if self.processes.is_empty() {
            return Ok("No ACE Guard processes found on the system. This is normal if no Tencent games are currently running.".to_string());
        }

        let mut modified_count = 0;
        let processes_len = self.processes.len();

        for i in 0..self.processes.len() {
            if self.optimize_process_at_index(i).await {
                modified_count += 1;
            }
        }

        let result = format!(
            "Process scan completed: Found {} processes, Modified {} processes",
            processes_len, modified_count
        );

        if modified_count == 0 {
            return Err(anyhow::anyhow!("No processes were successfully modified. This may be due to insufficient permissions or process protection."));
        } else if modified_count < processes_len {
            warn!("Some processes could not be modified");
        } else {
            info!("✓ All ACE Guard processes have been successfully optimized!");
        }

        Ok(result)
    }

    fn scan_processes(&mut self) -> Result<()> {
        self.processes.clear();

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

            let mut process_entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            info!("Enumerating system processes...");

            if Process32FirstW(snapshot, &mut process_entry).is_ok() {
                loop {
                    let process_name_raw = String::from_utf16_lossy(&process_entry.szExeFile);
                    let process_name = process_name_raw.trim_end_matches('\0');

                    if process_name.eq(constants::ACE_GUARD_64_PROCESS_NAME) {
                        let process_path = get_process_path(process_entry.th32ProcessID)
                            .unwrap_or_else(|_| "Access Denied".to_string());

                        info!("Found ACE Guard process:");
                        info!("  Name: {}", process_name);
                        info!("  PID: {}", process_entry.th32ProcessID);
                        info!("  Path: {}", process_path);

                        let (current_priority, current_affinity) = utils::get_process_status(process_entry.th32ProcessID)
                            .unwrap_or_else(|_| ("Access Denied".to_string(), "Access Denied".to_string()));

                        self.processes.push(ProcessInfo {
                            process_id: process_entry.th32ProcessID,
                            process_name: process_name.to_string(),
                            process_path,
                            priority_modified: false,
                            affinity_modified: false,
                            current_priority,
                            current_affinity,
                        });
                    }

                    if Process32NextW(snapshot, &mut process_entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

        info!("Found {} ACE Guard processes", self.processes.len());
        Ok(())
    }

    async fn optimize_process_at_index(&mut self, index: usize) -> bool {
        if index >= self.processes.len() {
            return false;
        }

        let process = &mut self.processes[index];
        let permissions = [
            PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION,
            PROCESS_SET_INFORMATION,
            PROCESS_ALL_ACCESS,
            PROCESS_QUERY_INFORMATION,
            PROCESS_QUERY_LIMITED_INFORMATION,
        ];

        let mut process_handle = None;
        let mut used_permission = 0;

        unsafe {
            for (i, &permission) in permissions.iter().enumerate() {
                match OpenProcess(permission, false, process.process_id) {
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
                    info!(
                        "  ✓ Successfully opened process handle (permission level: {})",
                        used_permission
                    );

                    let mut operation_success = false;

                    // Set process priority to idle
                    info!("  Setting process priority to IDLE...");
                    let priority_result = SetPriorityClass(handle, IDLE_PRIORITY_CLASS);

                    if priority_result.is_ok() {
                        info!("  ✓ Successfully lowered process priority");
                        process.priority_modified = true;
                        operation_success = true;
                    } else {
                        warn!("  ✗ Failed to set priority: {:?}", priority_result.err());
                    }

                    // Set CPU affinity to the last CPU core
                    info!("  Setting CPU affinity to last core...");
                    let cpu_count = num_cpus::get();
                    let last_core_mask = 1_usize << (cpu_count - 1);

                    let affinity_result = SetProcessAffinityMask(handle, last_core_mask);
                    if affinity_result.is_ok() {
                        info!(
                            "  ✓ Successfully set CPU affinity to core {}",
                            cpu_count - 1
                        );
                        process.affinity_modified = true;
                        operation_success = true;
                    } else {
                        warn!(
                            "  ✗ Failed to set CPU affinity: {:?}",
                            affinity_result.err()
                        );
                    }

                    if operation_success {
                        info!("  ✓ Process optimization completed");
                    } else {
                        warn!("  ✗ No operations succeeded for this process");
                    }

                    let _ = CloseHandle(handle);
                    operation_success
                }
                None => {
                    warn!("  ✗ Failed to open process handle with any permission level");
                    warn!("  This process may be protected or already terminated");
                    false
                }
            }
        }
    }

    pub fn get_processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    pub fn get_privileges_enabled(&self) -> bool {
        self.privileges_enabled
    }
}
