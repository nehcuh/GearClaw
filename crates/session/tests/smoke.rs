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

#[test]
fn session_manager_rejects_path_traversal_id() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manager = SessionManager::new(temp.path().to_path_buf()).expect("manager");
    assert!(manager.get_or_create_session("../escape").is_err());
    assert!(manager.get_or_create_session("..").is_err());
    assert!(manager.get_or_create_session("bad/name").is_err());
}

#[test]
fn session_manager_accepts_safe_channel_style_id() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manager = SessionManager::new(temp.path().to_path_buf()).expect("manager");
    let id = "discord:user:12345_abc-1.0";
    let loaded = manager.get_or_create_session(id).expect("load");
    assert_eq!(loaded.id, id);
}
