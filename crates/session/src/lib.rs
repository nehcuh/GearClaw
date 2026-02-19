use chrono::{DateTime, Utc};
use gearclaw_llm::Message;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;
const MAX_SESSION_ID_LENGTH: usize = 128;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid session id: {0}")]
    InvalidSessionId(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(default = "default_cwd")]
    pub cwd: PathBuf,
}

fn default_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            cwd: default_cwd(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{} msgs] (updated: {})",
            self.id,
            self.messages.len(),
            self.updated_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

pub struct SessionManager {
    session_dir: PathBuf,
}

impl SessionManager {
    pub fn new(session_dir: PathBuf) -> Result<Self, SessionError> {
        if !session_dir.exists() {
            std::fs::create_dir_all(&session_dir)?;
        }
        let session_dir = std::fs::canonicalize(session_dir)?;
        Ok(Self { session_dir })
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, SessionError> {
        let mut sessions = Vec::new();
        if !self.session_dir.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.session_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(stem.to_string());
                }
            }
        }
        sessions.sort();
        Ok(sessions)
    }

    pub fn get_or_create_session(&self, id: &str) -> Result<Session, SessionError> {
        let path = self.session_file_path(id)?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let session: Session = serde_json::from_str(&content)?;
            Ok(session)
        } else {
            Ok(Session::new(id.to_string()))
        }
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), SessionError> {
        let path = self.session_file_path(&session.id)?;
        let content = serde_json::to_string_pretty(session)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub fn delete_session(&self, id: &str) -> Result<(), SessionError> {
        let path = self.session_file_path(id)?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn session_file_path(&self, id: &str) -> Result<PathBuf, SessionError> {
        Self::validate_session_id(id)?;
        let path = self.session_dir.join(format!("{}.json", id));
        if !path.starts_with(&self.session_dir) {
            return Err(SessionError::InvalidSessionId(
                "resolved path escapes session directory".to_string(),
            ));
        }
        let parent = path.parent().ok_or_else(|| {
            SessionError::InvalidSessionId("failed to resolve session file parent".to_string())
        })?;
        if parent != self.session_dir {
            return Err(SessionError::InvalidSessionId(
                "session id cannot include path separators".to_string(),
            ));
        }
        Ok(path)
    }

    fn validate_session_id(id: &str) -> Result<(), SessionError> {
        if id.trim().is_empty() {
            return Err(SessionError::InvalidSessionId(
                "session id cannot be empty".to_string(),
            ));
        }
        if id.len() > MAX_SESSION_ID_LENGTH {
            return Err(SessionError::InvalidSessionId(format!(
                "session id too long (max {})",
                MAX_SESSION_ID_LENGTH
            )));
        }
        if id == "." || id == ".." || id.contains("..") {
            return Err(SessionError::InvalidSessionId(
                "session id cannot contain path traversal sequence".to_string(),
            ));
        }
        if id.contains('/') || id.contains('\\') {
            return Err(SessionError::InvalidSessionId(
                "session id cannot contain path separators".to_string(),
            ));
        }
        if !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
        {
            return Err(SessionError::InvalidSessionId(
                "session id contains unsupported characters".to_string(),
            ));
        }
        Ok(())
    }
}
