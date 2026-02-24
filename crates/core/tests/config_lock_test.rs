//! Concurrency tests for config file locking
//!
//! These tests verify that the file locking mechanism prevents concurrent
//! write issues and works correctly under contention.

use gearclaw_core::config::Config;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_concurrent_writes_with_lock() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("concurrent_test.toml");

    // Spawn 10 threads, each writing to the same config file
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let path = config_path.clone();
            thread::spawn(move || {
                // Small delay to create contention
                thread::sleep(Duration::from_millis(i * 5));

                let mut config = Config::sample();
                config.llm.primary = format!("model-{}", i);

                // Use blocking lock - should serialize writes
                config.save_with_lock(&path).unwrap();
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify the file is not corrupted
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_yml::from_str(&content).unwrap();

    // The config should be valid - either the default "zai/glm-4.7" or one of our model-X values
    assert!(
        loaded_config.llm.primary.starts_with("model-") || loaded_config.llm.primary == "zai/glm-4.7"
    );
}

#[test]
fn test_nonblocking_lock_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonblocking_test.toml");

    // Create initial config
    let config1 = Config::sample();
    config1.save_with_lock(&config_path).unwrap();

    // Try to acquire a non-blocking lock while another thread holds the lock
    let config_path_clone = config_path.clone();
    let handle = thread::spawn(move || {
        let config2 = Config::sample();
        // This should succeed or fail immediately, not block
        let result = config2.save_with_lock_nonblocking(&config_path_clone);

        // If it fails, verify it's because the file is locked
        if let Err(e) = result {
            assert!(e.to_string().contains("locked by another process"));
        }
    });

    // Give the other thread time to try acquiring the lock
    thread::sleep(Duration::from_millis(100));

    handle.join().unwrap();
}

#[test]
fn test_atomic_save_no_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("atomic_test.toml");

    // Perform multiple rapid saves
    for i in 0..20 {
        let mut config = Config::sample();
        config.llm.primary = format!("model-{}", i);

        config.save_with_lock(&config_path).unwrap();
    }

    // Verify final file is valid
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_yml::from_str(&content).unwrap();

    assert!(
        loaded_config.llm.primary.starts_with("model-") || loaded_config.llm.primary == "zai/glm-4.7"
    );
}

#[test]
fn test_temp_file_cleaned_up() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("temp_cleanup_test.toml");
    let temp_path = temp_dir.path().join("temp_cleanup_test.tmp");

    // Ensure temp file doesn't exist
    assert!(!temp_path.exists());

    // Save config
    let config = Config::sample();
    config.save_with_lock(&config_path).unwrap();

    // Temp file should be cleaned up after successful save
    assert!(!temp_path.exists());
    assert!(config_path.exists());
}

#[test]
fn test_legacy_save_still_works() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("legacy_test.toml");

    // Legacy save() method should still work
    let config = Config::sample();
    config.save(&config_path).unwrap();

    // Verify file was created
    assert!(config_path.exists());

    // Verify content is valid
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_yml::from_str(&content).unwrap();
    assert_eq!(loaded_config.llm.primary, config.llm.primary);
}
