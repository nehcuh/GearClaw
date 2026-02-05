use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::GearClawError;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Primary model (e.g., "openai/gpt-4", "zai/glm-4.7")
    pub primary: String,
    
    /// Fallback models
    #[serde(default)]
    pub fallbacks: Vec<String>,
    
    /// API endpoint
    #[serde(default = "default_endpoint")]
    pub endpoint: String,
    
    /// API key (optional, can be loaded from env)
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Tool security level: deny, allowlist, full
    #[serde(default = "default_security")]
    pub security: String,
    
    /// Exec host: gateway, sandbox, node
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Enable elevated tools
    #[serde(default)]
    pub elevated_enabled: bool,
    
    /// Tool profile: minimal, coding, messaging, full
    #[serde(default = "default_profile")]
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session directory
    pub session_dir: PathBuf,
    
    /// Auto-save interval in seconds
    #[serde(default = "default_save_interval")]
    pub save_interval: u64,
    
    /// Maximum context tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name
    #[serde(default = "default_agent_name")]
    pub name: String,
    
    /// System prompt
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
    
    /// Workspace directory
    pub workspace: PathBuf,
    
    /// Enable memory search
    #[serde(default)]
    pub memory_enabled: bool,

    /// Skills directory
    #[serde(default = "default_skills_path")]
    pub skills_path: PathBuf,
}

fn default_skills_path() -> PathBuf {
    dirs::home_dir().unwrap().join(".gearclaw/skills")
}

fn default_endpoint() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_security() -> String {
    "full".to_string()
}

fn default_host() -> String {
    "gateway".to_string()
}

fn default_profile() -> String {
    "full".to_string()
}

fn default_save_interval() -> u64 {
    60
}

fn default_max_tokens() -> usize {
    200000
}

fn default_agent_name() -> String {
    "GearClaw".to_string()
}

fn default_system_prompt() -> String {
    r#"
你是一个智能 AI 助手，名叫 GearClaw 🦞。

你的目标是帮助用户完成任务。你可以：
- 使用工具执行命令（在安全允许的范围内）
- 调用 LLM 进行推理和生成内容
- 管理会话上下文
- 提供编程帮助、调试、代码审查

请用友好、简洁的方式与用户交流。如果有不确定的地方，询问用户。
"#.trim().to_string()
}

impl Config {
    pub fn load(path: &Option<String>) -> Result<Self, GearClawError> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            // Default locations
            let default_paths = vec![
                dirs::home_dir().map(|h| h.join(".gearclaw/config.toml")),
                dirs::home_dir().map(|h| h.join(".openclaw/gear_claw.toml")),
                dirs::config_dir().map(|c| c.join("gear_claw.toml")),
                Some(PathBuf::from("./gear_claw.toml")),
            ];
            
            default_paths
                .into_iter()
                .flatten()
                .find(|p| p.exists())
                .ok_or_else(|| GearClawError::ConfigNotFound(
                    "未找到配置文件。请运行 `gear_claw init` 进行初始化，或手动创建 ~/.gearclaw/config.toml".to_string()
                ))?
        };
        
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| GearClawError::ConfigParseError(format!("读取失败: {}", e)))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| GearClawError::ConfigParseError(format!("解析失败: {}", e)))?;
        
        Ok(config)
    }
    
    pub fn save(&self, path: &PathBuf) -> Result<(), GearClawError> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| GearClawError::ConfigParseError(format!("序列化失败: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(GearClawError::IoError)?;
        
        Ok(())
    }
    
    /// Generate a sample configuration file
    pub fn sample() -> Self {
        Config {
            llm: LLMConfig {
                primary: "zai/glm-4.7".to_string(),
                fallbacks: vec![
                    "openai/gpt-4".to_string(),
                    "anthropic/claude-3-opus".to_string(),
                ],
                endpoint: default_endpoint(),
                api_key: None,
            },
            tools: ToolsConfig {
                security: default_security(),
                host: default_host(),
                elevated_enabled: true,
                profile: default_profile(),
            },
            session: SessionConfig {
                session_dir: dirs::home_dir()
                    .unwrap()
                    .join(".gearclaw/sessions"),
                save_interval: default_save_interval(),
                max_tokens: default_max_tokens(),
            },
            agent: AgentConfig {
                name: default_agent_name(),
                system_prompt: default_system_prompt(),
                workspace: dirs::home_dir()
                    .unwrap()
                    .join(".gearclaw/workspace"),
                memory_enabled: true,
                skills_path: default_skills_path(),
            },
        }
    }
}
