use std::sync::{Arc, Mutex};
use gpui::{App, Global};
use tracing_subscriber::Layer;

const MAX_LOG_ENTRIES: usize = 500;

/// Shared log buffer accessible from the UI.
#[derive(Clone)]
pub struct LogStore {
    pub entries: Arc<Mutex<Vec<String>>>,
}

impl Global for LogStore {}

impl LogStore {
    pub fn new() -> Self {
        LogStore {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn read_entries(&self) -> Vec<String> {
        self.entries.lock().unwrap().clone()
    }
}

/// Get the global LogStore from GPUI context.
pub fn log_entries(cx: &App) -> Vec<String> {
    cx.try_global::<LogStore>()
        .map(|store| store.read_entries())
        .unwrap_or_default()
}

/// A tracing Layer that appends formatted log lines to the shared buffer.
pub struct GuiLogLayer {
    entries: Arc<Mutex<Vec<String>>>,
}

impl GuiLogLayer {
    pub fn new(entries: Arc<Mutex<Vec<String>>>) -> Self {
        GuiLogLayer { entries }
    }
}

impl<S> Layer<S> for GuiLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let level = metadata.level();
        let target = metadata.target();

        // Collect fields
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);

        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        let line = format!("[{}] {} {} - {}", timestamp, level, target, visitor.message);

        if let Ok(mut entries) = self.entries.lock() {
            entries.push(line);
            // Trim to max size
            if entries.len() > MAX_LOG_ENTRIES {
                let drain = entries.len() - MAX_LOG_ENTRIES;
                entries.drain(..drain);
            }
        }
    }
}

#[derive(Default)]
struct FieldVisitor {
    message: String,
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else if !self.message.is_empty() {
            self.message.push_str(&format!(" {}={:?}", field.name(), value));
        } else {
            self.message = format!("{}={:?}", field.name(), value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else if !self.message.is_empty() {
            self.message.push_str(&format!(" {}={}", field.name(), value));
        } else {
            self.message = format!("{}={}", field.name(), value);
        }
    }
}
