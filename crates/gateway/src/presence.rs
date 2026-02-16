// Presence Management
//
// This module manages device presence (online/offline status) and broadcasts updates.

use crate::protocol::PresenceEntry;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Presence manager
pub struct PresenceManager {
    /// Current presence entries
    presence: Arc<RwLock<Vec<PresenceEntry>>>,

    /// State version for delta tracking
    state_version: Arc<RwLock<u64>>,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            presence: Arc::new(RwLock::new(Vec::new())),
            state_version: Arc::new(RwLock::new(0)),
        }
    }

    /// Add or update a presence entry
    pub async fn update(&self, entry: PresenceEntry) {
        let mut presence = self.presence.write().await;
        let mut state_version = self.state_version.write().await;

        // Remove existing entry for same host
        presence.retain(|e| e.host != entry.host);

        // Add new entry
        presence.push(entry);

        // Increment state version
        *state_version += 1;
    }

    /// Remove a presence entry
    pub async fn remove(&self, host: &str) {
        let mut presence = self.presence.write().await;
        let mut state_version = self.state_version.write().await;

        presence.retain(|e| e.host != host);

        *state_version += 1;
    }

    /// Get current presence snapshot
    pub async fn snapshot(&self) -> Vec<PresenceEntry> {
        let presence = self.presence.read().await;
        presence.clone()
    }

    /// Get current state version
    pub async fn state_version(&self) -> u64 {
        *self.state_version.read().await
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}
