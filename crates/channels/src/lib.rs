// Channel Adapters for Message Platforms
//
// This crate implements adapters for Discord, Telegram, and WhatsApp.

pub mod adapter;
pub mod platforms;

pub use adapter::{
    ChannelAdapter, ChannelError, ChannelManager, IncomingMessage, MessageContent, MessageSource,
    MessageTarget,
};
pub use platforms::discord::DiscordAdapter;
