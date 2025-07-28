use windows::{
    core::*,
    Win32::{
        Foundation::*, Security::*, System::Threading::*,
    },
};

/// check if the program is running in a terminal environment
pub fn detect_terminal_environment() {
    let is_windows_terminal = std::env::var("WT_SESSION").is_ok();
    let is_vscode_terminal = std::env::var("VSCODE_INJECTION").is_ok();
    
    if is_windows_terminal {
        tracing::debug!("Running in Windows Terminal");
    } else if is_vscode_terminal {
        tracing::debug!("Running in VS Code Terminal");
    } else {
        tracing::debug!("Running in standard terminal (likely CMD)");
    }
}

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
