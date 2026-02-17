use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "gearclaw", author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Config file path
    #[arg(short, long, value_name = "FILE")]
    pub config_path: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start interactive chat
    Chat,

    /// Generate sample configuration
    ConfigSample {
        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List all sessions
    ListSessions,

    /// Delete a session
    DeleteSession {
        /// Session ID
        session_id: String,
    },

    /// Run a single command
    Run {
        /// The prompt/command to run
        prompt: String,

        /// Session ID to use (optional)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Initialize configuration
    Init,

    /// Memory management
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Test MCP integration
    TestMcp,

    /// Start Gateway server
    Gateway {
        /// Gateway host
        #[arg(long, default_value = "127.0.0.1")]
        host: Option<String>,

        /// Gateway port
        #[arg(short, long, default_value = "18789")]
        port: Option<u16>,

        /// Development mode (verbose logging)
        #[arg(short, long)]
        dev: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// Sync memory index
    Sync,
    /// Search memory
    Search { query: String },
}
