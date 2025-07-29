#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use tencent_ace_tools::app::run_gui_mode;

fn main() -> iced::Result {
    run_gui_mode()
}
