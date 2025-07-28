use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use iced::{
    widget::{button, column, container, scrollable, text, Space},
    Element, Length, Padding, Task, Theme,
};
use std::sync::{Arc, Mutex};

use crate::constants::{COLOR_GREEN, COLOR_RED};

mod constants;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Parser)]
#[command(name = "tencent-ace-tools")]
#[command(
    about = "A Windows utility to optimize Tencent ACE Guard process for better gaming performance"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in debug mode with detailed console output
    Cli {
        /// Enable verbose debug logging
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run the GUI version (default)
    Gui,
}

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
                Task::perform(
                    async move { run_optimization(logs_clone).await },
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

async fn run_optimization(logs: Arc<Mutex<Vec<LogEntry>>>) -> Result<String, String> {
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
            Ok(result) => {
                add_log("SUCCESS", &result);
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
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Cli { verbose }) => {
            // Run in CLI debug mode
            run_cli_mode(verbose);
            Ok(())
        }
        Some(Commands::Gui) | None => {
            // Run GUI mode (default)
            run_gui_mode()
        }
    }
}

fn run_cli_mode(verbose: bool) {
    // Try to initialize tracing for debug mode with more detailed output
    let level = if verbose {
        tracing::Level::TRACE
    } else if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    // Use try_init to avoid conflicts if subscriber is already initialized
    let _ = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .try_init();

    println!("==========================================");
    println!("        CLI RUN MODE ACTIVATED");
    println!("==========================================");
    println!("Tencent ACE Tools v{}", env!("CARGO_PKG_VERSION"));
    println!("Running in CLI mode with enhanced logging");
    if verbose {
        println!("Verbose logging enabled (TRACE level)");
    }
    println!("==========================================");

    // Create a Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new().unwrap();

    #[cfg(target_os = "windows")]
    {
        match rt.block_on(windows::run_cli(verbose)) {
            Ok(_) => {
                println!("CLI run completed successfully");
            }
            Err(e) => {
                eprintln!("CLI run failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("CLI run is only supported on Windows");
        std::process::exit(1);
    }
}

fn run_gui_mode() -> iced::Result {
    // Initialize tracing for GUI mode with iced logging disabled
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("iced=error".parse().unwrap())
        .add_directive("wgpu=error".parse().unwrap())
        .add_directive("tracing=error".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .init();

    iced::application(AceToolsApp::title, AceToolsApp::update, AceToolsApp::view)
        .theme(AceToolsApp::theme)
        .window_size((800.0, 600.0))
        .run_with(AceToolsApp::new)
}
