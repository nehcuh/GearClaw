use gearclaw_tools::ToolExecutor;

#[test]
fn tools_smoke_available_tools_not_empty() {
    let executor = ToolExecutor::new("full");
    let tools = executor.available_tools();
    assert!(!tools.is_empty());
    assert!(tools.iter().any(|t| t.name == "exec"));
}
