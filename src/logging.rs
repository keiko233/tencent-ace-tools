use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt, Layer};
use crate::messages::LogEvent;
use chrono::Local;

// Custom layer to capture tracing events
#[derive(Clone)]
pub struct GuiLogLayer {
    logs: Arc<Mutex<Vec<LogEvent>>>,
}

impl GuiLogLayer {
    pub fn new(logs: Arc<Mutex<Vec<LogEvent>>>) -> Self {
        Self { logs }
    }
}

impl<S> Layer<S> for GuiLogLayer
where
    S: tracing::Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = metadata.level().to_string();
        let target = metadata.target().to_string();

        // Create a visitor to extract the message
        struct MessageVisitor {
            message: String,
        }

        impl tracing::field::Visit for MessageVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.message = format!("{:?}", value);
                    // Remove surrounding quotes if present
                    if self.message.starts_with('"') && self.message.ends_with('"') {
                        self.message = self.message[1..self.message.len() - 1].to_string();
                    }
                }
            }
        }

        let mut visitor = MessageVisitor {
            message: String::new(),
        };

        event.record(&mut visitor);

        if !visitor.message.is_empty() {
            let log_event = LogEvent {
                timestamp: Local::now(),
                level,
                message: visitor.message,
                target,
            };

            if let Ok(mut logs) = self.logs.lock() {
                logs.push(log_event);
                // Keep maximum 1000 log entries
                if logs.len() > 1000 {
                    logs.remove(0);
                }
            }
        }
    }
}

pub fn init_logging(gui_layer: GuiLogLayer) {
    #[cfg(debug_assertions)]
    {
        // Debug mode: show all logs to console and GUI
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("tencent_ace_tools=debug".parse().unwrap())
            .add_directive("iced=warn".parse().unwrap())
            .add_directive("wgpu=warn".parse().unwrap());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
            )
            .with(gui_layer)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        // Release mode: minimal logging for GUI
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("tencent_ace_tools=info".parse().unwrap())
            .add_directive("iced=error".parse().unwrap())
            .add_directive("wgpu=error".parse().unwrap())
            .add_directive("tracing=error".parse().unwrap());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(gui_layer)
            .init();
    }
}
