use gearclaw_llm::{ChatCompletionRequest, LLMClient, Message};

#[test]
fn llm_smoke_client_construction() {
    let _client = LLMClient::new(
        "test-key".to_string(),
        "https://example.com/v1".to_string(),
        "gpt-test".to_string(),
        "embed-test".to_string(),
    );
}

#[test]
fn llm_smoke_request_serde_roundtrip() {
    let req = ChatCompletionRequest {
        model: "gpt-test".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: Some("hello".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }],
        max_tokens: Some(32),
        temperature: Some(0.7),
        tools: None,
        tool_choice: None,
        stream: Some(true),
    };

    let json = serde_json::to_string(&req).expect("serialize request");
    let parsed: ChatCompletionRequest = serde_json::from_str(&json).expect("deserialize request");
    assert_eq!(parsed.model, "gpt-test");
    assert_eq!(parsed.messages.len(), 1);
    assert_eq!(parsed.messages[0].role, "user");
}
