#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use chrono::{DateTime, Local};
use iced::{
    widget::{button, column, container, scrollable, text, Space, row},
    Element, Length, Padding, Task, Theme,
};
use std::sync::{Arc, Mutex};

use crate::constants::{COLOR_BLUE, COLOR_GREEN, COLOR_RED};

mod constants;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone)]
pub enum Message {
    StartOptimization,
    OptimizationCompleted(Result<String, String>),
    ClearLogs,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: String,
    pub message: String,
}

pub struct AceToolsApp {
    is_optimizing: bool,
    optimization_result: Option<Result<String, String>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
    is_admin: bool,
    process_info: Arc<Mutex<Vec<windows::ProcessInfo>>>,
}

impl AceToolsApp {
    fn new() -> (AceToolsApp, Task<Message>) {
        let is_admin = check_admin_privileges();

        (
            AceToolsApp {
                is_optimizing: false,
                optimization_result: None,
                logs: Arc::new(Mutex::new(Vec::new())),
                is_admin,
                process_info: Arc::new(Mutex::new(Vec::new())),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        format!("Tencent ACE Tools v{}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartOptimization => {
                if !self.is_admin {
                    self.add_log("ERROR", "Administrator privileges required!");
                    return Task::none();
                }

                self.is_optimizing = true;
                self.optimization_result = None;
                self.add_log("INFO", "Starting ACE Guard optimization...");

                let logs_clone = Arc::clone(&self.logs);
                let process_info_clone = Arc::clone(&self.process_info);
                Task::perform(
                    async move { run_optimization(logs_clone, process_info_clone).await },
                    Message::OptimizationCompleted,
                )
            }
            Message::OptimizationCompleted(result) => {
                self.is_optimizing = false;

                match &result {
                    Ok(msg) => self.add_log("SUCCESS", msg),
                    Err(err) => self.add_log("ERROR", err),
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
        }
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let header = text("Tencent ACE Tools").size(24);

        let description = text(env!("CARGO_PKG_DESCRIPTION")).size(14);

        let admin_status = if self.is_admin {
            text("Running with administrator privileges").color(COLOR_GREEN)
        } else {
            text("Administrator privileges required").color(COLOR_RED)
        };

        let optimize_button = if self.is_optimizing {
            button("Optimizing...")
        } else {
            button("Start Optimization").on_press(Message::StartOptimization)
        };

        let clear_logs_button = button("Clear Logs").on_press(Message::ClearLogs);

        let buttons_row = iced::widget::row![
            optimize_button,
            Space::with_width(Length::Fixed(10.0)),
            clear_logs_button,
        ];

        let logs_content = if let Ok(logs) = self.logs.lock() {
            logs.iter()
                .rev()
                .take(50)
                .map(|entry| {
                    text(format!(
                        "[{}] [{}] {}",
                        entry.timestamp.format("%H:%M:%S"),
                        entry.level,
                        entry.message
                    ))
                    .size(12)
                    .font(iced::Font::with_name("monospace"))
                })
                .fold(column![], |col, log_text| col.push(log_text))
        } else {
            column![]
        };

        let logs_section = container(
            scrollable(
                container(logs_content)
                    .padding(Padding::from([10, 15]))
                    .width(Length::Fill),
            )
            .height(Length::Fixed(300.0)),
        )
        .padding(5)
        .width(Length::Fill);

        // Process status section
        let process_status_section: Element<Message> = if let Ok(processes) = self.process_info.lock() {
            if processes.is_empty() {
                text("No ACE Guard processes found").size(14).color(COLOR_RED).into()
            } else {
                let process_views: Vec<Element<Message>> = processes.iter().map(|process| {
                    let status_text = format!(
                        "PID: {} | Priority: {} | Affinity: {} | Modified: {}{}",
                        process.process_id,
                        process.current_priority,
                        process.current_affinity,
                        if process.priority_modified || process.affinity_modified { "✓" } else { "✗" },
                        if process.priority_modified && process.affinity_modified { " (Both)" } 
                        else if process.priority_modified { " (Priority)" }
                        else if process.affinity_modified { " (Affinity)" }
                        else { "" }
                    );
                    
                    text(status_text)
                        .size(12)
                        .font(iced::Font::with_name("monospace"))
                        .color(if process.priority_modified || process.affinity_modified { COLOR_GREEN } else { COLOR_RED })
                        .into()
                }).collect();

                column![
                    text("ACE Guard Process Status:").size(16),
                    Space::with_height(Length::Fixed(5.0)),
                    column(process_views).spacing(2)
                ].into()
            }
        } else {
            text("Unable to load process information").size(14).color(COLOR_RED).into()
        };

        let info_text = text(
            "This tool optimizes Tencent ACE Guard processes by:\n\
            • Lowering process priority to IDLE level\n\
            • Setting CPU affinity to the last core\n\
            • Improving gaming performance without compromising security\n\n\
            ⚠️ Requires administrator privileges to modify process priorities",
        )
        .size(12);

        let content = column![
            header,
            Space::with_height(Length::Fixed(10.0)),
            description,
            Space::with_height(Length::Fixed(15.0)),
            admin_status,
            Space::with_height(Length::Fixed(20.0)),
            buttons_row,
            Space::with_height(Length::Fixed(20.0)),
            process_status_section,
            Space::with_height(Length::Fixed(15.0)),
            text("Logs:").size(16),
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

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl AceToolsApp {
    fn add_log(&self, level: &str, message: &str) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(LogEntry {
                timestamp: Local::now(),
                level: level.to_string(),
                message: message.to_string(),
            });
        }
    }
}

async fn run_optimization(
    logs: Arc<Mutex<Vec<LogEntry>>>,
    process_info: Arc<Mutex<Vec<windows::ProcessInfo>>>,
) -> Result<String, String> {
    let add_log = |level: &str, message: &str| {
        if let Ok(mut logs_guard) = logs.lock() {
            logs_guard.push(LogEntry {
                timestamp: Local::now(),
                level: level.to_string(),
                message: message.to_string(),
            });
        }
    };

    #[cfg(target_os = "windows")]
    {
        match windows::run_optimization().await {
            Ok((result, processes)) => {
                add_log("SUCCESS", &result);
                if let Ok(mut process_info_guard) = process_info.lock() {
                    *process_info_guard = processes;
                }
                Ok(result)
            }
            Err(e) => {
                let error_msg = format!("Optimization failed: {}", e);
                add_log("ERROR", &error_msg);
                Err(error_msg)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let error_msg = "Not supported on this operating system".to_string();
        add_log("ERROR", &error_msg);
        Err(error_msg)
    }
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

fn main() -> iced::Result {
    run_gui_mode()
}

fn run_gui_mode() -> iced::Result {
    // Initialize tracing for GUI mode
    #[cfg(debug_assertions)]
    {
        // Debug mode: show all logs to console
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("tencent_ace_tools=debug".parse().unwrap())
            .add_directive("iced=warn".parse().unwrap())
            .add_directive("wgpu=warn".parse().unwrap());

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .init();
    }
    
    #[cfg(not(debug_assertions))]
    {
        // Release mode: minimal logging for GUI
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("iced=error".parse().unwrap())
            .add_directive("wgpu=error".parse().unwrap())
            .add_directive("tracing=error".parse().unwrap());

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .init();
    }

    #[cfg(debug_assertions)]
    tracing::info!("Starting Tencent ACE Tools in debug mode");
    
    iced::application(AceToolsApp::title, AceToolsApp::update, AceToolsApp::view)
        .theme(AceToolsApp::theme)
        .window_size((800.0, 600.0))
        .run_with(AceToolsApp::new)
}
