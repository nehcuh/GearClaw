use crate::config::SessionConfig;
use crate::error::GearClawError;
use crate::llm::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use std::path::PathBuf;

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
        Session {
            id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
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
    config: SessionConfig,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Result<Self, GearClawError> {
        if !config.session_dir.exists() {
            std::fs::create_dir_all(&config.session_dir)?;
        }
        Ok(SessionManager { config })
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, GearClawError> {
        let mut sessions = Vec::new();
        if !self.config.session_dir.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.config.session_dir)? {
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

    pub fn get_or_create_session(&self, id: &str) -> Result<Session, GearClawError> {
        let path = self.config.session_dir.join(format!("{}.json", id));

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let session: Session = serde_json::from_str(&content)?;
            Ok(session)
        } else {
            Ok(Session::new(id.to_string()))
        }
    }

    pub async fn save_session(&self, session: &Session) -> Result<(), GearClawError> {
        let path = self.config.session_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        
        tokio::fs::write(path, content).await.map_err(GearClawError::IoError)?;
        Ok(())
    }

    pub fn delete_session(&self, id: &str) -> Result<(), GearClawError> {
        let path = self.config.session_dir.join(format!("{}.json", id));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
