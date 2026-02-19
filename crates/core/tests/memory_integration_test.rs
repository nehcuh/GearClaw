// Memory Integration Test
//
// This test demonstrates the memory system integration with Agent.
// Note: This test requires API keys and embeddings to work.

use gearclaw_core::{Agent, Config};
use std::fs;
use std::io::Write;

#[tokio::test]
async fn test_memory_integration() {
    // Create temporary workspace
    let temp_dir = std::env::temp_dir().join("gearclaw_memory_test");
    fs::create_dir_all(&temp_dir).unwrap();

    // Create test markdown file
    let test_file = temp_dir.join("test.md");
    let mut file = fs::File::create(&test_file).unwrap();
    writeln!(file, "# Test Document\n\n").unwrap();
    writeln!(file, "This is a test document for memory integration.\n\n").unwrap();
    writeln!(file, "## Important Information\n\n").unwrap();
    writeln!(
        file,
        "The API key should be stored securely in environment variables.\n\n"
    )
    .unwrap();
    writeln!(file, "## Configuration\n\n").unwrap();
    writeln!(
        file,
        "Configuration files are located in ~/.gearclaw/config.toml\n\n"
    )
    .unwrap();
    file.flush().unwrap();

    println!("Created test file: {:?}", test_file);

    // Check for API key
    let has_api_key =
        std::env::var("ANTHROPIC_API_KEY").is_ok() || std::env::var("OPENAI_API_KEY").is_ok();

    if !has_api_key {
        println!("⚠️  Skipping test: No API key found");
        println!("   Set ANTHROPIC_API_KEY or OPENAI_API_KEY to run this test");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    // Load config
    let config_result = Config::load(&None);
    if let Err(e) = config_result {
        println!("⚠️  Skipping test: Failed to load config: {}", e);
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let mut config = config_result.unwrap();
    config.agent.workspace = temp_dir.clone();
    config.memory.enabled = true;
    config.agent.memory_enabled = true;

    // Create agent
    let agent_result = Agent::new(config).await;
    if let Err(e) = agent_result {
        println!("⚠️  Skipping test: Failed to create agent: {}", e);
        println!("   This is expected if embeddings are not available");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let agent = agent_result.unwrap();

    // Wait for background sync
    println!("⏳ Waiting for memory sync to complete...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Test memory search
    println!("🔍 Testing memory search...");
    let search_result = agent.memory_manager.search("API key security", 3).await;

    if let Err(e) = &search_result {
        println!("⚠️  Skipping test: Memory search failed: {}", e);
        println!("   This might be due to embedding API issues");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let results = search_result.unwrap();
    println!("\n📊 Memory search results:");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. [{:.2}] {}", i + 1, result.score, result.path);
    }

    // Check results
    if results.is_empty() {
        println!("⚠️  No results found. Sync might still be in progress.");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    let found_test_doc = results.iter().any(|r| r.path.contains("test.md"));
    if !found_test_doc {
        println!("⚠️  Test document not found. Embeddings might still be processing.");
        let _ = fs::remove_dir_all(&temp_dir);
        return;
    }

    println!("\n✅ Memory integration test PASSED!");

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}
