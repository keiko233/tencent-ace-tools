use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum Message {
    StartOptimization,
    OptimizationCompleted(Result<String, String>),
    ClearLogs,
    TakeScreenshot,
    ScreenshotCompleted(Result<ScreenshotData, String>),
}

#[derive(Debug, Clone)]
pub struct ScreenshotData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp: DateTime<Local>,
    pub level: String,
    pub message: String,
    pub target: String,
}
