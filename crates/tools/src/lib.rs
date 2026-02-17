//! gearclaw_tools
//!
//! Phase-2 extraction crate for tool execution and registry abstractions.
//! Currently re-exports `gearclaw_core` implementations for compatibility.

#[cfg(target_os = "macos")]
pub use gearclaw_core::macos::*;
pub use gearclaw_core::tools::*;

/// Registry abstraction for tool providers.
pub trait ToolRegistry {
    fn list_tools(&self) -> Vec<ToolSpec>;
}

impl ToolRegistry for ToolExecutor {
    fn list_tools(&self) -> Vec<ToolSpec> {
        self.available_tools()
    }
}
