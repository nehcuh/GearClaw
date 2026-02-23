//! Built-in MCP server registry.
//!
//! Provides a catalog of well-known MCP servers that the Agent can
//! search, install, and enable autonomously.

/// How to install an MCP server package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallMethod {
    /// `npm install -g <package>` or `npx -y <package>` (runs without install)
    Npx,
    /// `pip install <package>`
    Pip,
    /// `uvx <package>` (uv tool run)
    Uvx,
    /// `cargo install <package>`
    Cargo,
    /// Custom install instruction (shown to user)
    Manual(String),
}

impl InstallMethod {
    /// Returns the shell command string to install the package.
    pub fn install_command(&self, package: &str) -> Option<String> {
        match self {
            InstallMethod::Npx => Some(format!("npm install -g {}", package)),
            InstallMethod::Pip => Some(format!("pip install {}", package)),
            InstallMethod::Uvx => Some(format!("uv tool install {}", package)),
            InstallMethod::Cargo => Some(format!("cargo install {}", package)),
            InstallMethod::Manual(_) => None,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            InstallMethod::Npx => "npm/npx",
            InstallMethod::Pip => "pip",
            InstallMethod::Uvx => "uvx",
            InstallMethod::Cargo => "cargo",
            InstallMethod::Manual(_) => "manual",
        }
    }
}

/// An entry in the built-in MCP server registry.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// Unique identifier (used as config key)
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// What this server provides
    pub description: &'static str,
    /// How to install it
    pub install_method: InstallMethod,
    /// Package name for installation
    pub package: &'static str,
    /// Command to run the server (after install)
    pub command: &'static str,
    /// Default arguments (placeholders like `<path>` need substitution)
    pub default_args: &'static [&'static str],
    /// Required environment variable names (must be set by user)
    pub required_env: &'static [&'static str],
    /// Searchable tags
    pub tags: &'static [&'static str],
    /// Optional configuration notes shown to user
    pub notes: Option<&'static str>,
}

impl RegistryEntry {
    /// Returns true if the query matches this entry's id, name, description, or tags.
    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.id.to_lowercase().contains(&q)
            || self.name.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}

/// Returns the full built-in MCP server registry.
pub fn builtin_registry() -> &'static [RegistryEntry] {
    &REGISTRY
}

/// Search the registry by a query string.
pub fn search(query: &str) -> Vec<&'static RegistryEntry> {
    if query.trim().is_empty() {
        REGISTRY.iter().collect()
    } else {
        REGISTRY.iter().filter(|e| e.matches(query)).collect()
    }
}

/// Find an entry by exact id.
pub fn find_by_id(id: &str) -> Option<&'static RegistryEntry> {
    REGISTRY.iter().find(|e| e.id == id)
}

