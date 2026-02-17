//! Compatibility wrapper for memory subsystem.
//! Delegates to `gearclaw_memory` while preserving `gearclaw_core` API.
use crate::config::MemoryConfig as CoreMemoryConfig;
use crate::error::GearClawError;
use crate::llm::LLMClient;
use std::sync::Arc;

pub use gearclaw_memory::SearchResult;

#[derive(Clone)]
pub struct MemoryManager {
    inner: gearclaw_memory::MemoryManager,
}

impl MemoryManager {
    pub fn new(
        config: CoreMemoryConfig,
        workspace_path: std::path::PathBuf,
        llm_client: Arc<LLMClient>,
    ) -> Result<Self, GearClawError> {
        let inner = gearclaw_memory::MemoryManager::new(
            to_memory_config(config),
            workspace_path,
            llm_client,
        )
        .map_err(|e| GearClawError::Other(e.to_string()))?;
        Ok(Self { inner })
    }

    pub async fn sync(&self) -> Result<(), GearClawError> {
        self.inner
            .sync()
            .await
            .map_err(|e| GearClawError::Other(e.to_string()))
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, GearClawError> {
        self.inner
            .search(query, limit)
            .await
            .map_err(|e| GearClawError::Other(e.to_string()))
    }
}

fn to_memory_config(config: CoreMemoryConfig) -> gearclaw_memory::MemoryConfig {
    gearclaw_memory::MemoryConfig {
        enabled: config.enabled,
        db_path: config.db_path,
    }
}
