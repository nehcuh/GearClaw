use gearclaw_llm::LLMClient;
use gearclaw_memory::{MemoryConfig, MemoryManager};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_db_path() -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("gearclaw_memory_smoke_{}.db", stamp))
}

#[test]
fn memory_smoke_manager_new() {
    let db_path = unique_db_path();
    let config = MemoryConfig {
        enabled: false,
        db_path: db_path.clone(),
    };
    let llm_client = Arc::new(LLMClient::new(
        "test-key".to_string(),
        "https://example.com/v1".to_string(),
        "gpt-test".to_string(),
        "embed-test".to_string(),
    ));

    let manager = MemoryManager::new(config, std::env::temp_dir(), llm_client);
    assert!(manager.is_ok());

    let _ = std::fs::remove_file(db_path);
}
