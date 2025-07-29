use windows::{
    core::*,
    Win32::{
        Foundation::*, Security::*, System::Threading::*,
    },
};

/// check if the program is running as admin
pub fn is_running_as_admin() -> Result<bool> {
    unsafe {
        let mut token: HANDLE = HANDLE::default();

        // get the access token of the current process
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)?;

        // check if the program is running as admin
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;

        GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )?;

        CloseHandle(token).ok();

        Ok(elevation.TokenIsElevated != 0)
    }
}

/// Get the full path of a process with fallback permissions
pub fn get_process_path(process_id: u32) -> Result<String> {
    unsafe {
        // Try different permission levels in order of preference
        let permissions = [PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION];

        for &permission in &permissions {
            if let Ok(handle) = OpenProcess(permission, false, process_id) {
                let mut path_buffer = [0u16; 260]; // MAX_PATH
                let mut size = path_buffer.len() as u32;

                let result = QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_WIN32,
                    PWSTR(path_buffer.as_mut_ptr()),
                    &mut size,
                );

                CloseHandle(handle).ok();

                if result.is_ok() && size > 0 {
                    return Ok(String::from_utf16_lossy(&path_buffer[..size as usize]));
                }
            }
        }

        // If all methods fail, return an error
        Err(Error::from(E_ACCESSDENIED))
    }
}

/// Enable required privileges to access and modify processes - only what we actually need
pub fn enable_required_privileges() -> Result<()> {
    // Only request privileges that are actually needed for process management
    let privileges = [
        w!("SeDebugPrivilege"),                    // To access protected processes
        w!("SeIncreaseBasePriorityPrivilege"),     // To lower process priority
    ];

    let mut success_count = 0;

    for privilege_name in &privileges {
        if enable_single_privilege(privilege_name).is_ok() {
            success_count += 1;
            tracing::debug!("Successfully enabled privilege: {:?}", privilege_name);
        } else {
            tracing::debug!("Failed to enable privilege: {:?}", privilege_name);
        }
    }

    if success_count > 0 {
        tracing::info!(
            "Enabled {}/{} privileges",
            success_count,
            privileges.len()
        );
        Ok(())
    } else {
        tracing::warn!("Failed to enable any privileges");
        Err(Error::from_hresult(HRESULT(0x80070005u32 as i32)))
    }
}

/// Enable a single privilege
pub fn enable_single_privilege(privilege_name: &PCWSTR) -> Result<()> {
    unsafe {
        let mut token_handle = HANDLE::default();

        // Get current process token
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle,
        )
        .is_err()
        {
            return Err(Error::from_hresult(HRESULT(0x80070005u32 as i32)));
        }

        // Lookup privilege
        let mut luid = LUID::default();

        if LookupPrivilegeValueW(PCWSTR::null(), *privilege_name, &mut luid).is_err() {
            CloseHandle(token_handle).ok();
            return Err(Error::from_hresult(HRESULT(0x80070005u32 as i32)));
        }

        // Set up the privilege structure
        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }; 1],
        };

        // Adjust token privileges
        let result = AdjustTokenPrivileges(token_handle, false, Some(&mut tp), 0, None, None);

        CloseHandle(token_handle).ok();

        if result.is_err() {
            return Err(Error::from_hresult(HRESULT(0x80070005u32 as i32)));
        }

        Ok(())
    }
}

/// Get current process priority class
pub fn get_process_priority(process_id: u32) -> Result<String> {
    unsafe {
        let permissions = [PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION];

        for &permission in &permissions {
            if let Ok(handle) = OpenProcess(permission, false, process_id) {
                let priority = GetPriorityClass(handle);
                CloseHandle(handle).ok();

                if priority != 0 {
                    let priority_class = match priority {
                        0x40 => "IDLE",
                        0x4000 => "BELOW_NORMAL", 
                        0x20 => "NORMAL",
                        0x8000 => "ABOVE_NORMAL",
                        0x80 => "HIGH",
                        0x100 => "REALTIME",
                        _ => "UNKNOWN",
                    };
                    return Ok(priority_class.to_string());
                }
            }
        }

        Err(Error::from(E_ACCESSDENIED))
    }
}

/// Get current process CPU affinity
pub fn get_process_affinity(process_id: u32) -> Result<String> {
    unsafe {
        let permissions = [PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION];

        for &permission in &permissions {
            if let Ok(handle) = OpenProcess(permission, false, process_id) {
                let mut process_affinity_mask = 0_usize;
                let mut system_affinity_mask = 0_usize;

                let result = GetProcessAffinityMask(
                    handle,
                    &mut process_affinity_mask,
                    &mut system_affinity_mask,
                );

                CloseHandle(handle).ok();

                if result.is_ok() {
                    // Find which cores are set
                    let mut cores = Vec::new();
                    for i in 0..64 {
                        if (process_affinity_mask & (1 << i)) != 0 {
                            cores.push(i);
                        }
                    }

                    if cores.is_empty() {
                        return Ok("No cores assigned".to_string());
                    } else if cores.len() == 1 {
                        return Ok(format!("Core {}", cores[0]));
                    } else {
                        return Ok(format!("Cores: {}", cores.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")));
                    }
                }
            }
        }

        Err(Error::from(E_ACCESSDENIED))
    }
}

/// Get process status information including priority and affinity
pub fn get_process_status(process_id: u32) -> Result<(String, String)> {
    let priority = get_process_priority(process_id).unwrap_or_else(|_| "Access Denied".to_string());
    let affinity = get_process_affinity(process_id).unwrap_or_else(|_| "Access Denied".to_string());
    
    Ok((priority, affinity))
}

/// Find processes by name and return their process IDs
pub fn find_process_by_name(process_name: &str) -> Result<Vec<u32>> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };

    unsafe {
        let mut process_ids = Vec::new();
        
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        if snapshot.is_invalid() {
            return Err(Error::from_win32());
        }

        let mut process_entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut process_entry).is_ok() {
            loop {
                let current_process_name_raw = String::from_utf16_lossy(&process_entry.szExeFile);
                let current_process_name = current_process_name_raw.trim_end_matches('\0');

                if current_process_name.eq_ignore_ascii_case(process_name) {
                    process_ids.push(process_entry.th32ProcessID);
                }

                if Process32NextW(snapshot, &mut process_entry).is_err() {
                    break;
                }
            }
        }

        CloseHandle(snapshot).ok();
        
        if process_ids.is_empty() {
            Err(Error::from_hresult(windows::core::HRESULT(-1)))
        } else {
            Ok(process_ids)
        }
    }
}
