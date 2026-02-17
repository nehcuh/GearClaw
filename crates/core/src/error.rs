//! GearClaw Error Types
//!
//! This module defines a layered error hierarchy:
//! - `DomainError`: Business logic errors (config, tool, session, memory)
//! - `InfraError`: Infrastructure errors (IO, network, database, serialization)
//! - `GearClawError`: Top-level error that wraps both categories

use std::fmt;

/// Domain-level errors representing business logic failures
#[derive(Debug)]
pub enum DomainError {
    /// Configuration file not found at the specified path
    ConfigNotFound { path: String },
    /// Configuration validation failed
    ConfigInvalid { field: String, reason: String },
    /// Configuration parsing failed
    ConfigParse { source: String },
    /// LLM request failed
    LLMRequest { message: String },
    /// LLM response was invalid or unexpected
    LLMResponse { message: String },
    /// Tool execution failed
    ToolExecution { tool: String, reason: String },
    /// Tool not found in registry
    ToolNotFound { name: String },
    /// Session operation failed
    Session { operation: String, reason: String },
    /// Memory/embedding operation failed
    Memory { operation: String, reason: String },
    /// MCP protocol error
    Mcp { server: String, reason: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound { path } => write!(f, "Config not found: {}", path),
            Self::ConfigInvalid { field, reason } => {
                write!(f, "Config invalid [{}]: {}", field, reason)
            }
            Self::ConfigParse { source } => write!(f, "Config parse error: {}", source),
            Self::LLMRequest { message } => write!(f, "LLM request failed: {}", message),
            Self::LLMResponse { message } => write!(f, "LLM response error: {}", message),
            Self::ToolExecution { tool, reason } => {
                write!(f, "Tool '{}' execution failed: {}", tool, reason)
            }
            Self::ToolNotFound { name } => write!(f, "Tool not found: {}", name),
            Self::Session { operation, reason } => {
                write!(f, "Session {} failed: {}", operation, reason)
            }
            Self::Memory { operation, reason } => {
                write!(f, "Memory {} failed: {}", operation, reason)
            }
            Self::Mcp { server, reason } => write!(f, "MCP [{}] error: {}", server, reason),
        }
    }
}

impl std::error::Error for DomainError {}

/// Infrastructure-level errors representing external system failures
#[derive(Debug)]
pub enum InfraError {
    /// IO operation failed
    Io(std::io::Error),
    /// JSON serialization/deserialization failed
    Json(serde_json::Error),
    /// YAML serialization/deserialization failed
    Yaml(serde_yml::Error),
    /// Database operation failed
    Database(rusqlite::Error),
    /// Network/HTTP request failed
    Network { url: String, reason: String },
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Json(err) => write!(f, "JSON error: {}", err),
            Self::Yaml(err) => write!(f, "YAML error: {}", err),
            Self::Database(err) => write!(f, "Database error: {}", err),
            Self::Network { url, reason } => write!(f, "Network error [{}]: {}", url, reason),
        }
    }
}

impl std::error::Error for InfraError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            Self::Yaml(e) => Some(e),
            Self::Database(e) => Some(e),
            Self::Network { .. } => None,
        }
    }
}

impl From<std::io::Error> for InfraError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for InfraError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<serde_yml::Error> for InfraError {
    fn from(err: serde_yml::Error) -> Self {
        Self::Yaml(err)
    }
}

impl From<rusqlite::Error> for InfraError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err)
    }
}

/// Top-level error type for GearClaw
///
/// This enum wraps both domain and infrastructure errors, providing
/// a unified error type for the entire application while preserving
/// the ability to handle specific error categories.
#[derive(Debug)]
pub enum GearClawError {
    /// Business logic error
    Domain(DomainError),
    /// Infrastructure/external system error
    Infra(InfraError),
    /// Generic error for edge cases
    Other(String),
}

impl fmt::Display for GearClawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(e) => write!(f, "{}", e),
            Self::Infra(e) => write!(f, "{}", e),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for GearClawError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Domain(e) => Some(e),
            Self::Infra(e) => Some(e),
            Self::Other(_) => None,
        }
    }
}

impl From<DomainError> for GearClawError {
    fn from(err: DomainError) -> Self {
        Self::Domain(err)
    }
}

impl From<InfraError> for GearClawError {
    fn from(err: InfraError) -> Self {
        Self::Infra(err)
    }
}

