use crate::config::SessionConfig;
use crate::error::GearClawError;

pub use gearclaw_session::Session;

pub struct SessionManager {
    inner: gearclaw_session::SessionManager,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Result<Self, GearClawError> {
        let inner = gearclaw_session::SessionManager::new(config.session_dir)
            .map_err(|e| GearClawError::Other(e.to_string()))?;
        Ok(Self { inner })
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, GearClawError> {
        self.inner
            .list_sessions()
            .map_err(|e| GearClawError::Other(e.to_string()))
    }

    pub fn get_or_create_session(&self, id: &str) -> Result<Session, GearClawError> {
        self.inner
            .get_or_create_session(id)
            .map_err(|e| GearClawError::Other(e.to_string()))
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), GearClawError> {
        self.inner
            .save_session(session)
            .await
            .map_err(|e| GearClawError::Other(e.to_string()))
    }

    pub fn delete_session(&self, id: &str) -> Result<(), GearClawError> {
        self.inner
            .delete_session(id)
            .map_err(|e| GearClawError::Other(e.to_string()))
    }
}
