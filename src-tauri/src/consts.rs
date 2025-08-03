use std::sync::OnceLock;
use tauri::AppHandle;

pub static TAURI_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

// pub const ACE_ANTI_CHEAT_EXPERT_PATH: &str = "C:\\Program Files\\AntiCheatExpert";
// pub const ACE_GUARD_64_SUBPATH: &str = "SGuard\\x64";
pub const ACE_GUARD_64_PROCESS_NAME: &str = "SGuard64.exe";

pub const DELTA_FORCE_PROCESS_NAME: &str = "DeltaForceClient-Win64-Shipping.exe";
