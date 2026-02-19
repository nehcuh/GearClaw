// GearClaw Gateway - OpenClaw-compatible Gateway Service
//
// This crate implements the always-on Gateway service with WebSocket protocol support,
// device identity, and method handlers for OpenClaw compatibility.

pub mod auth;
pub mod handlers;
pub mod identity;
pub mod presence;
pub mod protocol;
pub mod server;
pub mod session;
pub mod triggers;

pub use crate::protocol::PresenceEntry;
pub use auth::TokenAuth;
pub use handlers::MethodHandlers;
pub use identity::{DeviceIdentity, DeviceKeyPair};
pub use presence::PresenceManager;
pub use protocol::*;
pub use server::{GatewayConfig, GatewayServer};
pub use session::GatewaySession;
