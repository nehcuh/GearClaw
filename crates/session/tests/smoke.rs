use gearclaw_session::{Session, SessionManager};

#[test]
fn session_smoke_construct_and_message_flow() {
    let mut session = Session::new("s1".to_string());
    assert_eq!(session.id, "s1");
    assert!(session.get_messages().is_empty());
    session.clear_history();
    assert!(session.get_messages().is_empty());
}

#[test]
fn session_manager_smoke_new() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manager = SessionManager::new(temp.path().to_path_buf());
    assert!(manager.is_ok());
}
