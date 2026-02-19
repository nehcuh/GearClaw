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

    /// Search skills from configured sources
    SearchSkill {
        /// Query text (matches name/description)
        query: String,
        /// Optional source name filter
        #[arg(long)]
        source: Option<String>,
        /// Force refresh git sources before searching
        #[arg(long)]
        update: bool,
    },

    /// Install a skill by name from configured sources
    InstallSkill {
        /// Skill name to install
        name: String,
        /// Optional source name filter
        #[arg(long)]
        source: Option<String>,
        /// Overwrite existing installed skill
        #[arg(long)]
        force: bool,
        /// Show installation plan without writing files
        #[arg(long)]
        dry_run: bool,
        /// Force refresh git sources before installation
        #[arg(long)]
        update: bool,
    },

    /// List configured skill sources
    ListSources,
    /// Show recent skill installation audit records
    ListAudit {
        /// Number of recent audit records to show
        #[arg(long, default_value = "20")]
        limit: usize,
        /// Filter by source name
        #[arg(long)]
        source: Option<String>,
        /// Filter by skill name
        #[arg(long)]
        skill: Option<String>,
        /// Filter by status (e.g. installed)
        #[arg(long)]
        status: Option<String>,
        /// Filter by minimum timestamp (epoch seconds, inclusive)
        #[arg(long)]
        since: Option<u64>,
        /// Filter by maximum timestamp (epoch seconds, inclusive)
        #[arg(long)]
        until: Option<u64>,
        /// Output records as JSON
        #[arg(long)]
        json: bool,
        /// Output format: text | json | jsonl
        #[arg(long, default_value = "text", value_parser = ["text", "json", "jsonl"])]
        output: String,
    },

    /// Show current skill installation trust policy
    TrustPolicy,

    /// Test MCP integration (shows capability status when MCP is disabled)
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

        /// Allow unauthenticated requests (DANGEROUS, dev-only)
        #[arg(long)]
        allow_unauthenticated: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// Sync memory index
    Sync,
    /// Search memory
    Search { query: String },
}
