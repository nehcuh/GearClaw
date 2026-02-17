//! gearclaw_session
//! Compatibility extraction crate for session management.

pub use gearclaw_core::llm::Message;
pub use gearclaw_core::session::*;

pub trait SessionStore {
    fn get_or_create_session(&self, id: &str) -> Result<Session, gearclaw_core::GearClawError>;
}

impl SessionStore for SessionManager {
    fn get_or_create_session(&self, id: &str) -> Result<Session, gearclaw_core::GearClawError> {
        SessionManager::get_or_create_session(self, id)
    }
}
