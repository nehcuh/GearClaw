// Agent Trigger Logic
//
// This module implements trigger checking for Agent auto-response.
// It determines whether a message should trigger Agent processing based on:
// - Trigger mode (always, mention, keyword)
// - Mention patterns (@agent, @bot, etc.)
// - Keywords
// - Channel whitelist/blacklist

use gearclaw_core::{AgentTriggerConfig, TriggerMode};
use crate::protocol::ChannelSource;

/// Check if a message should trigger Agent response
pub fn should_trigger_agent(
    platform: &str,
    source: &ChannelSource,
    content: &str,
    trigger_config: &AgentTriggerConfig,
) -> bool {
    // Check channel blacklist first
    let channel_key = format!("{}:{}", platform, get_channel_id(source));
    if trigger_config.disabled_channels.contains(&channel_key) {
        tracing::debug!("Message blocked by disabled_channels list: {}", channel_key);
        return false;
    }

    // Check channel whitelist (if configured)
    if !trigger_config.enabled_channels.is_empty() {
        if !trigger_config.enabled_channels.contains(&channel_key) {
            tracing::debug!("Message not in enabled_channels list: {}", channel_key);
            return false;
        }
    }

    // Check trigger mode
    match trigger_config.mode {
        TriggerMode::Always => {
            // Always respond (unless blocked by blacklist)
            true
        }
        TriggerMode::Mention => {
            // Check if any mention pattern is in the message
            trigger_config.mention_patterns.iter().any(|pattern| {
                content.contains(pattern) || content.starts_with(&pattern.replace('@', ""))
            })
        }
        TriggerMode::Keyword => {
            // Check if any keyword is in the message
            trigger_config.keywords.iter().any(|keyword| {
                content.to_lowercase().contains(&keyword.to_lowercase())
            })
        }
    }
}

/// Extract channel ID from source
fn get_channel_id(source: &ChannelSource) -> String {
    match source {
        ChannelSource::User { id, .. } => id.clone(),
        ChannelSource::Channel { id, .. } => id.clone(),
        ChannelSource::Group { id, .. } => id.clone(),
    }
}

/// Extract mention from message (e.g., "@agent help" -> "help")
pub fn extract_mention_prefix(content: &str, trigger_config: &AgentTriggerConfig) -> Option<String> {
    if trigger_config.mode != TriggerMode::Mention {
        return None;
    }

    for pattern in &trigger_config.mention_patterns {
        // Check if message starts with pattern
        if content.starts_with(pattern) {
            let remainder = content[pattern.len()..].trim();
            return Some(remainder.to_string());
        }

        // Check if message contains pattern anywhere
        if let Some(pos) = content.find(pattern) {
            let remainder = content[pos + pattern.len()..].trim();
            return Some(remainder.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_trigger_config() -> AgentTriggerConfig {
        AgentTriggerConfig::default()
    }

    fn user_source() -> ChannelSource {
        ChannelSource::User {
            id: "123456".to_string(),
            name: "TestUser".to_string(),
        }
    }

    #[test]
    fn test_always_mode_responds() {
        let mut config = default_trigger_config();
        config.mode = TriggerMode::Always;

        assert!(should_trigger_agent("discord", &user_source(), "hello", &config));
    }

    #[test]
    fn test_mention_mode_with_pattern() {
        let config = default_trigger_config(); // Default: Mention mode with @agent, @bot

        assert!(should_trigger_agent("discord", &user_source(), "@agent hello", &config));
        assert!(should_trigger_agent("discord", &user_source(), "@bot help", &config));
        assert!(!should_trigger_agent("discord", &user_source(), "hello", &config));
    }

    #[test]
    fn test_keyword_mode() {
        let mut config = default_trigger_config();
        config.mode = TriggerMode::Keyword;
        config.keywords = vec!["help".to_string(), "error".to_string()];

        assert!(should_trigger_agent("discord", &user_source(), "I need help", &config));
        assert!(should_trigger_agent("discord", &user_source(), "There's an error", &config));
        assert!(!should_trigger_agent("discord", &user_source(), "Hello world", &config));
    }

    #[test]
    fn test_blacklist() {
        let mut config = default_trigger_config();
        config.mode = TriggerMode::Always;
        config.disabled_channels = vec!["discord:123456".to_string()];

        assert!(!should_trigger_agent("discord", &user_source(), "hello", &config));
    }

    #[test]
    fn test_whitelist() {
        let mut config = default_trigger_config();
        config.mode = TriggerMode::Always;
        config.enabled_channels = vec!["discord:789012".to_string()];

        assert!(!should_trigger_agent("discord", &user_source(), "hello", &config));
        assert!(should_trigger_agent(
            "discord",
            &ChannelSource::User {
                id: "789012".to_string(),
                name: "TestUser".to_string(),
            },
            "hello",
            &config
        ));
    }

    #[test]
    fn test_extract_mention() {
        let config = default_trigger_config();

        assert_eq!(
            extract_mention_prefix("@agent hello world", &config),
            Some("hello world".to_string())
        );
        assert_eq!(
            extract_mention_prefix("hello @agent world", &config),
            Some("world".to_string())
        );
        assert_eq!(extract_mention_prefix("hello world", &config), None);
    }
}
