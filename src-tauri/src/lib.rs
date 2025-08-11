use crate::logging::LogEvent;
use specta_typescript::BigIntExportBehavior;
use specta_typescript::Typescript;
use std::{io, path::Path, process::Command};
use tauri_specta::{collect_commands, collect_events, Builder};

pub mod command;
use command::*;

pub mod logging;

#[cfg(target_os = "windows")]
pub mod windows;

pub mod consts;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn app_run() {
    logging::init_logging();

    let command_builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            greet,
            is_running_as_admin,
            get_all_ace_guard_processes,
            optimize_all_ace_guard_processes,
            get_controller_privileges_status,
            get_all_windows,
            try_capture_image_by_window_id,
            ocr_screen_region,
            ocr_image_region,
            ocr_full_screen,
        ])
        .events(collect_events![LogEvent,]);

    #[cfg(debug_assertions)]
    command_builder
        .export(
            Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .formatter(|file: &Path| {
                    Command::new("pnpm")
                        .arg("fix:prettier")
                        .arg(file)
                        .output()
                        .map(|_| ())
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                })
                .bigint(BigIntExportBehavior::Number)
                .header("/* eslint-disable @typescript-eslint/no-unused-vars */\n/* eslint-disable */\n// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(windows::AceProcessControllerState::default())
        .invoke_handler(command_builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            command_builder.mount_events(app);

            // set app handle via once lock
            let _ = consts::TAURI_APP_HANDLE.set(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
