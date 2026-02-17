use crate::config::MemoryConfig;
use crate::error::GearClawError;
use crate::llm::LLMClient;
use glob::glob;
use rusqlite::{params, Connection, OptionalExtension, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[derive(Clone)]
pub struct MemoryManager {
    config: MemoryConfig,
    conn: Arc<Mutex<Connection>>,
    workspace_path: PathBuf,
    llm_client: Arc<LLMClient>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub text: String,
    pub score: f32,
    pub start_line: Option<usize>,
}

impl MemoryManager {
    pub fn new(
        config: MemoryConfig,
        workspace_path: PathBuf,
        llm_client: Arc<LLMClient>,
    ) -> Result<Self, GearClawError> {
        let db_path = &config.db_path;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(GearClawError::IoError)?;
        }

        let conn = Connection::open(db_path)?;

        let manager = MemoryManager {
            config,
            conn: Arc::new(Mutex::new(conn)),
            workspace_path,
            llm_client,
        };

        manager.init_schema()?;

        Ok(manager)
    }

    fn init_schema(&self) -> Result<(), GearClawError> {
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

    pub async fn sync(&self) -> Result<(), GearClawError> {
        if !self.config.enabled {
            return Ok(());
        }

        info!("开始同步记忆...");

        // Find markdown files in workspace
        let pattern = self.workspace_path.join("**/*.md");
        let pattern_str = pattern
            .to_str()
            .ok_or_else(|| GearClawError::ConfigParseError("Invalid workspace path".to_string()))?;

        let mut files_to_process = Vec::new();
        let mut current_paths = std::collections::HashSet::new();

        for entry in glob(pattern_str).map_err(|e| {
            GearClawError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
        })? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
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
                            .unwrap()
                            .as_secs() as i64;
                        let size = metadata.len();

                        // Check if changed
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
                                Some((_hash, old_mtime)) => mtime > old_mtime, // Simple mtime check for now
                                None => true,
                            }
                        };

                        if should_process {
                            files_to_process.push((path, rel_path, mtime, size));
                        }
                    }
                }
                Err(e) => warn!("Glob error: {}", e),
            }
        }

        // Remove deleted files
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

        // Process new/changed files
        for (abs_path, rel_path, mtime, size) in files_to_process {
            info!("Indexing file: {}", rel_path);

            let content = fs::read_to_string(&abs_path)?;
            let hash = format!("{:x}", Sha256::digest(content.as_bytes()));

            // Simple chunking: split by paragraphs (double newline)
            // Ideally we should use a proper chunker like openclaw's
            let chunks: Vec<&str> = content
                .split("\n\n")
                .filter(|s| !s.trim().is_empty())
                .collect();

            // Process chunks
            let mut chunk_entries = Vec::new();
            for (i, chunk_text) in chunks.iter().enumerate() {
                // Get embedding
                let embedding = self.llm_client.get_embedding(chunk_text).await?;
                let embedding_json = serde_json::to_string(&embedding).unwrap();

                let chunk_id = format!(
                    "{:x}",
                    Sha256::digest(format!("{}:{}:{}", rel_path, i, chunk_text).as_bytes())
                );

                chunk_entries.push((chunk_id, chunk_text.to_string(), embedding_json, i));
            }

            // Update DB transactionally
            {
                let mut conn = self.conn.lock().unwrap();
                let tx = conn.transaction()?;

                // Remove old chunks
                tx.execute("DELETE FROM chunks WHERE path = ?", params![rel_path])?;

                // Insert new chunks
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO chunks (id, path, source, text, embedding, start_line) VALUES (?, ?, ?, ?, ?, ?)"
                    )?;

                    for (id, text, emb, idx) in chunk_entries {
                        stmt.execute(params![id, rel_path, "workspace", text, emb, idx])?;
                    }
                }

                // Update file record
                tx.execute(
                    "INSERT OR REPLACE INTO files (path, source, hash, mtime, size) VALUES (?, ?, ?, ?, ?)",
                    params![rel_path, "workspace", hash, mtime, size]
                )?;

                tx.commit()?;
            }
        }

        info!("Memory sync completed.");
        Ok(())
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, GearClawError> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let query_embedding = self.llm_client.get_embedding(query).await?;

        // Load all embeddings from DB (Naive approach for V1)
        // For production, use sqlite-vec or load into memory structure on sync
        let chunks = {
            let conn = self.conn.lock().unwrap();
            let mut stmt =
                conn.prepare("SELECT id, path, text, embedding, start_line FROM chunks")?;

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

        // Calculate Cosine Similarity
        let mut scored: Vec<SearchResult> = chunks
            .into_iter()
            .map(|(_id, path, text, embedding, start_line)| {
                let score = cosine_similarity(&query_embedding, &embedding);
                SearchResult {
                    path,
                    text,
                    score,
                    start_line,
                }
            })
            .collect();

        // Sort by score desc
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
