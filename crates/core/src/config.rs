//! GearClaw Configuration
//!
//! This module defines the configuration structures with proper defaults
//! using derive macros and serde attributes for cleaner code.

use crate::error::GearClawError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Constants
// ============================================================================

/// Default OpenAI-compatible API endpoint
pub const DEFAULT_ENDPOINT: &str = "https://api.openai.com/v1";
/// Default embedding model
pub const DEFAULT_EMBEDDING_MODEL: &str = "embedding-3";
/// Default gateway host
pub const DEFAULT_GATEWAY_HOST: &str = "127.0.0.1";
/// Default gateway port
pub const DEFAULT_GATEWAY_PORT: u16 = 18789;
/// Default WebSocket path
pub const DEFAULT_WS_PATH: &str = "/ws";
/// Default session save interval (seconds)
pub const DEFAULT_SAVE_INTERVAL: u64 = 60;
/// Default max context tokens
pub const DEFAULT_MAX_TOKENS: usize = 200000;
/// Default agent name
pub const DEFAULT_AGENT_NAME: &str = "GearClaw";
/// Default system prompt
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"你是一个智能 AI 助手，名叫 GearClaw 🫞。

你的目标是帮助用户完成任务。你可以：
- 使用工具执行命令（在安全允许的范围内）
- 调用 LLM 进行推理和生成内容
- 管理会话上下文
- 提供编程帮助、调试、代码审查

请用友好、简洁的方式与用户交流。如果有不确定的地方，询问用户。"#;

// ============================================================================
// Helper functions for paths (required for serde defaults)
// ============================================================================

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn default_gearclaw_dir() -> PathBuf {
    home_dir().join(".gearclaw")
}

// ============================================================================
// Main Config
// ============================================================================

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM provider configuration
    pub llm: LLMConfig,
    /// Tool configuration
    pub tools: ToolsConfig,
    /// Session configuration
    pub session: SessionConfig,
    /// Agent configuration
    pub agent: AgentConfig,
    /// Memory configuration
    #[serde(default)]
    pub memory: MemoryConfig,
    /// MCP configuration
    #[serde(default)]
    pub mcp: McpConfig,
    /// Gateway configuration
    #[serde(default)]
    pub gateway: GatewayConfig,
}

// ============================================================================
// LLM Config
// ============================================================================

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Primary model (e.g., "openai/gpt-4", "zai/glm-4.7")
    pub primary: String,
    /// Fallback models
    #[serde(default)]
    pub fallbacks: Vec<String>,
    /// API endpoint
    #[serde(default = "LLMConfig::default_endpoint")]
    pub endpoint: String,
    /// API key (optional, can be loaded from env)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Embedding model
    #[serde(default = "LLMConfig::default_embedding_model")]
    pub embedding_model: String,
}

impl LLMConfig {
    fn default_endpoint() -> String {
        DEFAULT_ENDPOINT.to_string()
    }
    fn default_embedding_model() -> String {
        DEFAULT_EMBEDDING_MODEL.to_string()
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            primary: "gpt-4".to_string(),
            fallbacks: vec![],
            endpoint: DEFAULT_ENDPOINT.to_string(),
            api_key: None,
            embedding_model: DEFAULT_EMBEDDING_MODEL.to_string(),
        }
    }
}

// ============================================================================
// Tools Config
// ============================================================================

/// Tool execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Tool security level: deny, allowlist, full
    #[serde(default = "ToolsConfig::default_security")]
    pub security: String,
    /// Exec host: gateway, sandbox, node
    #[serde(default = "ToolsConfig::default_host")]
    pub host: String,
    /// Enable elevated tools
    #[serde(default)]
    pub elevated_enabled: bool,
    /// Tool profile: minimal, coding, messaging, full
    #[serde(default = "ToolsConfig::default_profile")]
    pub profile: String,
}

impl ToolsConfig {
    fn default_security() -> String {
        "full".to_string()
    }
    fn default_host() -> String {
        "gateway".to_string()
    }
    fn default_profile() -> String {
        "full".to_string()
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            security: "full".to_string(),
            host: "gateway".to_string(),
            elevated_enabled: false,
            profile: "full".to_string(),
        }
    }
}

// ============================================================================
// Session Config
// ============================================================================

/// Session persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session directory
    #[serde(default = "SessionConfig::default_session_dir")]
    pub session_dir: PathBuf,
    /// Auto-save interval in seconds
    #[serde(default = "SessionConfig::default_save_interval")]
    pub save_interval: u64,
    /// Maximum context tokens
    #[serde(default = "SessionConfig::default_max_tokens")]
    pub max_tokens: usize,
}

