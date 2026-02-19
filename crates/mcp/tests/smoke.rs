use gearclaw_mcp::{McpConfig, McpManager, McpServerConfig};
use std::collections::HashMap;

#[test]
fn mcp_smoke_config_default() {
    let config = McpConfig::default();
    assert!(config.servers.is_empty());
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
        },
    );

    let config = McpConfig { servers };
    let _manager = McpManager::new(config);
}
