// Gateway Session Management
//
// This module manages active client connections and their state.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Active gateway session
#[derive(Debug, Clone)]
pub struct GatewaySession {
    /// Session ID
    pub id: String,

    /// Device ID (if authenticated)
    pub device_id: Option<String>,

    /// Client mode (operator, node, etc.)
    pub mode: String,

    /// Connection capabilities
    pub capabilities: Vec<String>,

    /// Session start timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl GatewaySession {
    pub fn new(device_id: Option<String>, mode: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            device_id,
            mode,
            capabilities: Vec::new(),
            connected_at: chrono::Utc::now(),
        }
    }
}

/// Gateway session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, GatewaySession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, device_id: Option<String>, mode: String) -> GatewaySession {
        let session = GatewaySession::new(device_id, mode);
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        session
    }

    pub async fn get_session(&self, id: &str) -> Option<GatewaySession> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    pub async fn remove_session(&self, id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
    }

    pub async fn list_sessions(&self) -> Vec<GatewaySession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
