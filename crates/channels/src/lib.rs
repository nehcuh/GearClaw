// Channel Adapters for Message Platforms
//
// This crate implements adapters for Discord, Telegram, and WhatsApp.

pub mod adapter;
pub mod platforms;

pub use adapter::{
    ChannelAdapter,
    ChannelManager,
    ChannelError,
    MessageTarget,
    MessageContent,
    IncomingMessage,
    MessageSource,
};
pub use platforms::discord::DiscordAdapter;
