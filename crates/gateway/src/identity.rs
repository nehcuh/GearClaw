// Device Identity Management
//
// This module implements device identity using ed25519 public key cryptography.
// Each device has a unique keypair used for signing requests and verifying identity.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

/// Device identity keypair
#[derive(Clone)]
pub struct DeviceKeyPair {
    /// The actual ed25519 signing key
    signing_key: SigningKey,
    /// Base64URL-encoded public key (for serialization)
    pub public_key: String,
    /// Base64URL-encoded secret key (for persistence)
    pub secret_key: String,
}

impl DeviceKeyPair {
    /// Generate a new random ed25519 keypair using cryptographically secure RNG
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();
        let public_bytes = verifying_key.to_bytes();
        let secret_bytes = signing_key.to_bytes();

        Self {
            signing_key,
            public_key: base64_url_encode(&public_bytes),
            secret_key: base64_url_encode(&secret_bytes),
        }
    }

    /// Get public key as Base64URL-encoded string
    pub fn public_key_base64(&self) -> String {
        self.public_key.clone()
    }

    /// Load existing keypair from file, or generate a new one if it doesn't exist
    pub fn load_or_generate(path: &std::path::PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let keypair = Self::generate();
            keypair.save(path)?;
            Ok(keypair)
        }
    }

    /// Load keypair from file
    pub fn load(path: &std::path::PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;

        // Parse JSON to get public_key and secret_key strings
        let data: serde_json::Value = serde_json::from_str(&content)?;

        let public_key = data["public_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing public_key in file"))?;
        let secret_key = data["secret_key"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing secret_key in file"))?;

        // Decode the secret key to reconstruct the signing key
        let secret_bytes = base64_url_decode(secret_key)?;
        let signing_key: SigningKey = secret_bytes[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid secret key length"))?;

        // Derive verifying key from signing key to verify public_key matches
        let verifying_key = signing_key.verifying_key();
        let expected_public_bytes = verifying_key.to_bytes();
        let actual_public_bytes = base64_url_decode(public_key)?;

        if expected_public_bytes[..] != actual_public_bytes[..] {
            return Err(anyhow::anyhow!("Public key in file does not match secret key"));
        }

        Ok(Self {
            signing_key,
            public_key: public_key.to_string(),
            secret_key: secret_key.to_string(),
        })
    }

    /// Save keypair to file (with secure permissions on Unix)
    pub fn save(&self, path: &std::path::PathBuf) -> anyhow::Result<()> {
        // Serialize to JSON (only the string fields, not the actual Keypair)
        let data = serde_json::json!({
            "public_key": self.public_key,
            "secret_key": self.secret_key,
        });
        let content = serde_json::to_string_pretty(&data)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to file
        std::fs::write(path, content)?;

        // Set file permissions to 0600 (owner read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
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

    /// Signature (optional, not included in signed data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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
            signature: None,
        }
    }

    /// Sign the identity claims using ed25519
    pub fn sign(&mut self, keypair: &DeviceKeyPair) -> anyhow::Result<String> {
        // 1. Serialize identity claims to JSON (without signature field)
        let mut identity_copy = self.clone();
        identity_copy.signature = None;
        let claims_json = serde_json::to_string(&identity_copy)?;

        // 2. Sign using ed25519
        let signature_bytes = keypair.signing_key.sign(claims_json.as_bytes());

        // 3. Base64URL encode the signature
        let signature_b64 = base64_url_encode(signature_bytes.to_bytes().as_ref());

        // 4. Update self with the signature
        self.signature = Some(signature_b64.clone());

        Ok(signature_b64)
    }

    /// Verify a signature using ed25519
    pub fn verify(&self, signature: &str, public_key: &str) -> anyhow::Result<bool> {
        use ed25519_dalek::Signature;

        // 1. Decode the public key
        let pub_bytes = base64_url_decode(public_key)?;
        let pub_key: VerifyingKey = pub_bytes[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid public key length"))?;

        // 2. Decode the signature
        let sig_bytes = base64_url_decode(signature)?;
        let sig: Signature = sig_bytes[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;

        // 3. Serialize identity without the signature field
        let mut identity_copy = self.clone();
        identity_copy.signature = None;
        let claims_json = serde_json::to_string(&identity_copy)?;

        // 4. Verify the signature
        Ok(pub_key.verify(claims_json.as_bytes(), &sig).is_ok())
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
    use tempfile::TempDir;

    #[test]
    fn test_ed25519_keypair_generation() {
        let keypair = DeviceKeyPair::generate();
        assert_eq!(keypair.public_key.len(), 44); // Base64 of 32 bytes
        assert_eq!(keypair.secret_key.len(), 44);

        // Verify that two generated keypairs are different
        let keypair2 = DeviceKeyPair::generate();
        assert_ne!(keypair.public_key, keypair2.public_key);
        assert_ne!(keypair.secret_key, keypair2.secret_key);
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
        assert!(identity.signature.is_none()); // Initially no signature
    }

    #[test]
    fn test_ed25519_sign_and_verify() {
        let keypair = DeviceKeyPair::generate();
        let mut identity = DeviceIdentity::new(
            "device-123".to_string(),
            "Test Device".to_string(),
            "macos".to_string(),
            keypair.public_key_base64(),
        );

        // Sign the identity
        let signature = identity.sign(&keypair).unwrap();
        assert_eq!(signature.len(), 88); // Base64 of 64 bytes ed25519 signature
        assert!(identity.signature.is_some()); // Signature should be stored

        // Verify with correct public key
        let verified = identity.verify(&signature, &keypair.public_key).unwrap();
        assert!(verified);

        // Verify that the signature is not just a prefix
        assert!(!signature.starts_with("signed_"));
    }

    #[test]
    fn test_verify_fails_with_wrong_key() {
        let keypair1 = DeviceKeyPair::generate();
        let keypair2 = DeviceKeyPair::generate();

        let mut identity = DeviceIdentity::new(
            "device-123".to_string(),
            "Test Device".to_string(),
            "macos".to_string(),
            keypair1.public_key_base64(),
        );

        let signature = identity.sign(&keypair1).unwrap();

        // Verify with wrong public key should fail
        let verified = identity.verify(&signature, &keypair2.public_key).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_verify_fails_with_tampered_identity() {
        let keypair = DeviceKeyPair::generate();
        let mut identity = DeviceIdentity::new(
            "device-123".to_string(),
            "Test Device".to_string(),
            "macos".to_string(),
            keypair.public_key_base64(),
        );

        let signature = identity.sign(&keypair).unwrap();

        // Tamper with the identity after signing
        identity.device_id = "tampered-device".to_string();

        // Verification should fail
        let verified = identity.verify(&signature, &keypair.public_key).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_keypair_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test_key.json");

        // Generate and save
        let keypair1 = DeviceKeyPair::generate();
        keypair1.save(&key_path).unwrap();

        // Verify file exists and has correct permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&key_path).unwrap().permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        }

        // Load the keypair
        let keypair2 = DeviceKeyPair::load(&key_path).unwrap();
        assert_eq!(keypair1.public_key, keypair2.public_key);
        assert_eq!(keypair1.secret_key, keypair2.secret_key);
    }

    #[test]
    fn test_load_or_generates_creates_new_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test_key.json");

        assert!(!key_path.exists());

        // Should create new keypair
        let keypair = DeviceKeyPair::load_or_generate(&key_path).unwrap();
        assert!(key_path.exists());
        assert_eq!(keypair.public_key.len(), 44);
    }

    #[test]
    fn test_load_or_generates_loads_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test_key.json");

        // Create initial keypair
        let keypair1 = DeviceKeyPair::generate();
        keypair1.save(&key_path).unwrap();

        // Load should return the same keypair
        let keypair2 = DeviceKeyPair::load_or_generate(&key_path).unwrap();
        assert_eq!(keypair1.public_key, keypair2.public_key);
        assert_eq!(keypair1.secret_key, keypair2.secret_key);
    }
}
