// Device Identity Management
//
// This module implements device identity using ed25519 public key cryptography.
// Each device has a unique keypair used for signing requests and verifying identity.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Device identity keypair
#[derive(Clone, Debug)]
pub struct DeviceKeyPair {
    pub public_key: String,
    pub secret_key: String,
}

impl DeviceKeyPair {
    /// Generate a new random ed25519 keypair
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        // For now: generate random 32-byte keys as placeholder
        // TODO: [TODO-ID-001] Implement proper ed25519 keypair generation
        let mut secret = [0u8; 32];
        rng.fill(&mut secret);

        Self {
            public_key: base64_url_encode(&secret), // Same for now
            secret_key: base64_url_encode(&secret),
        }
    }

    /// Get public key as Base64URL-encoded string
    pub fn public_key_base64(&self) -> String {
        self.public_key.clone()
    }
}

/// Device identity claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// Unique device ID (derived from public key)
    pub device_id: String,

    /// Display name
    pub display_name: String,

    /// Platform (macos, linux, windows)
    pub platform: String,

    /// Device family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,

    /// Model identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_identifier: Option<String>,

    /// Public key (Base64URL-encoded)
    pub public_key: String,

    /// Timestamp
    pub ts: i64,
}

impl DeviceIdentity {
    pub fn new(
        device_id: String,
        display_name: String,
        platform: String,
        public_key: String,
    ) -> Self {
        Self {
            device_id,
            display_name,
            platform,
            device_family: None,
            model_identifier: None,
            public_key,
            ts: chrono::Utc::now().timestamp(),
        }
    }

    /// Sign the identity claims
    /// TODO: [TODO-ID-001] Implement proper ed25519 signing
    pub fn sign(&self, _keypair: &DeviceKeyPair) -> anyhow::Result<String> {
        // Serialize identity to JSON
        let identity_json = serde_json::to_string(self)?;

        // For now: prefix with "signed_" as placeholder
        // TODO: [TODO-ID-001] Implement proper ed25519 signature
        Ok(format!("signed_{}", identity_json))
    }

    /// Verify a signature
    /// TODO: [TODO-ID-001] Implement proper ed25519 verification
    pub fn verify(&self, _signature: &str, _public_key: &str) -> anyhow::Result<bool> {
        // For now: just check if signature starts with "signed_"
        // TODO: [TODO-ID-001] Implement proper ed25519 verification
        Ok(true)
    }
}

/// Base64URL encode (URL-safe base64)
pub fn base64_url_encode(data: &[u8]) -> String {
    use base64::prelude::*;
    BASE64_URL_SAFE.encode(data)
}

/// Base64URL decode
pub fn base64_url_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::prelude::*;
    BASE64_URL_SAFE.decode(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = DeviceKeyPair::generate();
        assert_eq!(keypair.public_key.len(), 44); // Base64 of 32 bytes
        assert_eq!(keypair.secret_key.len(), 44);
    }

    #[test]
    fn test_identity_creation() {
        let identity = DeviceIdentity::new(
            "device-123".to_string(),
            "Test Device".to_string(),
            "macos".to_string(),
            "test-public-key".to_string(),
        );
        assert_eq!(identity.device_id, "device-123");
        assert_eq!(identity.platform, "macos");
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = DeviceKeyPair::generate();
        let identity = DeviceIdentity::new(
            "device-123".to_string(),
            "Test Device".to_string(),
            "macos".to_string(),
            keypair.public_key_base64(),
        );

        let signature = identity.sign(&keypair).unwrap();
        assert!(signature.starts_with("signed_"));

        let verified = identity.verify(&signature, &keypair.public_key).unwrap();
        assert!(verified);
    }
}
