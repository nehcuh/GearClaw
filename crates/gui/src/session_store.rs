//! Session persistence for the GearClaw GUI.
//!
//! This module handles saving and loading chat sessions to/from disk,
//! allowing users to continue their conversations across application restarts.

use crate::app::ChatMessage;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Serializable session data that can be stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Unique session identifier
    pub id: usize,
    /// Display name for the session
    pub name: String,
    /// All messages in this session
    pub messages: Vec<ChatMessage>,
    /// When the session was created
    pub created_at: String,
    /// When the session was last updated
    pub updated_at: String,
}

/// Manages persistent storage of chat sessions.
pub struct SessionStore {
    /// Directory where session files are stored
    session_dir: PathBuf,
}

impl SessionStore {
    /// Creates a new SessionStore with the specified session directory.
    ///
    /// # Arguments
    /// * `session_dir` - Directory path where session files will be stored
    ///
    /// # Returns
    /// * `Result<Self>` - The session store or an error if directory creation fails
    ///
    /// # Example
    /// ```no_run
    /// use gearclaw_gui::session_store::SessionStore;
    /// use std::path::PathBuf;
    ///
    /// let store = SessionStore::new(PathBuf::from("~/.gearclaw/sessions/"))
    ///     .expect("Failed to create session store");
    /// ```
    pub fn new(session_dir: PathBuf) -> Result<Self> {
        // Expand ~ to home directory if present
        let session_dir = dirs::home_dir()
            .map(|home| {
                session_dir
                    .to_str()
                    .and_then(|s| s.replace("~", home.to_str().unwrap_or("")).into())
                    .map(PathBuf::from)
                    .unwrap_or(session_dir.clone())
            })
            .unwrap_or(session_dir);

        // Create directory if it doesn't exist
        fs::create_dir_all(&session_dir)
            .with_context(|| format!("Failed to create session directory at {:?}", session_dir))?;

        Ok(Self { session_dir })
    }

    /// Saves a session to disk as JSON.
    ///
    /// # Arguments
    /// * `session` - The session data to save
    ///
    /// # Returns
    /// * `Result<()>` - Success or error message
    pub fn save_session(&self, session: &SessionData) -> Result<()> {
        let path = self.session_dir.join(format!("session_{}.json", session.id));
        let json = serde_json::to_string_pretty(session)
            .with_context(|| format!("Failed to serialize session {}", session.id))?;
        fs::write(&path, json)
            .with_context(|| format!("Failed to write session file to {:?}", path))?;
        Ok(())
    }

    /// Loads all sessions from disk.
    ///
    /// # Returns
    /// * `Result<Vec<SessionData>>` - Vector of sessions sorted by ID, or error
    pub fn load_sessions(&self) -> Result<Vec<SessionData>> {
        let mut sessions = Vec::new();

        let entries = fs::read_dir(&self.session_dir)
            .with_context(|| format!("Failed to read session directory {:?}", self.session_dir))?;

        for entry in entries {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read session file {:?}", path))?;

                let session: SessionData = serde_json::from_str(&json)
                    .with_context(|| format!("Failed to parse session file {:?}", path))?;

                sessions.push(session);
            }
        }

        // Sort by ID for consistent ordering
        sessions.sort_by_key(|s| s.id);

        Ok(sessions)
    }

    /// Deletes a session from disk.
    ///
    /// # Arguments
    /// * `id` - The session ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error if file doesn't exist or can't be deleted
    pub fn delete_session(&self, id: usize) -> Result<()> {
        let path = self.session_dir.join(format!("session_{}.json", id));

        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete session file {:?}", path))?;
        }

        Ok(())
    }

    /// Gets the next available session ID.
    ///
    /// # Returns
    /// * `usize` - The next ID to use for a new session
    pub fn next_session_id(&self) -> Result<usize> {
        let sessions = self.load_sessions()?;
        Ok(sessions.len().saturating_add(1))
    }

    /// Returns the path to the session directory.
    pub fn session_dir(&self) -> &PathBuf {
        &self.session_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_message(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn test_session_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("sessions");

        let store = SessionStore::new(session_dir.clone());
        assert!(store.is_ok());
        assert!(session_dir.exists());
    }

    #[test]
    fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("sessions");
        let store = SessionStore::new(session_dir).unwrap();

        let session = SessionData {
            id: 1,
            name: "Test Session".to_string(),
            messages: vec![
                create_test_message("user", "Hello"),
                create_test_message("assistant", "Hi there!"),
            ],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        // Save session
        assert!(store.save_session(&session).is_ok());

        // Load sessions
        let loaded = store.load_sessions().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, 1);
        assert_eq!(loaded[0].name, "Test Session");
        assert_eq!(loaded[0].messages.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("sessions");
        let store = SessionStore::new(session_dir).unwrap();

        let session = SessionData {
            id: 1,
            name: "Test Session".to_string(),
            messages: vec![create_test_message("user", "Hello")],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        store.save_session(&session).unwrap();
        assert_eq!(store.load_sessions().unwrap().len(), 1);

        store.delete_session(1).unwrap();
        assert_eq!(store.load_sessions().unwrap().len(), 0);
    }

    #[test]
    fn test_next_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let session_dir = temp_dir.path().join("sessions");
        let store = SessionStore::new(session_dir).unwrap();

        assert_eq!(store.next_session_id().unwrap(), 1);

        let session = SessionData {
            id: 1,
            name: "Test".to_string(),
            messages: vec![],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        store.save_session(&session).unwrap();
        assert_eq!(store.next_session_id().unwrap(), 2);
    }
}
