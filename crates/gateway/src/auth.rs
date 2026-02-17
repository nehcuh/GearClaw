// Bearer Token Authentication
//
// Simplified authentication using Bearer tokens (temporary replacement for ed25519).
//
// TODO: [TODO-ID-001] Replace with full ed25519 device identity signature/verification

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token authentication system
pub struct TokenAuth {
    tokens: Arc<RwLock<HashMap<String, TokenInfo>>>,
}

/// Token information
#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub device_id: String,
    pub mode: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl TokenAuth {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate a token
    pub async fn validate(&self, token: &str) -> bool {
        // Check format: "Bearer <token>"
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        // Check length (should be at least 32 chars)
        if token.len() < 32 {
            tracing::warn!("Token validation failed: too short ({} chars)", token.len());
            return false;
        };

        // Check format (alphanumeric with some special chars)
        if !token
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            tracing::warn!("Token validation failed: invalid characters");
            return false;
        }

        // Check if token exists in registry
        let tokens = self.tokens.read().await;
        match tokens.get(token) {
            Some(_info) => {
                tracing::debug!("Token validated successfully");
                true
            }
            None => {
                tracing::warn!("Token validation failed: token not found in registry");
                false
            }
        }
    }

    /// Register a new token
    pub async fn register(&self, token: String, device_id: String, mode: String) -> Result<()> {
        // Validate token format
        let token = if token.starts_with("Bearer ") {
            token[7..].to_string()
        } else {
            token
        };

        if token.len() < 32 {
            return Err(anyhow::anyhow!("Token too short (min 32 chars)"));
        }

        let info = TokenInfo {
            device_id,
            mode,
            created_at: Utc::now(),
            last_used: None,
        };

        self.tokens.write().await.insert(token, info);
        tracing::info!("Token registered successfully");
        Ok(())
    }

    /// Get token info
    pub async fn get_token_info(&self, token: &str) -> Option<TokenInfo> {
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        self.tokens.read().await.get(token).cloned()
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, token: &str) {
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        if let Some(info) = self.tokens.write().await.get_mut(token) {
            info.last_used = Some(Utc::now());
        }
    }

    /// Revoke a token
    pub async fn revoke(&self, token: &str) -> Result<()> {
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        self.tokens.write().await.remove(token);
        tracing::info!("Token revoked successfully");
        Ok(())
    }

    /// Get registered token count
    pub async fn token_count(&self) -> usize {
        self.tokens.read().await.len()
    }
}

impl Default for TokenAuth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_validation_valid() {
        let auth = TokenAuth::new();

        // Register a token
        auth.register(
            "test-token-abcdefghijklmnopqrstuvwxyz123456".to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        // Validate it
        assert!(
            auth.validate("test-token-abcdefghijklmnopqrstuvwxyz123456")
                .await
        );
        assert!(
            auth.validate("Bearer test-token-abcdefghijklmnopqrstuvwxyz123456")
                .await
        );
    }

    #[tokio::test]
    async fn test_token_validation_invalid_length() {
        let auth = TokenAuth::new();

        // Too short
        assert!(!auth.validate("short-token").await);
    }

    #[tokio::test]
    async fn test_token_validation_invalid_chars() {
        let auth = TokenAuth::new();

        // Invalid characters
        assert!(!auth.validate("token-with-@-invalid-chars!").await);
    }

    #[tokio::test]
    async fn test_token_validation_not_registered() {
        let auth = TokenAuth::new();

        // Not registered
        assert!(
            !auth
                .validate("not-registered-token-abcdefghijklmnopqrstuvwxyz")
                .await
        );
    }

    #[tokio::test]
    async fn test_token_registration() {
        let auth = TokenAuth::new();

        // Register a token
        auth.register(
            "test-token-abcdefghijklmnopqrstuvwxyz123456".to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        // Check token count
        assert_eq!(auth.token_count().await, 1);

        // Get token info
        let info = auth
            .get_token_info("test-token-abcdefghijklmnopqrstuvwxyz123456")
            .await;
        assert!(info.is_some());
        assert_eq!(info.unwrap().device_id, "device-1");
    }

    #[tokio::test]
    async fn test_token_revoke() {
        let auth = TokenAuth::new();

        // Register a token
        auth.register(
            "test-token-abcdefghijklmnopqrstuvwxyz123456".to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        // Revoke it
        auth.revoke("test-token-abcdefghijklmnopqrstuvwxyz123456")
            .await
            .unwrap();

        // Should no longer be valid
        assert!(
            !auth
                .validate("test-token-abcdefghijklmnopqrstuvwxyz123456")
                .await
        );
        assert_eq!(auth.token_count().await, 0);
    }
}
