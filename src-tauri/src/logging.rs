use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;
use tauri_specta::Event;
use tracing::{field::Visit, Level, Subscriber};
use tracing_subscriber::{
    layer::{Context, SubscriberExt},
    util::SubscriberInitExt,
    Layer,
};

use crate::consts::TAURI_APP_HANDLE;

// Log level enum for TypeScript bindings
#[derive(Debug, Clone, Serialize, Type)]
pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::TRACE => LogLevel::TRACE,
            Level::DEBUG => LogLevel::DEBUG,
            Level::INFO => LogLevel::INFO,
            Level::WARN => LogLevel::WARN,
            Level::ERROR => LogLevel::ERROR,
        }
    }
}

impl From<&Level> for LogLevel {
    fn from(level: &Level) -> Self {
        LogLevel::from(*level)
    }
}

#[derive(Debug, Clone, Serialize, Type, Event)]
pub struct LogEvent {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub fields: std::collections::HashMap<String, String>,
}

struct TauriEventLayer;

struct LogFieldVisitor {
    fields: std::collections::HashMap<String, String>,
    message: String,
}

impl LogFieldVisitor {
    fn new() -> Self {
        Self {
            fields: std::collections::HashMap::new(),
            message: String::new(),
        }
    }
}

impl Visit for LogFieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .insert(field.name().to_string(), value.to_string());
        }
    }
}

impl<S> Layer<S> for TauriEventLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if let Some(app_handle) = TAURI_APP_HANDLE.get() {
            let mut visitor = LogFieldVisitor::new();
            event.record(&mut visitor);

            let log_event = LogEvent {
                level: LogLevel::from(event.metadata().level()),
                target: event.metadata().target().to_string(),
                message: visitor.message,
                timestamp: Utc::now(),
                fields: visitor.fields,
            };

            log_event.emit(app_handle).unwrap();
        }
        // Remove the warning log to prevent infinite recursion
        // The TAURI_APP_HANDLE will be set once the app is properly initialized
    }
}

pub fn init_logging() {
    #[cfg(debug_assertions)]
    {
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("trace".parse().unwrap());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_target(true))
            .with(TauriEventLayer)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(TauriEventLayer)
            .try_init()
            .ok();
    }
}