impl SessionConfig {
    fn default_session_dir() -> PathBuf {
        default_gearclaw_dir().join("sessions")
    }
    fn default_save_interval() -> u64 {
        DEFAULT_SAVE_INTERVAL
    }
    fn default_max_tokens() -> usize {
        DEFAULT_MAX_TOKENS
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_dir: Self::default_session_dir(),
            save_interval: DEFAULT_SAVE_INTERVAL,
            max_tokens: DEFAULT_MAX_TOKENS,
        }
    }
}

// ============================================================================
// Agent Config
// ============================================================================

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    #[serde(default = "AgentConfig::default_name")]
    pub name: String,
    /// System prompt
    #[serde(default = "AgentConfig::default_system_prompt")]
    pub system_prompt: String,
    /// Workspace directory
    #[serde(default = "AgentConfig::default_workspace")]
    pub workspace: PathBuf,
    /// Enable memory search
    #[serde(default)]
    pub memory_enabled: bool,
    /// Skills directory
    #[serde(default = "AgentConfig::default_skills_path")]
    pub skills_path: PathBuf,
    /// Channel trigger configuration
    #[serde(default)]
    pub triggers: AgentTriggerConfig,
}

impl AgentConfig {
    fn default_name() -> String {
        DEFAULT_AGENT_NAME.to_string()
    }
    fn default_system_prompt() -> String {
        DEFAULT_SYSTEM_PROMPT.to_string()
    }
    fn default_workspace() -> PathBuf {
        default_gearclaw_dir().join("workspace")
    }
    fn default_skills_path() -> PathBuf {
        default_gearclaw_dir().join("skills")
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: DEFAULT_AGENT_NAME.to_string(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            workspace: Self::default_workspace(),
            memory_enabled: false,
            skills_path: Self::default_skills_path(),
            triggers: AgentTriggerConfig::default(),
        }
    }
}

// ============================================================================
// Agent Trigger Config
// ============================================================================

/// Trigger mode for agent responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TriggerMode {
    /// Respond to all messages
    Always,
    /// Only respond when mentioned
    #[default]
    Mention,
    /// Only respond when keywords are matched
    Keyword,
}

/// Agent trigger configuration for channel messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTriggerConfig {
    /// Trigger mode: always, mention, or keyword
    #[serde(default)]
    pub mode: TriggerMode,
    /// Mention patterns (e.g., ["@agent", "@bot", "!ai"])
    #[serde(default = "AgentTriggerConfig::default_mention_patterns")]
    pub mention_patterns: Vec<String>,
    /// Keywords that trigger the agent (for keyword mode)
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Whitelist: only respond in these channels (format: "platform:channel_id")
    #[serde(default)]
    pub enabled_channels: Vec<String>,
    /// Blacklist: never respond in these channels (format: "platform:channel_id")
    #[serde(default)]
    pub disabled_channels: Vec<String>,
}

impl AgentTriggerConfig {
    fn default_mention_patterns() -> Vec<String> {
        vec!["@agent".to_string(), "@bot".to_string()]
    }
}

impl Default for AgentTriggerConfig {
    fn default() -> Self {
        Self {
            mode: TriggerMode::Mention,
            mention_patterns: Self::default_mention_patterns(),
            keywords: vec![],
            enabled_channels: vec![],
            disabled_channels: vec![],
        }
    }
}

// ============================================================================
// Memory Config
// ============================================================================

/// Memory/embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable memory system
    #[serde(default = "MemoryConfig::default_enabled")]
    pub enabled: bool,
    /// Database path for embeddings
    #[serde(default = "MemoryConfig::default_db_path")]
    pub db_path: PathBuf,
}

impl MemoryConfig {
    fn default_enabled() -> bool {
        true
    }
    fn default_db_path() -> PathBuf {
        default_gearclaw_dir().join("memory/index.sqlite")
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            db_path: Self::default_db_path(),
        }
    }
}

// ============================================================================
// MCP Config
// ============================================================================

/// MCP (Model Context Protocol) configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// MCP server configurations
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

/// Individual MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

// ============================================================================
// Gateway Config
// ============================================================================

/// Gateway server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Gateway host
    #[serde(default = "GatewayConfig::default_host")]
    pub host: String,
    /// Gateway port
    #[serde(default = "GatewayConfig::default_port")]
    pub port: u16,
    /// WebSocket path
    #[serde(default = "GatewayConfig::default_ws_path")]
    pub ws_path: String,
    /// Device key path
    #[serde(default = "GatewayConfig::default_device_key_path")]
    pub device_key_path: PathBuf,
    /// Auto-start on boot
    #[serde(default)]
    pub auto_start: bool,
    /// Enable TLS
    #[serde(default)]
    pub tls_enabled: bool,
    /// TLS certificate path
    #[serde(default)]
    pub tls_cert_path: Option<PathBuf>,
    /// TLS key path
    #[serde(default)]
    pub tls_key_path: Option<PathBuf>,
}

