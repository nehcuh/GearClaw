// GearClaw Gateway - OpenClaw-compatible Gateway Service
//
// This crate implements the always-on Gateway service with WebSocket protocol support,
// device identity, and method handlers for OpenClaw compatibility.

pub mod protocol;
pub mod server;
pub mod handlers;
pub mod auth;
pub mod identity;
pub mod session;
pub mod presence;
pub mod triggers;

pub use protocol::*;
pub use server::{GatewayServer, GatewayConfig};
pub use handlers::MethodHandlers;
pub use auth::TokenAuth;
pub use identity::{DeviceIdentity, DeviceKeyPair};
pub use session::GatewaySession;
pub use presence::PresenceManager;
pub use crate::protocol::PresenceEntry;
