use std::fmt;

#[derive(Debug)]
pub enum GearClawError {
    // Configuration errors
    ConfigNotFound(String),
    ConfigParseError(String),
    
    // LLM errors
    LLMError(String),
    LLMResponseError(String),
    
    // Tool execution errors
    ToolExecutionError(String),
    ToolNotFound(String),
    
    // Session errors
    // SessionError(String),
    
    // IO errors
    IoError(std::io::Error),
    
    // Serialization errors
    SerdeError(serde_json::Error),
    YamlError(serde_yaml::Error),

    // Database errors
    DatabaseError(rusqlite::Error),
}

impl fmt::Display for GearClawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GearClawError::ConfigNotFound(path) => {
                write!(f, "配置文件未找到: {}", path)
            }
            GearClawError::ConfigParseError(msg) => {
                write!(f, "配置解析错误: {}", msg)
            }
            GearClawError::LLMError(msg) => {
                write!(f, "LLM 错误: {}", msg)
            }
            GearClawError::LLMResponseError(msg) => {
                write!(f, "LLM 响应错误: {}", msg)
            }
            GearClawError::ToolExecutionError(msg) => {
                write!(f, "工具执行错误: {}", msg)
            }
            GearClawError::ToolNotFound(tool) => {
                write!(f, "工具未找到: {}", tool)
            }
            // GearClawError::SessionError(msg) => {
            //     write!(f, "会话错误: {}", msg)
            // }
            GearClawError::IoError(err) => {
                write!(f, "IO 错误: {}", err)
            }
            GearClawError::SerdeError(err) => {
                write!(f, "序列化错误: {}", err)
            }
            GearClawError::YamlError(err) => {
                write!(f, "YAML 错误: {}", err)
            }
            GearClawError::DatabaseError(err) => {
                write!(f, "数据库错误: {}", err)
            }
        }
    }
}

impl std::error::Error for GearClawError {}

impl From<std::io::Error> for GearClawError {
    fn from(err: std::io::Error) -> Self {
        GearClawError::IoError(err)
    }
}

impl From<serde_json::Error> for GearClawError {
    fn from(err: serde_json::Error) -> Self {
        GearClawError::SerdeError(err)
    }
}

impl From<serde_yaml::Error> for GearClawError {
    fn from(err: serde_yaml::Error) -> Self {
        GearClawError::YamlError(err)
    }
}

impl From<rusqlite::Error> for GearClawError {
    fn from(err: rusqlite::Error) -> Self {
        GearClawError::DatabaseError(err)
    }
}
