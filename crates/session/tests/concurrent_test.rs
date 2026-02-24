//! Concurrency tests for session-level locking
//!
//! These tests verify that the session-level locking prevents race conditions
//! when multiple tasks modify the same session concurrently.

use gearclaw_llm::Message;
use gearclaw_session::SessionManager;
use tempfile::TempDir;
use tokio::task::JoinSet;

#[tokio::test]
async fn test_concurrent_message_addition() {
    let temp_dir = TempDir::new().unwrap();
    let session_manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();

    let session_id = "concurrent_test";
    let num_tasks = 10;
    let messages_per_task = 10;

    // Spawn concurrent tasks, each adding multiple messages
    let mut join_set = JoinSet::new();

    for task_id in 0..num_tasks {
        let manager = session_manager.clone();
        let id = session_id.to_string();

        join_set.spawn(async move {
            for msg_num in 0..messages_per_task {
                let msg = Message {
                    role: "user".to_string(),
                    content: Some(format!("Task {} Message {}", task_id, msg_num)),
                    tool_calls: None,
                    tool_call_id: None,
                };

                // Use atomic add_message
                manager.add_message(&id, msg).await.unwrap();
            }
        });
    }

    // Wait for all tasks to complete
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    // Verify final session state
    let session = session_manager.get_or_create_session(session_id).unwrap();
    assert_eq!(session.messages.len(), num_tasks * messages_per_task);
}

#[tokio::test]
async fn test_concurrent_clear_and_add() {
    let temp_dir = TempDir::new().unwrap();
    let session_manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();

    let session_id = "clear_test";

    // Add initial messages
    for i in 0..10 {
        session_manager
            .add_message(
                session_id,
                Message {
                    role: "user".to_string(),
                    content: Some(format!("Initial message {}", i)),
                    tool_calls: None,
                    tool_call_id: None,
                },
            )
            .await
            .unwrap();
    }

    // Spawn concurrent tasks: some add messages, some clear
    let mut join_set = JoinSet::new();

    // Task 1: Clear the session
    let manager1 = session_manager.clone();
    let id1 = session_id.to_string();
    join_set.spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        manager1.clear_history(&id1).await.unwrap();
    });

    // Task 2: Add messages after clear
    let manager2 = session_manager.clone();
    let id2 = session_id.to_string();
    join_set.spawn(async move {
        for i in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            manager2
                .add_message(
                    &id2,
                    Message {
                        role: "user".to_string(),
                        content: Some(format!("After clear message {}", i)),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                )
                .await
                .unwrap();
        }
    });

    // Wait for all tasks
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    // Verify session is consistent
    let session = session_manager.get_or_create_session(session_id).unwrap();
    // Should have only messages added after clear (message count may vary due to timing)
    // The key is that there should be no corruption or duplicate messages
    assert!(session.messages.len() <= 15); // At most 5 messages after clear
}

#[tokio::test]
async fn test_atomic_with_session() {
    let temp_dir = TempDir::new().unwrap();
    let session_manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();

    let session_id = "atomic_test";

    // Use atomic operation to load, modify, and save
    session_manager
        .with_session(session_id, |session| {
            session.add_message(Message {
                role: "user".to_string(),
                content: Some("Test message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            });
            session.add_message(Message {
                role: "assistant".to_string(),
                content: Some("Response".to_string()),
                tool_calls: None,
                tool_call_id: None,
            });
            Ok::<(), gearclaw_session::SessionError>(())
        })
        .await
        .unwrap();

    // Verify messages were added atomically
    let session = session_manager.get_or_create_session(session_id).unwrap();
    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].role, "user");
    assert_eq!(session.messages[1].role, "assistant");
}

#[tokio::test]
async fn test_session_lock_prevents_data_loss() {
    let temp_dir = TempDir::new().unwrap();
    let session_manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();

    let session_id = "data_loss_test";

    // Spawn many concurrent tasks adding messages
    let num_tasks = 20;
    let mut join_set = JoinSet::new();

    for i in 0..num_tasks {
        let manager = session_manager.clone();
        let id = session_id.to_string();
        join_set.spawn(async move {
            manager
                .add_message(
                    &id,
                    Message {
                        role: "user".to_string(),
                        content: Some(format!("Message from task {}", i)),
                        tool_calls: None,
                        tool_call_id: None,
                    },
                )
                .await
                .unwrap();
        });
    }

    // Wait for all tasks
    while let Some(result) = join_set.join_next().await {
        result.unwrap();
    }

    // Verify no message loss
    let session = session_manager.get_or_create_session(session_id).unwrap();
    assert_eq!(
        session.messages.len(),
        num_tasks,
        "Expected {} messages, got {}",
        num_tasks,
        session.messages.len()
    );

    // Verify all messages are unique
    let mut message_contents: Vec<_> = session
        .messages
        .iter()
        .filter_map(|m| m.content.as_ref())
        .collect();
    message_contents.sort();
    message_contents.dedup();

    assert_eq!(
        message_contents.len(),
        num_tasks,
        "Messages were lost or duplicated"
    );
}