// Convenience conversions from infrastructure error sources
impl From<std::io::Error> for GearClawError {
    fn from(err: std::io::Error) -> Self {
        Self::Infra(InfraError::Io(err))
    }
}

impl From<serde_json::Error> for GearClawError {
    fn from(err: serde_json::Error) -> Self {
        Self::Infra(InfraError::Json(err))
    }
}

impl From<serde_yml::Error> for GearClawError {
    fn from(err: serde_yml::Error) -> Self {
        Self::Infra(InfraError::Yaml(err))
    }
}

impl From<rusqlite::Error> for GearClawError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Infra(InfraError::Database(err))
    }
}

impl From<gearclaw_llm::LlmError> for GearClawError {
    fn from(err: gearclaw_llm::LlmError) -> Self {
        match err {
            gearclaw_llm::LlmError::Request(message) => Self::llm_error(message),
            gearclaw_llm::LlmError::Response(message) => Self::llm_response_error(message),
            gearclaw_llm::LlmError::Json(source) => Self::from(source),
        }
    }
}

// ============================================================================
// Backward Compatibility Layer
// ============================================================================
// The following associated functions and type aliases provide backward
// compatibility with existing code that uses the old error variants.
// These should be gradually migrated to the new layered error types.

impl GearClawError {
    /// Create a config not found error (backward compatibility)
    pub fn config_not_found(path: impl Into<String>) -> Self {
        Self::Domain(DomainError::ConfigNotFound { path: path.into() })
    }

    /// Create a config parse error (backward compatibility)
    pub fn config_parse_error(source: impl Into<String>) -> Self {
        Self::Domain(DomainError::ConfigParse {
            source: source.into(),
        })
    }

    /// Create an LLM error (backward compatibility)
    pub fn llm_error(message: impl Into<String>) -> Self {
        Self::Domain(DomainError::LLMRequest {
            message: message.into(),
        })
    }

    /// Create an LLM response error (backward compatibility)
    pub fn llm_response_error(message: impl Into<String>) -> Self {
        Self::Domain(DomainError::LLMResponse {
            message: message.into(),
        })
    }

    /// Create a tool execution error (backward compatibility)
    pub fn tool_execution_error(reason: impl Into<String>) -> Self {
        Self::Domain(DomainError::ToolExecution {
            tool: "unknown".to_string(),
            reason: reason.into(),
        })
    }

    /// Create a tool not found error (backward compatibility)
    pub fn tool_not_found(name: impl Into<String>) -> Self {
        Self::Domain(DomainError::ToolNotFound { name: name.into() })
    }
}

// Legacy type aliases for backward compatibility
// These allow existing code to continue using the old variant names
#[allow(non_snake_case)]
impl GearClawError {
    /// Legacy: ConfigNotFound variant constructor
    #[inline]
    pub fn ConfigNotFound(path: String) -> Self {
        Self::config_not_found(path)
    }

    /// Legacy: ConfigParseError variant constructor
    #[inline]
    pub fn ConfigParseError(msg: String) -> Self {
        Self::config_parse_error(msg)
    }

    /// Legacy: LLMError variant constructor
    #[inline]
    pub fn LLMError(msg: String) -> Self {
        Self::llm_error(msg)
    }

    /// Legacy: LLMResponseError variant constructor
    #[inline]
    pub fn LLMResponseError(msg: String) -> Self {
        Self::llm_response_error(msg)
    }

    /// Legacy: ToolExecutionError variant constructor
    #[inline]
    pub fn ToolExecutionError(msg: String) -> Self {
        Self::tool_execution_error(msg)
    }

    /// Legacy: ToolNotFound variant constructor
    #[inline]
    pub fn ToolNotFound(name: String) -> Self {
        Self::tool_not_found(name)
    }

    /// Legacy: IoError variant constructor
    #[inline]
    pub fn IoError(err: std::io::Error) -> Self {
        Self::from(err)
    }

    /// Legacy: SerdeError variant constructor
    #[inline]
    pub fn SerdeError(err: serde_json::Error) -> Self {
        Self::from(err)
    }

    /// Legacy: YamlError variant constructor
    #[inline]
    pub fn YamlError(err: serde_yml::Error) -> Self {
        Self::from(err)
    }

    /// Legacy: DatabaseError variant constructor
    #[inline]
    pub fn DatabaseError(err: rusqlite::Error) -> Self {
        Self::from(err)
    }
}