static REGISTRY: &[RegistryEntry] = &[
    RegistryEntry {
        id: "filesystem",
        name: "Filesystem",
        description: "Read and write local files. Allows the Agent to browse directories, read file contents, and write files within allowed paths.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-filesystem",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
        required_env: &[],
        tags: &["files", "filesystem", "read", "write", "local"],
        notes: Some("Replace /tmp with the directories you want to expose."),
    },
    RegistryEntry {
        id: "github",
        name: "GitHub",
        description: "Search repositories, read files, list issues and pull requests via the GitHub API.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-github",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-github"],
        required_env: &["GITHUB_PERSONAL_ACCESS_TOKEN"],
        tags: &["github", "git", "repos", "issues", "pr", "code"],
        notes: Some("Requires GITHUB_PERSONAL_ACCESS_TOKEN with repo access."),
    },
    RegistryEntry {
        id: "fetch",
        name: "Fetch",
        description: "Fetch and convert web pages to Markdown. Useful for browsing documentation, articles, and websites.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-fetch",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-fetch"],
        required_env: &[],
        tags: &["web", "fetch", "http", "browse", "markdown", "url"],
        notes: None,
    },
    RegistryEntry {
        id: "memory",
        name: "Memory (Knowledge Graph)",
        description: "Persistent key-value and knowledge graph memory. Stores and retrieves structured facts across sessions.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-memory",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-memory"],
        required_env: &[],
        tags: &["memory", "knowledge", "graph", "persist", "facts"],
        notes: None,
    },
    RegistryEntry {
        id: "postgres",
        name: "PostgreSQL",
        description: "Connect to a PostgreSQL database. Execute read-only queries and inspect schema.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-postgres",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-postgres", "postgresql://localhost/mydb"],
        required_env: &[],
        tags: &["database", "postgres", "sql", "db", "query"],
        notes: Some("Replace the connection URL with your PostgreSQL connection string."),
    },
    RegistryEntry {
        id: "sqlite",
        name: "SQLite",
        description: "Query and inspect local SQLite databases. Read schema and execute queries.",
        install_method: InstallMethod::Uvx,
        package: "mcp-server-sqlite",
        command: "uvx",
        default_args: &["mcp-server-sqlite", "--db-path", "/tmp/db.sqlite"],
        required_env: &[],
        tags: &["database", "sqlite", "sql", "db", "local"],
        notes: Some("Replace /tmp/db.sqlite with the path to your SQLite file."),
    },
    RegistryEntry {
        id: "brave-search",
        name: "Brave Search",
        description: "Web and local search using the Brave Search API. Returns high-quality web results.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-brave-search",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-brave-search"],
        required_env: &["BRAVE_API_KEY"],
        tags: &["search", "web", "brave", "internet", "query"],
        notes: Some("Requires a free Brave Search API key from https://brave.com/search/api/"),
    },
    RegistryEntry {
        id: "puppeteer",
        name: "Puppeteer (Browser Automation)",
        description: "Control a headless Chrome browser. Screenshot pages, click elements, fill forms, and scrape dynamic content.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-puppeteer",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-puppeteer"],
        required_env: &[],
        tags: &["browser", "puppeteer", "chrome", "headless", "scrape", "screenshot", "automation"],
        notes: Some("Requires Node.js and will download Chromium on first run."),
    },
    RegistryEntry {
        id: "slack",
        name: "Slack",
        description: "Read and post messages in Slack workspaces. List channels, users, and message history.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-slack",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-slack"],
        required_env: &["SLACK_BOT_TOKEN", "SLACK_TEAM_ID"],
        tags: &["slack", "chat", "messaging", "workspace", "team"],
        notes: Some("Requires a Slack Bot Token and Team ID from your Slack app configuration."),
    },
    RegistryEntry {
        id: "aws-kb-retrieval",
        name: "AWS Knowledge Base Retrieval",
        description: "Query Amazon Bedrock Knowledge Bases for semantic document retrieval using AWS RAG.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-aws-kb-retrieval",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-aws-kb-retrieval"],
        required_env: &["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_REGION"],
        tags: &["aws", "bedrock", "rag", "knowledge", "cloud"],
        notes: Some("Requires AWS credentials and a configured Bedrock Knowledge Base."),
    },
    RegistryEntry {
        id: "google-maps",
        name: "Google Maps",
        description: "Geocoding, directions, places search, and distance matrix via Google Maps API.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-google-maps",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-google-maps"],
        required_env: &["GOOGLE_MAPS_API_KEY"],
        tags: &["maps", "location", "geocode", "directions", "places", "google"],
        notes: Some("Requires a Google Maps Platform API key."),
    },
    RegistryEntry {
        id: "sequential-thinking",
        name: "Sequential Thinking",
        description: "Structured multi-step reasoning tool. Helps break down complex problems into sequential thought steps.",
        install_method: InstallMethod::Npx,
        package: "@modelcontextprotocol/server-sequential-thinking",
        command: "npx",
        default_args: &["-y", "@modelcontextprotocol/server-sequential-thinking"],
        required_env: &[],
        tags: &["thinking", "reasoning", "planning", "steps", "analysis"],
        notes: None,
    },
    RegistryEntry {
        id: "context7",
        name: "Context7",
        description: "Up-to-date library documentation and code examples. Resolves library names to live docs, preventing hallucinated APIs.",
        install_method: InstallMethod::Npx,
        package: "@upstash/context7-mcp",
        command: "npx",
        default_args: &["-y", "@upstash/context7-mcp"],
        required_env: &[],
        tags: &["docs", "documentation", "library", "context", "upstash", "context7"],
        notes: Some("Provides resolve-library-id and get-library-docs tools. No API key required."),
    },
];
