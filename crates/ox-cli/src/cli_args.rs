use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// High-performance, secure, minimalist agent harness for developers.
#[derive(Parser, Debug)]
#[command(name = "ox", version, about, long_about = None)]
pub struct Cli {
    /// LLM model name (e.g. claude-3-7-sonnet, gpt-4o, deepseek-chat, gemini-2.0-flash).
    #[arg(short, long, global = true)]
    pub model: Option<String>,

    /// Provider type: anthropic, openai, gemini, ollama, custom.
    #[arg(short, long, global = true)]
    pub provider: Option<String>,

    /// Path to the workspace root directory (defaults to current working directory).
    #[arg(short, long, global = true)]
    pub workspace: Option<PathBuf>,

    /// Custom API Base URL (e.g. http://localhost:11434/v1).
    #[arg(long, global = true)]
    pub base_url: Option<String>,

    /// Auto-approve all mutating tool actions without human-in-the-loop prompt.
    #[arg(short = 'y', long, global = true)]
    pub auto_approve: bool,

    /// Verbose diagnostic output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start an interactive chat session with branched history and tool execution.
    Chat {
        /// Optional session ID to resume.
        #[arg(short, long)]
        session: Option<String>,

        /// Initial prompt to send immediately upon launch.
        #[arg(long)]
        prompt: Option<String>,
    },

    /// Run a single prompt in non-interactive batch mode and exit.
    Run {
        /// Prompt instruction to execute.
        prompt: String,

        /// Maximum reasoning turns allowed.
        #[arg(long)]
        max_turns: Option<usize>,
    },

    /// Manage conversational sessions and branched DAG checkpoints.
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Inspect all registered built-in and MCP tools.
    Tools,

    /// Run the interactive setup wizard to configure a provider, model, and API key.
    /// Useful for first-time setup or adding credentials for a new provider.
    Setup,
}

#[derive(Subcommand, Debug)]
pub enum SessionCommands {
    /// List all saved sessions in the workspace.
    List,

    /// Inspect the tree structure and branches of a session.
    Tree {
        /// Session ID to inspect.
        session_id: String,
    },

    /// Export a session history to markdown or JSON.
    Export {
        /// Session ID to export.
        session_id: String,

        /// Output file path.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
