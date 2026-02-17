//! gearclaw_memory
//! Compatibility extraction crate for memory indexing/search.

use async_trait::async_trait;

pub use gearclaw_core::memory::*;

#[async_trait]
pub trait MemoryIndex {
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, gearclaw_core::GearClawError>;
}

#[async_trait]
impl MemoryIndex for MemoryManager {
    async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, gearclaw_core::GearClawError> {
        MemoryManager::search(self, query, limit).await
    }
}
