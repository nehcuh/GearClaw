//! Compatibility wrapper for session subsystem.
//! Delegates to `gearclaw_session` while preserving `gearclaw_core` API.
use crate::config::SessionConfig;
use crate::error::GearClawError;
use gearclaw_llm::Message;

pub use gearclaw_session::Session;

pub struct SessionManager {
    inner: gearclaw_session::SessionManager,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Result<Self, GearClawError> {
        let inner = gearclaw_session::SessionManager::new(config.session_dir)
            .map_err(|e| GearClawError::config_parse_error(e.to_string()))?;
        Ok(Self { inner })
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, GearClawError> {
        self.inner.list_sessions().map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: "list_sessions".to_string(),
                reason: e.to_string(),
            })
        })
    }

    pub fn get_or_create_session(&self, id: &str) -> Result<Session, GearClawError> {
        self.inner.get_or_create_session(id).map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: format!("get_or_create_session({})", id),
                reason: e.to_string(),
            })
        })
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), GearClawError> {
        self.inner.save_session(session).await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: format!("save_session({})", session.id),
                reason: e.to_string(),
            })
        })
    }

    pub fn delete_session(&self, id: &str) -> Result<(), GearClawError> {
        self.inner.delete_session(id).map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: format!("delete_session({})", id),
                reason: e.to_string(),
            })
        })
    }

    /// Atomic operation: add a message to a session
    ///
    /// This is a convenience method that adds a message to the session atomically,
    /// preventing race conditions when multiple tasks modify the same session.
    pub async fn add_message(&self, id: &str, message: Message) -> Result<(), GearClawError> {
        self.inner.add_message(id, message).await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: format!("add_message({})", id),
                reason: e.to_string(),
            })
        })
    }

    /// Atomic operation: clear session history
    ///
    /// This is a convenience method that clears the session history atomically.
    pub async fn clear_history(&self, id: &str) -> Result<(), GearClawError> {
        self.inner.clear_history(id).await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Session {
                operation: format!("clear_history({})", id),
                reason: e.to_string(),
            })
        })
    }
}
