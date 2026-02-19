//! gearclaw_core
//!
//! Core facade crate for GearClaw.
//! During the architecture split, this crate keeps stable public API
//! while delegating subsystem implementations to extracted crates
//! (`gearclaw_llm`, `gearclaw_session`, `gearclaw_memory`,
//! `gearclaw_mcp`, `gearclaw_tools`).
pub mod agent;
pub mod config;
pub mod error;
pub mod llm;
pub mod macos;
pub mod mcp;
pub mod memory;
pub mod session;
pub mod skills;
pub mod tools;

// Re-export commonly used types
pub use agent::Agent;
pub use config::{AgentConfig, AgentTriggerConfig, Config, GatewayConfig, TriggerMode};
pub use error::{DomainError, GearClawError, InfraError};
