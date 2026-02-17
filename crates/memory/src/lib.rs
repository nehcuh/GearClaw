use async_trait::async_trait;
use gearclaw_llm::LLMClient;
use glob::glob;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default)]
    pub enabled: bool,
    pub db_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub text: String,
    pub score: f32,
    pub start_line: Option<usize>,
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Clone)]
pub struct MemoryManager {
    config: MemoryConfig,
    conn: Arc<Mutex<Connection>>,
    workspace_path: PathBuf,
    llm_client: Arc<LLMClient>,
}

impl MemoryManager {
    pub fn new(
        config: MemoryConfig,
        workspace_path: PathBuf,
        llm_client: Arc<LLMClient>,
    ) -> Result<Self, MemoryError> {
        let db_path = &config.db_path;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        let manager = Self {
            config,
            conn: Arc::new(Mutex::new(conn)),
            workspace_path,
            llm_client,
        };
        manager.init_schema()?;
        Ok(manager)
    }

    fn init_schema(&self) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                hash TEXT NOT NULL,
                mtime INTEGER NOT NULL,
                size INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                source TEXT NOT NULL,
                text TEXT NOT NULL,
                embedding TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub async fn sync(&self) -> Result<(), MemoryError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("开始同步记忆...");
        let pattern = self.workspace_path.join("**/*.md");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| MemoryError::Other("Invalid workspace path".to_string()))?;

        let mut files_to_process = Vec::new();
        let mut current_paths = HashSet::new();

        for entry in glob(pattern_str).map_err(|e| MemoryError::Other(e.to_string()))? {
            match entry {
                Ok(path) if path.is_file() => {
                    let rel_path = path
                        .strip_prefix(&self.workspace_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    current_paths.insert(rel_path.clone());

                    let metadata = fs::metadata(&path)?;
                    let mtime = metadata
                        .modified()?
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| MemoryError::Other(e.to_string()))?
                        .as_secs() as i64;
                    let size = metadata.len();

                    let should_process = {
                        let conn = self.conn.lock().unwrap();
                        let existing: Option<(String, i64)> = conn
                            .query_row(
                                "SELECT hash, mtime FROM files WHERE path = ?",
                                params![rel_path],
                                |row| Ok((row.get(0)?, row.get(1)?)),
                            )
                            .optional()?;
                        match existing {
                            Some((_hash, old_mtime)) => mtime > old_mtime,
                            None => true,
                        }
                    };

                    if should_process {
                        files_to_process.push((path, rel_path, mtime, size));
                    }
                }
                Ok(_) => {}
                Err(e) => warn!("Glob error: {}", e),
            }
        }

        {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT path FROM files WHERE source = 'workspace'")?;
            let stored_paths: Vec<String> = stmt
                .query_map([], |row| row.get(0))?
                .filter_map(Result::ok)
                .collect();

            for path in stored_paths {
                if !current_paths.contains(&path) {
                    info!("Removing deleted file from memory: {}", path);
                    conn.execute("DELETE FROM files WHERE path = ?", params![path])?;
                    conn.execute("DELETE FROM chunks WHERE path = ?", params![path])?;
                }
            }
        }

        for (abs_path, rel_path, mtime, size) in files_to_process {
            info!("Indexing file: {}", rel_path);
            let content = fs::read_to_string(&abs_path)?;
            let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
            let chunks: Vec<&str> = content
                .split("\n\n")
                .filter(|s| !s.trim().is_empty())
                .collect();

            let mut chunk_entries = Vec::new();
            for (i, chunk_text) in chunks.iter().enumerate() {
                let embedding = self
                    .llm_client
                    .get_embedding(chunk_text)
                    .await
                    .map_err(|e| MemoryError::Llm(e.to_string()))?;
                let embedding_json = serde_json::to_string(&embedding)?;
                let chunk_id = format!(
                    "{:x}",
                    Sha256::digest(format!("{}:{}:{}", rel_path, i, chunk_text).as_bytes())
                );
                chunk_entries.push((chunk_id, chunk_text.to_string(), embedding_json, i));
            }

            {
                let mut conn = self.conn.lock().unwrap();
                let tx = conn.transaction()?;
                tx.execute("DELETE FROM chunks WHERE path = ?", params![rel_path])?;
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO chunks (id, path, source, text, embedding, start_line) VALUES (?, ?, ?, ?, ?, ?)",
                    )?;
                    for (id, text, emb, idx) in chunk_entries {
                        stmt.execute(params![id, rel_path, "workspace", text, emb, idx])?;
                    }
                }
                tx.execute(
                    "INSERT OR REPLACE INTO files (path, source, hash, mtime, size) VALUES (?, ?, ?, ?, ?)",
                    params![rel_path, "workspace", hash, mtime, size],
                )?;
                tx.commit()?;
            }
        }

        info!("Memory sync completed.");
        Ok(())
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, MemoryError> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }
        let query_embedding = self
            .llm_client
            .get_embedding(query)
            .await
            .map_err(|e| MemoryError::Llm(e.to_string()))?;

        let chunks = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT id, path, text, embedding, start_line FROM chunks")?;
            let rows = stmt
                .query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                let text: String = row.get(2)?;
                let emb_json: String = row.get(3)?;
                let start_line: Option<usize> = row.get(4)?;
                let embedding: Vec<f32> = serde_json::from_str(&emb_json).unwrap_or_default();
                Ok((id, path, text, embedding, start_line))
            })?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
            rows
        };

        let mut scored: Vec<SearchResult> = chunks
            .into_iter()
            .map(|(_id, path, text, embedding, start_line)| SearchResult {
                path,
                text,
                score: cosine_similarity(&query_embedding, &embedding),
                start_line,
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(scored.into_iter().take(limit).collect())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
pub trait MemoryIndex {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, MemoryError>;
}

#[async_trait]
impl MemoryIndex for MemoryManager {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, MemoryError> {
        MemoryManager::search(self, query, limit).await
    }
}
