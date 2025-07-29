use iced::{
    widget::{button, column, container, scrollable, text, Space},
    Element, Length, Padding,
};
use std::sync::{Arc, Mutex};

use crate::constants::{COLOR_BLUE, COLOR_GREEN, COLOR_ORANGE, COLOR_RED};
use crate::messages::{LogEvent, Message, ScreenshotData};
use crate::ui::theme::{get_header_font, get_monospace_font};

#[cfg(target_os = "windows")]
use crate::windows;

pub fn create_header() -> Element<'static, Message> {
    text("Tencent ACE Tools")
        .size(24)
        .font(get_header_font())
        .into()
}

pub fn create_description() -> Element<'static, Message> {
    text(env!("CARGO_PKG_DESCRIPTION"))
        .size(14)
        .into()
}

pub fn create_admin_status(is_admin: bool) -> Element<'static, Message> {
    if is_admin {
        text("Running with administrator privileges")
            .color(COLOR_GREEN)
            .into()
    } else {
        text("Administrator privileges required")
            .color(COLOR_RED)
            .into()
    }
}

pub fn create_buttons(is_optimizing: bool, is_taking_screenshot: bool) -> Element<'static, Message> {
    let optimize_button = if is_optimizing {
        button("Optimizing...")
    } else {
        button("Start Optimization").on_press(Message::StartOptimization)
    };

    let clear_logs_button = button("Clear Logs").on_press(Message::ClearLogs);

    let screenshot_button = if is_taking_screenshot {
        button("Taking Screenshot...")
    } else {
        button("Screenshot Game").on_press(Message::TakeScreenshot)
    };

    iced::widget::row![
        optimize_button,
        Space::with_width(Length::Fixed(10.0)),
        clear_logs_button,
        Space::with_width(Length::Fixed(10.0)),
        screenshot_button,
    ].into()
}

pub fn create_screenshot_section(screenshot_data: &Option<ScreenshotData>) -> Element<'_, Message> {
    if let Some(screenshot) = screenshot_data {
        // Create image handle from raw RGBA data
        let image_handle = iced::widget::image::Handle::from_rgba(
            screenshot.width,
            screenshot.height,
            screenshot.data.clone(),
        );

        container(
            iced::widget::image(image_handle)
                .width(Length::Fixed(400.0))
                .height(Length::Fixed(300.0)),
        )
        .padding(10)
        .width(Length::Fill)
        .into()
    } else {
        container(text("No screenshot available")
            .size(14)
            .color(COLOR_BLUE))
            .padding(10)
            .width(Length::Fill)
            .into()
    }
}

pub fn create_logs_section(logs: &Arc<Mutex<Vec<LogEvent>>>) -> Element<'_, Message> {
    let logs_content = if let Ok(logs) = logs.lock() {
        logs.iter()
            .rev()
            .take(50)
            .map(|entry| {
                let color = match entry.level.as_str() {
                    "ERROR" => COLOR_RED,
                    "WARN" => COLOR_ORANGE,
                    "INFO" => COLOR_GREEN,
                    "DEBUG" => COLOR_BLUE,
                    _ => iced::Color::WHITE,
                };
                
                text(format!(
                    "[{}] [{}] {}",
                    entry.timestamp.format("%H:%M:%S"),
                    entry.level,
                    entry.message
                ))
                .size(12)
                .font(get_monospace_font())
                .color(color)
            })
            .fold(column![], |col, log_text| col.push(log_text))
    } else {
        column![]
    };

    container(
        scrollable(
            container(logs_content)
                .padding(Padding::from([10, 15]))
                .width(Length::Fill),
        )
        .height(Length::Fixed(300.0)),
    )
    .padding(5)
    .width(Length::Fill)
    .into()
}

#[cfg(target_os = "windows")]
pub fn create_process_status_section(process_info: &Arc<Mutex<Vec<windows::ProcessInfo>>>) -> Element<'_, Message> {
    if let Ok(processes) = process_info.lock() {
        if processes.is_empty() {
            text("No ACE Guard processes found")
                .size(14)
                .color(COLOR_RED)
                .into()
        } else {
            let process_views: Vec<Element<Message>> = processes
                .iter()
                .map(|process| {
                    let status_text = format!(
                        "PID: {} | Priority: {} | Affinity: {} | Modified: {}{}",
                        process.process_id,
                        process.current_priority,
                        process.current_affinity,
                        if process.priority_modified || process.affinity_modified {
                            "✓"
                        } else {
                            "✗"
                        },
                        if process.priority_modified && process.affinity_modified {
                            " (Both)"
                        } else if process.priority_modified {
                            " (Priority)"
                        } else if process.affinity_modified {
                            " (Affinity)"
                        } else {
                            ""
                        }
                    );

                    text(status_text)
                        .size(12)
                        .font(get_monospace_font())
                        .color(if process.priority_modified || process.affinity_modified {
                            COLOR_GREEN
                        } else {
                            COLOR_RED
                        })
                        .into()
                })
                .collect();

            column![
                text("ACE Guard Process Status:")
                    .size(16),
                Space::with_height(Length::Fixed(5.0)),
                column(process_views).spacing(2)
            ]
            .into()
        }
    } else {
        text("Unable to load process information")
            .size(14)
            .color(COLOR_RED)
            .into()
    }
}

#[cfg(not(target_os = "windows"))]
pub fn create_process_status_section(_process_info: &Arc<Mutex<Vec<()>>>) -> Element<'_, Message> {
    text("Process status not available on this platform")
        .size(14)
        .color(COLOR_RED)
        .into()
}

pub fn create_info_text() -> Element<'static, Message> {
    text(
        "This tool optimizes Tencent ACE Guard processes by:\n\
        • Lowering process priority to IDLE level\n\
        • Setting CPU affinity to the last core\n\
        • Improving gaming performance without compromising security\n\n\
        ⚠️ Requires administrator privileges to modify process priorities",
    )
    .size(12)
    .into()
}
