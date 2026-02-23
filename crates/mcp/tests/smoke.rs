use gearclaw_mcp::{McpCapability, McpConfig, McpManager, McpServerConfig};
use std::collections::HashMap;

#[test]
fn mcp_smoke_config_default() {
    let config = McpConfig::default();
    assert!(config.servers.is_empty());
}

#[test]
fn mcp_smoke_manager_capability_is_enabled() {
    let manager = McpManager::new(McpConfig::default());
    assert_eq!(manager.capability(), McpCapability::Enabled);
    assert!(manager.is_enabled());
}

#[test]
fn mcp_smoke_manager_new_with_server_config() {
    let mut servers = HashMap::new();
    servers.insert(
        "test".to_string(),
        McpServerConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
            enabled: true,
        },
    );

    let config = McpConfig { servers };
    let manager = McpManager::new(config);
    // Not connected yet (init_clients not called)
    assert!(manager.clients.is_empty());
    assert_eq!(manager.config.servers.len(), 1);
}

#[test]
fn mcp_smoke_registry_has_entries() {
    let all = gearclaw_mcp::builtin_registry();
    assert!(!all.is_empty());

    // Each entry should have a non-empty id, name, and description
    for entry in all {
        assert!(!entry.id.is_empty());
        assert!(!entry.name.is_empty());
        assert!(!entry.description.is_empty());
    }
}

#[test]
fn mcp_smoke_registry_search() {
    let results = gearclaw_mcp::search_registry("database");
    assert!(!results.is_empty());
    // Should find postgres and sqlite at minimum
    let ids: Vec<&str> = results.iter().map(|e| e.id).collect();
    assert!(ids.contains(&"postgres") || ids.contains(&"sqlite"));
}

#[test]
fn mcp_smoke_registry_find_by_id() {
    let entry = gearclaw_mcp::find_by_id("filesystem");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().id, "filesystem");

    let missing = gearclaw_mcp::find_by_id("nonexistent_server_xyz");
    assert!(missing.is_none());
}
