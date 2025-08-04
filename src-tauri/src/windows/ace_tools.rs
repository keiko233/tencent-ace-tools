use crate::{
    consts,
    windows::utils::{enable_required_privileges, get_process_path, get_process_status},
};
use windows::Win32::{
    Foundation::CloseHandle,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::{
            OpenProcess, SetPriorityClass, SetProcessAffinityMask, IDLE_PRIORITY_CLASS,
            PROCESS_ALL_ACCESS, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SET_INFORMATION,
        },
    },
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub process_name: String,
    pub process_path: String,
    pub priority_modified: bool,
    pub affinity_modified: bool,
    pub current_priority: String,
    pub current_affinity: String,
    pub is_optimized: bool,
}

#[derive(Clone)]
pub struct AceProcessController {
    processes: Vec<ProcessInfo>,
    privileges_enabled: bool,
}

impl AceProcessController {
    pub fn new() -> Self {
        // Try to enable privileges first
        let privileges_enabled = enable_required_privileges().is_ok();
        tracing::debug!("Privileges enabled: {}", privileges_enabled);
        Self {
            processes: Vec::new(),
            privileges_enabled,
        }
    }

    pub fn scan_ace_guard_processes(&mut self) -> Result<Vec<ProcessInfo>, String> {
        self.scan_processes()
            .map_err(|e| format!("Failed to scan processes: {}", e))?;

        if self.processes.is_empty() {
            return Err("No ACE Guard processes found on the system.".to_string());
        }

        tracing::info!("Found {} ACE Guard processes", self.processes.len());
        Ok(self.processes.clone())
    }

    pub async fn optimize_all_processes(&mut self) -> Result<String, String> {
        if self.processes.is_empty() {
            return Err("No processes to optimize. Please scan processes first.".to_string());
        }

        let mut modified_count = 0;
        let processes_len = self.processes.len();

        for i in 0..self.processes.len() {
            if self.optimize_process_at_index(i).await {
                modified_count += 1;
            }
        }

        let result = format!(
            "Process optimization completed: Found {} processes, Modified {} processes",
            processes_len, modified_count
        );

        if modified_count == 0 {
            return Err("No processes were successfully modified. This may be due to insufficient permissions or process protection.".to_string());
        } else if modified_count < processes_len {
            tracing::warn!("Some processes could not be modified");
        } else {
            tracing::info!("ACE Guard processes have been successfully optimized!");
        }

        Ok(result)
    }

    pub async fn optimize_single_process(&mut self, process_id: u32) -> Result<String, String> {
        let process_index = self
            .processes
            .iter()
            .position(|p| p.process_id == process_id)
            .ok_or_else(|| {
                format!(
                    "Process with PID {} not found in scanned processes",
                    process_id
                )
            })?;

        if self.optimize_process_at_index(process_index).await {
            Ok(format!(
                "Process {} (PID: {}) optimized successfully",
                self.processes[process_index].process_name, process_id
            ))
        } else {
            Err(format!(
                "Failed to optimize process {} (PID: {})",
                self.processes[process_index].process_name, process_id
            ))
        }
    }

    pub async fn optimize_ace_guard_processes(&mut self) -> std::result::Result<String, String> {
        self.scan_ace_guard_processes()?;
        self.optimize_all_processes().await
    }

    fn scan_processes(&mut self) -> Result<(), String> {
        // get the previous optimization states
        let previous_optimized_states: std::collections::HashMap<u32, bool> = self
            .processes
            .iter()
            .map(|p| (p.process_id, p.is_optimized))
            .collect();

        self.processes.clear();

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|e| format!("Failed to create process snapshot: {:?}", e))?;

            let mut process_entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            tracing::debug!("Enumerating system processes...");

            if Process32FirstW(snapshot, &mut process_entry).is_ok() {
                loop {
                    let process_name_raw = String::from_utf16_lossy(&process_entry.szExeFile);
                    let process_name = process_name_raw.trim_end_matches('\0');

                    if process_name.eq(consts::ACE_GUARD_64_PROCESS_NAME) {
                        let process_path = get_process_path(process_entry.th32ProcessID)
                            .unwrap_or_else(|_| "Access Denied".to_string());

                        tracing::debug!(
                            "Found ACE Guard process: {} (PID: {})",
                            process_name,
                            process_entry.th32ProcessID
                        );

                        let (current_priority, current_affinity) =
                            get_process_status(process_entry.th32ProcessID).unwrap_or_else(|_| {
                                ("Access Denied".to_string(), "Access Denied".to_string())
                            });

                        // 检查是否之前已优化过此进程
                        let is_optimized = previous_optimized_states
                            .get(&process_entry.th32ProcessID)
                            .copied()
                            .unwrap_or(false);

                        self.processes.push(ProcessInfo {
                            process_id: process_entry.th32ProcessID,
                            process_name: process_name.to_string(),
                            process_path,
                            priority_modified: false,
                            affinity_modified: false,
                            current_priority,
                            current_affinity,
                            is_optimized,
                        });
                    }

                    if Process32NextW(snapshot, &mut process_entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
        }

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
                        tracing::debug!("Permission level {} failed: {:?}", i, e);
                        continue;
                    }
                }
            }

            match process_handle {
                Some(handle) => {
                    tracing::info!(
                        "Successfully opened process handle (permission level: {})",
                        used_permission
                    );

                    let mut operation_success = false;

                    // Set process priority to idle
                    let priority_result = SetPriorityClass(handle, IDLE_PRIORITY_CLASS);

                    if priority_result.is_ok() {
                        tracing::info!("Successfully lowered process priority");
                        process.priority_modified = true;
                        operation_success = true;
                    } else {
                        tracing::warn!("Failed to set priority: {:?}", priority_result.err());
                    }

                    // Set CPU affinity to the last CPU core
                    tracing::info!("Setting CPU affinity to last core...");
                    let cpu_count = num_cpus::get();
                    let last_core_mask = 1_usize << (cpu_count - 1);

                    let affinity_result = SetProcessAffinityMask(handle, last_core_mask);
                    if affinity_result.is_ok() {
                        process.affinity_modified = true;
                        operation_success = true;
                    } else {
                        tracing::warn!("Failed to set CPU affinity: {:?}", affinity_result.err());
                    }

                    if operation_success {
                        process.is_optimized = true;
                        tracing::info!("Process optimization completed");
                    } else {
                        process.is_optimized = false;
                        tracing::warn!("No operations succeeded for this process");
                    }

                    let _ = CloseHandle(handle);
                    operation_success
                }
                None => false,
            }
        }
    }

    pub fn get_processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    pub fn get_privileges_enabled(&self) -> bool {
        self.privileges_enabled
    }

    pub fn get_process_count(&self) -> usize {
        self.processes.len()
    }

    pub fn get_optimized_count(&self) -> usize {
        self.processes
            .iter()
            .filter(|p| p.is_optimized)
            .count()
    }

    pub fn clear_processes(&mut self) {
        self.processes.clear();
    }

    pub fn has_processes(&self) -> bool {
        !self.processes.is_empty()
    }
}