impl GatewayConfig {
    fn default_host() -> String {
        DEFAULT_GATEWAY_HOST.to_string()
    }
    fn default_port() -> u16 {
        DEFAULT_GATEWAY_PORT
    }
    fn default_ws_path() -> String {
        DEFAULT_WS_PATH.to_string()
    }
    fn default_device_key_path() -> PathBuf {
        default_gearclaw_dir().join("device.key")
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_GATEWAY_HOST.to_string(),
            port: DEFAULT_GATEWAY_PORT,
            ws_path: DEFAULT_WS_PATH.to_string(),
            device_key_path: Self::default_device_key_path(),
            auto_start: false,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

// ============================================================================
// Config Loading and Validation
// ============================================================================

/// Configuration loader with validation
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from file or default locations
    pub fn load(path: Option<&str>) -> Result<Config, GearClawError> {
        let config_path = Self::resolve_config_path(path)?;
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| GearClawError::config_parse_error(format!("Failed to read: {}", e)))?;
        let config: Config = serde_yml::from_str(&content)
            .map_err(|e| GearClawError::config_parse_error(format!("Failed to parse: {}", e)))?;
        Ok(config)
    }

    /// Resolve configuration file path
    fn resolve_config_path(path: Option<&str>) -> Result<PathBuf, GearClawError> {
        if let Some(p) = path {
            return Ok(PathBuf::from(p));
        }

        let default_paths = [
            home_dir().join(".gearclaw/config.toml"),
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gearclaw.toml"),
            PathBuf::from("./gearclaw.toml"),
        ];

        default_paths
            .into_iter()
            .find(|p| p.exists())
            .ok_or_else(|| {
                GearClawError::config_not_found(
                    "Config not found. Run `gearclaw init` or create ~/.gearclaw/config.toml",
                )
            })
    }
}

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate configuration
    pub fn validate(config: &Config) -> Result<(), GearClawError> {
        // Validate LLM config
        if config.llm.primary.is_empty() {
            return Err(GearClawError::Domain(crate::error::DomainError::ConfigInvalid {
                field: "llm.primary".to_string(),
                reason: "Primary model cannot be empty".to_string(),
            }));
        }

        // Validate security level
        let valid_security = ["deny", "allowlist", "full"];
        if !valid_security.contains(&config.tools.security.as_str()) {
            return Err(GearClawError::Domain(crate::error::DomainError::ConfigInvalid {
                field: "tools.security".to_string(),
                reason: format!(
                    "Invalid security level '{}'. Must be one of: {:?}",
                    config.tools.security, valid_security
                ),
            }));
        }

        Ok(())
    }
}

// ============================================================================
// Config Implementation (backward compatibility)
// ============================================================================

impl Config {
    /// Load configuration (backward compatibility wrapper)
    pub fn load(path: &Option<String>) -> Result<Self, GearClawError> {
        ConfigLoader::load(path.as_deref())
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<(), GearClawError> {
        let content = serde_yml::to_string(self)
            .map_err(|e| GearClawError::config_parse_error(format!("Serialization failed: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate a sample configuration
    pub fn sample() -> Self {
        Config {
            llm: LLMConfig {
                primary: "zai/glm-4.7".to_string(),
                fallbacks: vec![
                    "openai/gpt-4".to_string(),
                    "anthropic/claude-3-opus".to_string(),
                ],
                endpoint: DEFAULT_ENDPOINT.to_string(),
                api_key: None,
                embedding_model: DEFAULT_EMBEDDING_MODEL.to_string(),
            },
            tools: ToolsConfig {
                security: "full".to_string(),
                host: "gateway".to_string(),
                elevated_enabled: true,
                profile: "full".to_string(),
            },
            session: SessionConfig {
                session_dir: default_gearclaw_dir().join("sessions"),
                save_interval: DEFAULT_SAVE_INTERVAL,
                max_tokens: DEFAULT_MAX_TOKENS,
            },
            agent: AgentConfig {
                name: DEFAULT_AGENT_NAME.to_string(),
                system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
                workspace: default_gearclaw_dir().join("workspace"),
                memory_enabled: true,
                skills_path: default_gearclaw_dir().join("skills"),
                triggers: AgentTriggerConfig::default(),
            },
            memory: MemoryConfig::default(),
            mcp: McpConfig::default(),
            gateway: GatewayConfig::default(),
        }
    }
}

// ============================================================================
// Backward Compatibility
// ============================================================================

/// Backward compatibility: default_endpoint function
pub fn default_endpoint() -> String {
    DEFAULT_ENDPOINT.to_string()
}

