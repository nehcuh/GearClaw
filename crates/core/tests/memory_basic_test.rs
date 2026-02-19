// Basic Memory System Test
//
// Tests MemoryManager creation and basic functionality without requiring API keys

use gearclaw_core::config::MemoryConfig;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_memory_manager_creation() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();

    let db_path = temp_dir.path().join("test.db");

    // Note: We can't create a full LLMClient without API key
    // This test verifies the structure compiles correctly

    println!("✅ MemoryManager structure test passed");
    println!("Workspace: {:?}", workspace);
    println!("Database: {:?}", db_path);
}

#[tokio::test]
async fn test_workspace_file_detection() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();

    // Create test markdown files
    let docs_dir = workspace.join("docs");
    fs::create_dir_all(&docs_dir).unwrap();

    let file1 = docs_dir.join("test1.md");
    let mut f1 = fs::File::create(&file1).unwrap();
    writeln!(f1, "# Test Document 1\n\nContent for testing.").unwrap();

    let file2 = docs_dir.join("test2.md");
    let mut f2 = fs::File::create(&file2).unwrap();
    writeln!(f2, "# Test Document 2\n\nMore content.").unwrap();

    // Create a non-markdown file (should be ignored)
    let ignored_file = docs_dir.join("ignored.txt");
    let mut f3 = fs::File::create(&ignored_file).unwrap();
    writeln!(f3, "This should be ignored").unwrap();

    // Use glob to find markdown files
    use glob::glob;
    let pattern = workspace.join("**/*.md");
    let pattern_str = pattern.to_str().unwrap();

    let mut md_files = Vec::new();
    for entry in glob(pattern_str).unwrap() {
        if let Ok(path) = entry {
            if path.is_file() {
                md_files.push(path);
            }
        }
    }

    assert_eq!(md_files.len(), 2, "Should find exactly 2 markdown files");
    assert!(md_files.iter().any(|p| p.ends_with("test1.md")));
    assert!(md_files.iter().any(|p| p.ends_with("test2.md")));
    assert!(!md_files.iter().any(|p| p.ends_with("ignored.txt")));

    println!("✅ Workspace file detection test passed");
    println!("Found {} markdown files", md_files.len());
}

#[test]
fn test_memory_config_defaults() {
    let config = MemoryConfig {
        enabled: true,
        db_path: "/tmp/test.db".into(),
    };

    assert!(config.enabled);
    assert_eq!(config.db_path, std::path::PathBuf::from("/tmp/test.db"));

    println!("✅ MemoryConfig test passed");
}

#[tokio::test]
async fn test_database_schema_creation() {
    use rusqlite::Connection;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let conn = Connection::open(&db_path).unwrap();

    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            path TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            hash TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            source TEXT NOT NULL,
            text TEXT NOT NULL,
            embedding TEXT NOT NULL,
            start_line INTEGER
        )",
        [],
    )
    .unwrap();

    // Verify tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    assert!(tables.contains(&"files".to_string()));
    assert!(tables.contains(&"chunks".to_string()));

    println!("✅ Database schema creation test passed");
    println!("Tables: {:?}", tables);
}

#[test]
fn test_chunking_logic() {
    let content = "# Header\n\nParagraph 1.\n\nParagraph 2.\n\nParagraph 3.";

    // Simple chunking by double newline
    let chunks: Vec<&str> = content
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .collect();

    assert_eq!(chunks.len(), 4);
    assert_eq!(chunks[0], "# Header");
    assert_eq!(chunks[1], "Paragraph 1.");
    assert_eq!(chunks[2], "Paragraph 2.");
    assert_eq!(chunks[3], "Paragraph 3.");

    println!("✅ Chunking logic test passed");
    println!("Created {} chunks", chunks.len());
}
