use iced::{
    widget::{column, container, Space},
    Element, Length, Task, Theme,
};
use std::sync::{Arc, Mutex};

use crate::logging::{GuiLogLayer, init_logging};
use crate::messages::{LogEvent, Message, ScreenshotData};
use crate::ui::components::*;

#[cfg(target_os = "windows")]
use crate::windows;

pub struct AceToolsApp {
    pub is_optimizing: bool,
    pub optimization_result: Option<Result<String, String>>,
    pub logs: Arc<Mutex<Vec<LogEvent>>>,
    pub is_admin: bool,
    #[cfg(target_os = "windows")]
    pub process_info: Arc<Mutex<Vec<windows::ProcessInfo>>>,
    #[cfg(not(target_os = "windows"))]
    pub process_info: Arc<Mutex<Vec<()>>>,
    pub screenshot_data: Option<ScreenshotData>,
    pub is_taking_screenshot: bool,
}

impl AceToolsApp {
    pub fn new() -> (AceToolsApp, Task<Message>) {
        let is_admin = check_admin_privileges();
        let logs = Arc::new(Mutex::new(Vec::new()));

        (
            AceToolsApp {
                is_optimizing: false,
                optimization_result: None,
                logs: logs.clone(),
                is_admin,
                process_info: Arc::new(Mutex::new(Vec::new())),
                screenshot_data: None,
                is_taking_screenshot: false,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        format!("Tencent ACE Tools v{}", env!("CARGO_PKG_VERSION"))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartOptimization => {
                if !self.is_admin {
                    tracing::error!("Administrator privileges required!");
                    return Task::none();
                }

                self.is_optimizing = true;
                self.optimization_result = None;
                tracing::info!("Starting ACE Guard optimization...");

                let process_info_clone = Arc::clone(&self.process_info);
                Task::perform(
                    async move { run_optimization(process_info_clone).await },
                    Message::OptimizationCompleted,
                )
            }
            Message::OptimizationCompleted(result) => {
                self.is_optimizing = false;

                match &result {
                    Ok(msg) => {
                        tracing::info!("Optimization completed: {}", msg);
                    }
                    Err(err) => {
                        tracing::error!("Optimization failed: {}", err);
                    }
                }

                self.optimization_result = Some(result);
                Task::none()
            }
            Message::ClearLogs => {
                if let Ok(mut logs) = self.logs.lock() {
                    logs.clear();
                }
                Task::none()
            }
            Message::TakeScreenshot => {
                self.is_taking_screenshot = true;
                tracing::info!("Taking screenshot of game window...");

                Task::perform(
                    async move { take_game_screenshot().await },
                    Message::ScreenshotCompleted,
                )
            }
            Message::ScreenshotCompleted(result) => {
                self.is_taking_screenshot = false;

                match result {
                    Ok(image_data) => {
                        self.screenshot_data = Some(image_data);
                        tracing::info!("Screenshot captured successfully!");
                    }
                    Err(err) => {
                        tracing::error!("Screenshot failed: {}", err);
                    }
                }

                Task::none()
            }
        }
    }

    pub fn view(&'_ self) -> Element<'_, Message> {
        let header = create_header();
        let description = create_description();
        let admin_status = create_admin_status(self.is_admin);
        let buttons_row = create_buttons(self.is_optimizing, self.is_taking_screenshot);
        let screenshot_section = create_screenshot_section(&self.screenshot_data);
        let logs_section = create_logs_section(&self.logs);
        let process_status_section = create_process_status_section(&self.process_info);
        let info_text = create_info_text();

        let content = column![
            header,
            Space::with_height(Length::Fixed(10.0)),
            description,
            Space::with_height(Length::Fixed(15.0)),
            admin_status,
            Space::with_height(Length::Fixed(20.0)),
            buttons_row,
            Space::with_height(Length::Fixed(15.0)),
            iced::widget::text("Screenshot:").size(16),
            Space::with_height(Length::Fixed(5.0)),
            screenshot_section,
            Space::with_height(Length::Fixed(20.0)),
            process_status_section,
            Space::with_height(Length::Fixed(15.0)),
            iced::widget::text("Logs:").size(16),
            Space::with_height(Length::Fixed(5.0)),
            logs_section,
            Space::with_height(Length::Fixed(15.0)),
            info_text,
        ]
        .padding(20)
        .width(Length::Fill);

        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }
}

#[cfg(target_os = "windows")]
async fn run_optimization(
    process_info: Arc<Mutex<Vec<windows::ProcessInfo>>>,
) -> Result<String, String> {
    match windows::run_optimization().await {
        Ok((result, processes)) => {
            tracing::info!("Optimization completed successfully: {}", result);
            if let Ok(mut process_info_guard) = process_info.lock() {
                *process_info_guard = processes;
            }
            Ok(result)
        }
        Err(e) => Err(format!("Optimization failed: {}", e)),
    }
}

#[cfg(not(target_os = "windows"))]
async fn run_optimization(_process_info: Arc<Mutex<Vec<()>>>) -> Result<String, String> {
    Err("Not supported on this operating system".to_string())
}

fn check_admin_privileges() -> bool {
    #[cfg(target_os = "windows")]
    {
        match windows::is_running_as_admin() {
            Ok(is_admin) => is_admin,
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "windows"))]
    false
}

async fn take_game_screenshot() -> Result<ScreenshotData, String> {
    #[cfg(target_os = "windows")]
    {
        use crate::constants::DELTA_FORCE_PROCESS_NAME;
        use crate::windows::find_process_by_name;
        use crate::windows::screen::WindowScreenshot;

        // Try to find Delta Force process first
        tracing::info!("Looking for process: {}", DELTA_FORCE_PROCESS_NAME);

        match find_process_by_name(DELTA_FORCE_PROCESS_NAME) {
            Ok(process_ids) => {
                if !process_ids.is_empty() {
                    let process_id = process_ids[0];
                    tracing::info!(
                        "Found game process: {} (PID: {})",
                        DELTA_FORCE_PROCESS_NAME,
                        process_id
                    );

                    match WindowScreenshot::capture_window_by_pid(process_id) {
                        Ok(screenshot) => {
                            tracing::info!(
                                "Screenshot captured: {}x{}",
                                screenshot.width,
                                screenshot.height
                            );

                            // Convert BGRA to RGBA for iced
                            let mut rgba_data = Vec::with_capacity(screenshot.data.len());
                            for chunk in screenshot.data.chunks(4) {
                                if chunk.len() == 4 {
                                    // Convert BGRA to RGBA
                                    rgba_data.push(chunk[2]); // R
                                    rgba_data.push(chunk[1]); // G
                                    rgba_data.push(chunk[0]); // B
                                    rgba_data.push(chunk[3]); // A
                                }
                            }

                            return Ok(ScreenshotData {
                                data: rgba_data,
                                width: screenshot.width as u32,
                                height: screenshot.height as u32,
                            });
                        }
                        Err(e) => {
                            return Err(format!(
                                "Failed to capture window for process '{}' (PID: {}): {}",
                                DELTA_FORCE_PROCESS_NAME, process_id, e
                            ));
                        }
                    }
                } else {
                    return Err(format!(
                        "Process '{}' found but no valid PID",
                        DELTA_FORCE_PROCESS_NAME
                    ));
                }
            }
            Err(e) => {
                return Err(format!(
                    "Process '{}' not found: {}",
                    DELTA_FORCE_PROCESS_NAME, e
                ));
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Screenshot not supported on this operating system".to_string())
    }
}

pub fn run_gui_mode() -> iced::Result {
    // Create application instance to get shared log storage
    let (app, _) = AceToolsApp::new();
    let gui_layer = GuiLogLayer::new(app.logs.clone());

    // Initialize logging
    init_logging(gui_layer);

    #[cfg(debug_assertions)]
    tracing::info!("Starting Tencent ACE Tools in debug mode");

    iced::application(AceToolsApp::title, AceToolsApp::update, AceToolsApp::view)
        .theme(AceToolsApp::theme)
        .window_size((800.0, 600.0))
        .run_with(move || (app, Task::none()))
}
