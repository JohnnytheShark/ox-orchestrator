/**
 * ox-orchestrator — Diataxis Documentation Web Portal & Interactive Demo
 */

// Documentation Dataset following Diátaxis framework
const DOCS_DATA = {
  // 1. TUTORIALS
  "tutorials/01_getting_started.md": {
    title: "Getting Started with ox",
    category: "Tutorials",
    badge: "🎓 LEARNING-ORIENTED",
    content: `# Tutorial: Getting Started with ox

In this tutorial, you will learn how to build, configure, and use \`ox\` to assist in exploring and modifying a software repository.

---

## Prerequisites
- Rust 1.80+ (\`rustup default stable\`)
- An API key for your preferred LLM provider (Anthropic Claude, OpenAI, Google Gemini, DeepSeek, or a local Ollama instance).

---

## Step 1: Building and Installing ox

Clone the repository and build the release binary:

\`\`\`bash
cargo build --release -p ox-cli
\`\`\`

Optionally install \`ox\` to your cargo bin directory:

\`\`\`bash
cargo install --path crates/ox-cli
\`\`\`

Verify that \`ox\` is ready:

\`\`\`bash
ox --help
\`\`\`

---

## Step 2: Setting up Credentials

\`ox\` automatically reads credentials from standard environment variables:

\`\`\`bash
# For Anthropic Claude (Default)
export ANTHROPIC_API_KEY="sk-ant-..."

# For OpenAI
export OPENAI_API_KEY="sk-..."

# For Google Gemini
export GEMINI_API_KEY="..."

# For DeepSeek
export DEEPSEEK_API_KEY="sk-..."
\`\`\`

---

## Step 3: Launching an Interactive Session

Navigate to any codebase directory and start \`ox\`:

\`\`\`bash
cd /path/to/your/project
ox chat
\`\`\`

You will be greeted by the \`ox\` banner showing your active provider, workspace path, and session ID:

\`\`\`text
   ____  _  __
  / __ \\| |/ /   ox-orchestrator v0.1.0
 / /_/ /|   /    Minimalist & Secure AI Agent Harness
 \\____//_/|_\\    

 Provider  : Anthropic (claude-3-7-sonnet-20250219)
 Workspace : /path/to/your/project
 Session   : session-a1b2c3d4
 Security  : Path-Jailed, Env-Scrubbed, Zeroized Secrets

user >
\`\`\`

---

## Step 4: Interacting with Code and Tools

Ask \`ox\` to inspect your repository:

\`\`\`text
user > Find all Rust files and summarize their responsibilities.
\`\`\`

\`ox\` will invoke \`find_files\`, read relevant files with \`read_file\`, and provide a concise summary.

When \`ox\` wishes to perform a mutating action (such as writing or modifying a file), you will be prompted by the Human-in-the-Loop security gate:

\`\`\`text
[SECURITY AUDIT] Tool Execution Request:
  Tool: write_file
  Arguments: {
    "path": "src/utils.rs",
    "content": "..."
  }
Authorize execution? [y]es / [n]o / [a]lways approve:
\`\`\`

Type \`y\` to approve, \`n\` to reject, or \`a\` to approve all subsequent actions in this session.

---

## Step 5: Session Checkpoints and Rewind

If \`ox\` takes an unwanted path, use \`/undo\` to step back one turn, or \`/tree\` to inspect all branches in the session DAG:

\`\`\`text
user > /tree

Session DAG [id: session-a1b2c3d4 | total nodes: 4]:
  | [d4e5f6a1] (ROOT) [USER] "Find all Rust files..."
  | [b2c3d4e5] <- parent: d4e5f6a1 [ASSISTANT] "Found 6 files..."
  * (ACTIVE LEAF) [c3d4e5f6] <- parent: b2c3d4e5 [USER] "Now refactor utils.rs"

user > /undo
Rewound to turn: b2c3d4e5
\`\`\`
`
  },

  "tutorials/02_custom_mcp_integration.md": {
    title: "Custom MCP Integration",
    category: "Tutorials",
    badge: "🎓 LEARNING-ORIENTED",
    content: `# Tutorial: Custom MCP Integration

In this tutorial, you will learn how to connect an external Model Context Protocol (MCP) server to \`ox\` to provide database queries, git operations, or browser automation tools.

---

## What is MCP in ox?

The Model Context Protocol (MCP) is an open standard allowing external processes to provide tools dynamically to AI models. In \`ox\`, MCP servers run as isolated child processes communicating via JSON-RPC 2.0 over standard input and output (stdio).

---

## Step 1: Create a Workspace Config File

Inside your project root directory, create \`.ox/config.json\`:

\`\`\`json
{
  "default_model": "claude-3-7-sonnet-20250219",
  "mcp_servers": {
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "app.db"]
    }
  }
}
\`\`\`

---

## Step 2: Inspecting Discovered Tools

Launch \`ox tools\` to verify that \`ox\` connects to the MCP server, completes the handshake, and discovers its tools:

\`\`\`bash
ox tools
\`\`\`

Output:
\`\`\`text
Registered Tools (7):

  * read_file        [READ-ONLY / SAFE]
  * write_file       [MUTATING - REQUIRES HITL APPROVAL]
  * edit_file        [MUTATING - REQUIRES HITL APPROVAL]
  * exec_command     [MUTATING - REQUIRES HITL APPROVAL]
  * grep_search      [READ-ONLY / SAFE]
  * find_files       [READ-ONLY / SAFE]
  * sqlite__query    [MUTATING - REQUIRES HITL APPROVAL]
    Description: Execute a SQL query against the SQLite database
\`\`\`

---

## Step 3: Using MCP Tools in Chat

Start an interactive chat session:

\`\`\`bash
ox chat
\`\`\`

Prompt:
\`\`\`text
user > Show me the schema of the users table and list the last 5 signups.
\`\`\`

\`ox\` will invoke \`sqlite__query\` and prompt you for confirmation before running the query against your database.
`
  },

  // 2. HOW-TO GUIDES
  "how_to/configuring_providers.md": {
    title: "Configuring LLM Providers",
    category: "How-To Guides",
    badge: "🛠️ PROBLEM-ORIENTED",
    content: `# How-To Guide: Configuring Providers

This guide shows how to configure and switch between different LLM providers in \`ox\`.

---

## Supported Providers

| Provider | Provider Flag | Default Model | Environment Variable |
|---|---|---|---|
| **Anthropic** | \`--provider anthropic\` | \`claude-3-7-sonnet-20250219\` | \`ANTHROPIC_API_KEY\` |
| **OpenAI** | \`--provider openai\` | \`gpt-4o\` | \`OPENAI_API_KEY\` |
| **Google Gemini** | \`--provider gemini\` | \`gemini-2.0-flash\` | \`GEMINI_API_KEY\` |
| **DeepSeek** | \`--provider openai\` | \`deepseek-chat\` | \`DEEPSEEK_API_KEY\` |
| **Ollama (Local)** | \`--provider ollama\` | \`llama3.3\` | None (Local) |

---

## 1. Using CLI Flags

Pass \`--provider\` and \`--model\` directly to \`ox\`:

\`\`\`bash
# Use OpenAI GPT-4o
ox chat --provider openai --model gpt-4o

# Use DeepSeek via OpenAI-compatible API
ox chat --provider openai --model deepseek-chat --base-url https://api.deepseek.com/v1

# Use Local Ollama
ox chat --provider ollama --model qwen2.5-coder:14b
\`\`\`

---

## 2. Using Workspace Configuration

To persist default model choices for a project, define them in \`.ox/config.json\`:

\`\`\`json
{
  "default_provider": "openai",
  "default_model": "gpt-4o",
  "base_url": "https://api.openai.com/v1"
}
\`\`\`
`
  },

  "how_to/session_branching_and_checkpoints.md": {
    title: "DAG Session Branching & Checkpoints",
    category: "How-To Guides",
    badge: "🛠️ PROBLEM-ORIENTED",
    content: `# How-To Guide: Session Branching and Checkpoints

This guide explains how to use \`ox\`'s non-destructive Directed Acyclic Graph (DAG) session manager to branch, rewind, and restore conversations.

---

## 1. Viewing the Session Tree

To visualize the conversation history DAG at any point during an interactive chat, type:

\`\`\`text
user > /tree
\`\`\`

Output:
\`\`\`text
Session DAG [id: session-a1b2c3d4 | total nodes: 5]:
  | [d4e5f6a1] (ROOT) [USER] "Implement JWT token validation"
  | [b2c3d4e5] <- parent: d4e5f6a1 [ASSISTANT] "Here is the implementation in auth.rs..."
  |-- [c3d4e5f6] <- parent: b2c3d4e5 [USER] "Now add Redis caching"
  |   \\-- [e5f6a1b2] <- parent: c3d4e5f6 [ASSISTANT] "Added redis crate..."
  * (ACTIVE LEAF) [f6a1b2c3] <- parent: b2c3d4e5 [USER] "Wait, use in-memory cache instead"
\`\`\`

---

## 2. Undoing the Last Turn

If an agent produces an unhelpful response, type:

\`\`\`text
user > /undo
Rewound to parent node: b2c3d4e5
\`\`\`

---

## 3. Switching Branches

To switch your active context to any node in the graph:

\`\`\`text
user > /checkout e5f6a1b2
Active conversation leaf switched to node e5f6a1b2.
\`\`\`
`
  },

  "how_to/human_in_the_loop_policies.md": {
    title: "Human-in-the-Loop Security Policies",
    category: "How-To Guides",
    badge: "🛠️ PROBLEM-ORIENTED",
    content: `# How-To Guide: Human-in-the-Loop Security & Policies

\`ox\` enforces strict boundaries between read-only discovery tools and mutating tools that change the state of your computer.

---

## Tool Classification Matrix

| Tool | Type | Default Policy | Requires Approval? |
|---|---|---|---|
| \`read_file\` | Read-only | Path-jailed to workspace | No |
| \`grep_search\` | Read-only | Jailed to workspace | No |
| \`find_files\` | Read-only | Jailed to workspace | No |
| \`write_file\` | Mutating | Atomic temp-file swap | **Yes** |
| \`edit_file\` | Mutating | Precise surgical patch | **Yes** |
| \`exec_command\` | Mutating | Env-scrubbed subprocess | **Yes** |
| MCP tools | Mutating | Stdio sandboxed | **Yes** |

---

## Interactive Authorization Options

When prompted during chat:
- \`y\` / \`Enter\`: Approve this single invocation.
- \`n\`: Deny this action. The rejection reason is returned to the model so it can propose an alternative.
- \`a\`: Auto-approve all subsequent mutating actions for the rest of the current session.

---

## Toggling Auto-Approve

To toggle auto-approval during chat, type:

\`\`\`text
user > /auto
Auto-approve mutating tools: true
\`\`\`

Or pass \`-y\` / \`--auto-approve\` when launching:

\`\`\`bash
ox chat -y
\`\`\`
`
  },

  "how_to/running_in_ci_cd.md": {
    title: "Running Batch Jobs in CI/CD",
    category: "How-To Guides",
    badge: "🛠️ PROBLEM-ORIENTED",
    content: `# How-To Guide: Running in CI/CD

This guide shows how to run \`ox\` as an automated reviewer or repair agent in GitHub Actions or any automated CI pipeline.

---

## 1. Using \`ox run\`

The \`ox run\` command executes a prompt non-interactively and exits with code 0 on success, or a non-zero code on failure.

\`\`\`bash
ox run "Review changed files against main, run tests, and fix any compiler warnings." -y --max-turns 25
\`\`\`

---

## 2. GitHub Actions Workflow Example

\`\`\`yaml
name: AI Code Review
on: [pull_request]

jobs:
  ox-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install ox
        run: |
          curl -fsSL https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      - name: Run ox automated review
        env:
          ANTHROPIC_API_KEY: \${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          ox run "Inspect all modified files in this PR and report potential bugs or memory leaks." -y
\`\`\`
`
  },

  // 3. REFERENCE
  "reference/cli_reference.md": {
    title: "CLI Flags & Slash Commands",
    category: "Reference",
    badge: "📖 INFORMATION-ORIENTED",
    content: `# Reference: Command Line Interface (CLI)

Complete reference for all \`ox\` command line arguments, flags, and interactive slash commands.

---

## Global Options

| Flag | Long Form | Description | Default |
|---|---|---|---|
| \`-m\` | \`--model <NAME>\` | Model identifier (e.g. \`claude-3-7-sonnet-20250219\`, \`gpt-4o\`) | Inferred from provider or config |
| \`-p\` | \`--provider <NAME>\` | Provider family: \`anthropic\`, \`openai\`, \`gemini\`, \`ollama\`, \`custom\` | \`anthropic\` |
| \`-w\` | \`--workspace <PATH>\` | Target workspace root directory | Current directory or \`.git\` parent |
| | \`--base-url <URL>\` | Custom API base URL | Standard provider endpoint |
| \`-y\` | \`--auto-approve\` | Automatically approve mutating tools | \`false\` |
| \`-v\` | \`--verbose\` | Enable debug logs | \`false\` |
| \`-h\` | \`--help\` | Print help information | |
| \`-V\` | \`--version\` | Print version information | |

---

## Subcommands

### 1. \`ox chat\`
Starts interactive REPL session with DAG branching history.
* \`-s, --session <ID>\`: Resume an existing session by ID.
* \`-p, --prompt <TEXT>\`: Initial instruction to execute immediately upon starting.

### 2. \`ox run\`
Non-interactive single-command batch execution.
* \`<PROMPT>\`: Prompt text to execute.
* \`--max-turns <N>\`: Maximum allowed reasoning steps (default: 30).

### 3. \`ox session\`
* \`list\`: List all stored sessions.
* \`tree <session_id>\`: Render ASCII tree of conversational DAG.
* \`export <session_id> [-o <path>]\`: Export session history to Markdown.

### 4. \`ox tools\`
Lists all registered built-in tools and connected MCP tools with descriptions and JSON parameter schemas.

---

## Interactive Slash Commands

| Command | Description |
|---|---|
| \`/cost\` | Displays cumulative session token usage, cache statistics, and estimated cost |
| \`/diff\` | Inspects current git working tree modifications |
| \`/undo\` | Rewinds the conversation leaf pointer to the previous turn |
| \`/tree\` | Visualizes the full DAG tree of branches and turns |
| \`/checkout <id>\` | Switches the active conversation pointer to node \`<id>\` |
| \`/history\` | Displays the linear sequence of messages from root to leaf |
| \`/auto\` | Toggles automatic approval of mutating tools on/off |
| \`/save\` | Forces a snapshot save of the session tree to disk |
| \`/help\` | Prints the slash command quick reference |
| \`/exit\`, \`/quit\` | Saves the session and closes the REPL |
`
  },

  "reference/configuration_schema.md": {
    title: "Configuration Schema (.ox/config.json)",
    category: "Reference",
    badge: "📖 INFORMATION-ORIENTED",
    content: `# Reference: Configuration Schema

The \`.ox/config.json\` file stores workspace-level defaults and external Model Context Protocol (MCP) server definitions.

---

## Full Schema Example

\`\`\`json
{
  "$schema": "https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/schema.json",
  "default_provider": "anthropic",
  "default_model": "claude-3-7-sonnet-20250219",
  "base_url": "https://api.anthropic.com/v1",
  "auto_approve": false,
  "max_tokens": 8192,
  "temperature": 0.2,
  "context_compaction_threshold": 120000,
  "mcp_servers": {
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "app.db"],
      "env": {
        "SQLITE_READ_ONLY": "0"
      }
    },
    "git": {
      "command": "mcp-server-git",
      "args": ["--repository", "."]
    }
  }
}
\`\`\`
`
  },

  "reference/crate_architecture.md": {
    title: "Crate Architecture & APIs",
    category: "Reference",
    badge: "📖 INFORMATION-ORIENTED",
    content: `# Reference: Crate Architecture

\`ox-orchestrator\` is organized as a Cargo workspace with high cohesion, strict boundary layering, and zero circular dependencies.

---

## Crate Dependency Hierarchy

\`\`\`
ox-cli  (Clap CLI, Terminal REPL, HITL Gate)
  └── ox-core  (Session DAG, TokenBudgeter, ContextCompactor, AgentEngine)
        ├── ox-providers  (Anthropic, OpenAI, Gemini, Ollama SSE Adapters)
        ├── ox-tools      (Builtin tools & MCP stdio client)
        └── ox-security   (PathJail, EnvScrubber, Zeroize memory)
\`\`\`

---

## Crate Breakdown

1. **\`ox-security\`**:
   - \`PathJail\`: Zero-overhead canonicalization protecting against directory traversal, path escaping, and symlink exploits.
   - \`EnvScrubber\`: Subprocess environment scrubbing protecting against credential leakage.
   - Zeroize memory traits for secrets.

2. **\`ox-providers\`**:
   - Unified \`LlmProvider\` async trait with streaming SSE events (\`StreamChunk\`).
   - Adapters for Anthropic Messages API, OpenAI Chat Completions, Google Gemini REST, and Ollama.

3. **\`ox-tools\`**:
   - Builtin tools: \`read_file\`, \`write_file\`, \`edit_file\`, \`exec_command\`, \`grep_search\`, \`find_files\`.
   - \`McpClient\`: Asynchronous JSON-RPC 2.0 stdio client with schema negotiation.

4. **\`ox-core\`**:
   - \`SessionDAG\`: Thread-safe directed acyclic graph representing branching conversation trees.
   - \`ContextCompactor\`: Sliding window token budget manager with smart turn pruning.
   - \`AgentEngine\`: Main orchestration loop.

5. **\`ox-cli\`**:
   - Entry point, Clap command line parser, interactive ANSI terminal renderer, and Human-in-the-Loop prompts.
`
  },

  "reference/security_model.md": {
    title: "Security Model & Guarantees",
    category: "Reference",
    badge: "📖 INFORMATION-ORIENTED",
    content: `# Reference: Security Model

\`ox\` was built specifically to defend against malicious prompts, prompt injections, unintended directory traversal, and API credential leakage.

---

## Three Pillars of ox Security

### 1. Kernel-Level Path Sandboxing (\`PathJail\`)
- All file reads, writes, edits, and searches are evaluated against a root canonical directory.
- Relative paths, parent climbing (\`../../\`), and symlinks resolving outside the root jail are strictly rejected before any OS syscall occurs.

### 2. Subprocess Environment Scrubbing (\`EnvScrubber\`)
- Subprocesses invoked via \`exec_command\` or MCP child processes receive a scrubbed environment.
- Sensitive environment variables matching patterns such as \`*_API_KEY\`, \`*TOKEN*\`, \`*SECRET*\`, \`AWS_*\`, \`SSH_*\` are stripped.

### 3. In-Memory Credential Zeroization
- All API keys stored in memory are wrapped in structures implementing the \`zeroize::Zeroize\` trait.
- Upon drop, memory buffers are explicitly wiped with zeroes using memory fences to prevent credential leakage in core dumps or memory scrapers.
`
  },

  // 4. EXPLANATION
  "explanation/architecture_overview.md": {
    title: "Architecture Overview & Why Rust",
    category: "Explanation",
    badge: "💡 UNDERSTANDING-ORIENTED",
    content: `# Explanation: Architecture Overview & Why Rust

This document explains the technical motivations behind \`ox\`'s architecture and why Rust was selected as the implementation language.

---

## Why Rust?

Traditional AI agent harnesses are predominantly written in TypeScript/Node.js or Python. While fast to prototype, these ecosystems present major hurdles for autonomous agents:

1. **Massive Runtime Footprint**: Node.js and Python runtimes require hundreds of megabytes of dependencies (\`node_modules\`, virtualenvs), creating distribution friction.
2. **Cold-Start Latency**: Scripted CLI tools take 200–800ms just to initialize their interpreters. \`ox\` boots in under **15 milliseconds**.
3. **Memory Safety & Concurrency**: Rust's ownership model guarantees data-race freedom across asynchronous streaming SSE connections and tool execution tasks.
4. **Binary Portability**: Compiles to a single, standalone native machine binary (~12MB) that runs everywhere with zero prerequisites.

---

## Design Principles

- **Minimalist Core**: Do one thing exceptionally well without bloated abstractions.
- **Safety by Default**: Read operations are allowed; mutating actions require explicit confirmation.
- **Provider Agnostic**: Direct native HTTP streaming without heavyweight third-party SDK dependencies.
- **Non-Destructive State**: Conversational history is an immutable DAG, never a lossy array.
`
  },

  "explanation/session_dag_vs_linear_history.md": {
    title: "Session DAG vs Linear History",
    category: "Explanation",
    badge: "💡 UNDERSTANDING-ORIENTED",
    content: `# Explanation: Session DAG vs Linear History

Most LLM chat harnesses maintain conversational history as a flat array (\`Vec<Message>\`). When a user edits a message or rewinds, prior turns are discarded forever.

\`ox\` replaces linear history with a **Directed Acyclic Graph (DAG)** of turns.

---

## The Problem with Linear History

In a linear history:
\`\`\`
Turn 1 -> Turn 2 -> Turn 3 (Bad path)
\`\`\`
If you rewind to Turn 2 and ask a different question, Turn 3 is obliterated. If the new direction proves even worse, you cannot recover Turn 3.

---

## The DAG Solution in ox

In \`ox\`, every message is an immutable node referencing its parent node:

\`\`\`
       [Turn 1]
          |
       [Turn 2]
       /      \\
   [Turn 3A]   [Turn 3B] (Active)
      |
   [Turn 4A]
\`\`\`

- You can switch active leaves using \`/checkout <id>\`.
- You can step back with \`/undo\` without deleting alternative reasoning paths.
- Sessions can be rendered as ASCII trees with \`ox session tree <id>\`.
`
  },

  "explanation/sandboxing_philosophy.md": {
    title: "Sandboxing & Defense-in-Depth",
    category: "Explanation",
    badge: "💡 UNDERSTANDING-ORIENTED",
    content: `# Explanation: Sandboxing & Defense-in-Depth

When an AI agent executes tools autonomously, security cannot rely on model alignment alone. A model can be jailbroken or tricked by adversarial prompt injection in codebase files.

---

## Defense-in-Depth Architecture

\`ox\` enforces safety through layers of deterministic hardware and OS boundaries:

1. **Static Analysis & Schema Validation**: Arguments are typed and validated against strict schemas before dispatch.
2. **Path Jail Canonicalization**: Filesystem operations are verified against the authorized workspace root using canonical paths.
3. **Process Isolation**: MCP servers run in separate subprocesses with scrubbed environment variables.
4. **Human Confirmation Gate**: Mutating operations trigger an interactive audit prompt, displaying the exact parameters and affected files.
`
  },

  "explanation/pi_comparison.md": {
    title: "Evolution from Pi to ox",
    category: "Explanation",
    badge: "💡 UNDERSTANDING-ORIENTED",
    content: `# Explanation: Evolution from Pi to ox

\`ox-orchestrator\` builds on lessons learned from the Pi coding agent harness, elevating its strengths and replacing runtime limitations with native Rust systems engineering.

---

## Comparison Matrix

| Feature | Pi Agent Harness | ox-orchestrator |
|---|---|---|
| **Language** | TypeScript (Node.js) | Pure Rust (2021 edition) |
| **Startup Latency** | ~450 ms | **< 15 ms** |
| **Binary Size** | Node runtime + ~85MB deps | **~12MB standalone binary** |
| **Path Sandboxing** | Advisory string checks | **Kernel-level PathJail** |
| **Session Model** | Linear Array | **Non-Destructive Session DAG** |
| **Memory Security** | GC / Plain strings | **Zeroized Credential Buffers** |
| **MCP Support** | High-level SDK | **Native JSON-RPC 2.0 Client** |
| **Documentation** | Ad-hoc Markdown | **Formal Diátaxis Architecture** |
`
  }
};

// State
let currentDocKey = "tutorials/01_getting_started.md";

// Terminal Demo Script
const TERMINAL_STEPS = [
  { type: "prompt", text: "ox chat --model claude-3-7-sonnet-20250219" },
  { type: "banner", text: "   ____  _  __\n  / __ \\| |/ /   ox-orchestrator v0.1.0\n / /_/ /|   /    Minimalist & Secure AI Agent Harness\n \\____//_/|_\\    \n\n Provider  : Anthropic (claude-3-7-sonnet-20250219)\n Workspace : /home/user/project\n Session   : session-a1b2c3d4\n Security  : Path-Jailed, Env-Scrubbed, Zeroized" },
  { type: "user", text: "Find all Rust source files and audit them for unsafe blocks." },
  { type: "tool", text: "[TOOL] find_files({\"pattern\": \"*.rs\"}) -> 12 files discovered." },
  { type: "tool", text: "[TOOL] grep_search({\"query\": \"unsafe {\"}) -> 0 unsafe blocks found." },
  { type: "ai", text: "All 12 Rust files were audited. The codebase is 100% safe Rust with zero `unsafe` blocks." },
  { type: "user", text: "Create a new module src/cache.rs with an in-memory LRU cache." },
  { type: "security", text: "[SECURITY AUDIT] Tool Execution Request:\n  Tool: write_file\n  Target: /home/user/project/src/cache.rs\nAuthorize execution? [y]es / [n]o / [a]lways approve: y" },
  { type: "success", text: "✓ File src/cache.rs created atomically via temporary swap." }
];

// Initialize
document.addEventListener("DOMContentLoaded", () => {
  renderDocNavList();
  renderActiveDoc(currentDocKey);
  autoDetectPlatform();
  setupSearchKeyboardShortcuts();
  startTerminalDemo();
  fetchLatestRelease();
});

// Dynamic Release Asset Updater via GitHub API
async function fetchLatestRelease() {
  try {
    const res = await fetch("https://api.github.com/repos/JohnnytheShark/ox-orchestrator/releases/latest");
    if (!res.ok) return;
    const release = await res.json();
    if (!release || !release.assets) return;

    // Update release badge text
    const badge = document.getElementById("latestReleaseBadge");
    if (badge && release.tag_name) {
      badge.textContent = `${release.tag_name} Latest Release`;
    }

    // Map targets to download assets
    const targets = [
      "x86_64-unknown-linux-gnu",
      "x86_64-unknown-linux-musl",
      "aarch64-unknown-linux-gnu",
      "aarch64-apple-darwin",
      "x86_64-pc-windows-msvc"
    ];

    targets.forEach(t => {
      const asset = release.assets.find(a => a.name.includes(t) && !a.name.endsWith(".sha256"));
      if (asset) {
        const linkEl = document.querySelector(`[data-target="${t}"]`);
        if (linkEl) {
          linkEl.href = asset.browser_download_url;
        }
        const sizeEl = document.querySelector(`[data-size-target="${t}"]`);
        if (sizeEl && asset.size) {
          const mb = (asset.size / (1024 * 1024)).toFixed(1);
          sizeEl.textContent = `${mb}MB`;
        }
      }

      const shaAsset = release.assets.find(a => a.name.includes(t) && a.name.endsWith(".sha256"));
      if (shaAsset) {
        const shaEl = document.querySelector(`[data-checksum-target="${t}"]`);
        if (shaEl) {
          shaEl.href = shaAsset.browser_download_url;
        }
      }
    });
  } catch (err) {
    console.debug("Latest release query skipped or rate-limited:", err);
  }
}

// Render Documentation Navigation Sidebar
function renderDocNavList(filter = "") {
  const navList = document.getElementById("docNavList");
  if (!navList) return;

  const categories = ["Tutorials", "How-To Guides", "Reference", "Explanation"];
  let html = "";

  categories.forEach(cat => {
    const keys = Object.keys(DOCS_DATA).filter(k => {
      const doc = DOCS_DATA[k];
      if (doc.category !== cat) return false;
      if (!filter) return true;
      const q = filter.toLowerCase();
      return doc.title.toLowerCase().includes(q) || doc.content.toLowerCase().includes(q);
    });

    if (keys.length > 0) {
      html += `<div class="doc-group-title">${cat}</div>`;
      keys.forEach(k => {
        const doc = DOCS_DATA[k];
        const isActive = k === currentDocKey ? "active" : "";
        html += `
          <div class="doc-nav-item ${isActive}" onclick="openDiataxisDoc('${k}')">
            <span>${doc.title}</span>
          </div>
        `;
      });
    }
  });

  if (!html) {
    html = `<div style="padding: 16px; color: var(--text-muted); font-size: 0.84rem;">No articles matched "${filter}".</div>`;
  }

  navList.innerHTML = html;
}

// Filter docs list in sidebar
function filterDocsList() {
  const query = document.getElementById("docFilterInput").value;
  renderDocNavList(query);
}

// Open and render Diataxis doc
function openDiataxisDoc(key) {
  if (!DOCS_DATA[key]) return;
  currentDocKey = key;
  renderDocNavList(document.getElementById("docFilterInput")?.value || "");
  renderActiveDoc(key);

  // Smooth scroll to viewer
  const viewer = document.getElementById("diataxis-viewer");
  if (viewer) {
    viewer.scrollIntoView({ behavior: "smooth", block: "start" });
  }
}

// Render active doc content
function renderActiveDoc(key) {
  const doc = DOCS_DATA[key];
  if (!doc) return;

  const breadcrumbs = document.getElementById("docBreadcrumbs");
  if (breadcrumbs) {
    breadcrumbs.textContent = `${doc.category} / ${doc.title}`;
  }

  const githubLink = document.getElementById("viewOnGithubLink");
  if (githubLink) {
    githubLink.href = `https://github.com/JohnnytheShark/ox-orchestrator/blob/main/docs/${key}`;
  }

  const bodyEl = document.getElementById("docRenderedBody");
  if (bodyEl) {
    bodyEl.innerHTML = renderMarkdown(doc.content);
  }
}

// Minimal fast client-side markdown renderer
function renderMarkdown(md) {
  let html = md;

  // Code blocks ```lang ... ```
  html = html.replace(/```([a-z0-9_-]*)\n([\s\S]*?)```/gi, (match, lang, code) => {
    const escaped = escapeHtml(code.trim());
    return `<pre><code class="language-${lang}">${escaped}</code></pre>`;
  });

  // Inline code `code`
  html = html.replace(/`([^`]+)`/g, (match, code) => `<code>${escapeHtml(code)}</code>`);

  // Headings
  html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');

  // Bold & Italic
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>');

  // Blockquotes
  html = html.replace(/^\> (.*$)/gim, '<blockquote>$1</blockquote>');

  // Horizontal rules
  html = html.replace(/^---$/gim, '<hr>');

  // Unordered list items
  html = html.replace(/^\* (.*$)/gim, '<li>$1</li>');
  html = html.replace(/^- (.*$)/gim, '<li>$1</li>');
  html = html.replace(/(<li>.*<\/li>)/gis, '<ul>$1</ul>');

  // Markdown Tables
  html = html.replace(/\|(.+)\|/g, (match) => {
    if (match.includes('---')) return ''; // separator row
    const cells = match.split('|').filter(c => c.trim() !== '');
    const isHeader = false;
    const tds = cells.map(c => `<td>${c.trim()}</td>`).join('');
    return `<tr>${tds}</tr>`;
  });
  html = html.replace(/(<tr>.*<\/tr>)/gis, '<table>$1</table>');

  // Paragraphs
  const lines = html.split('\n\n');
  html = lines.map(block => {
    block = block.trim();
    if (!block) return '';
    if (block.startsWith('<h') || block.startsWith('<pre') || block.startsWith('<ul') || block.startsWith('<table') || block.startsWith('<blockquote') || block.startsWith('<hr')) {
      return block;
    }
    return `<p>${block.replace(/\n/g, '<br>')}</p>`;
  }).join('\n');

  return html;
}

function escapeHtml(str) {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// Copy Current Document Markdown
function copyCurrentDocMarkdown() {
  const doc = DOCS_DATA[currentDocKey];
  if (!doc) return;
  navigator.clipboard.writeText(doc.content).then(() => {
    showToast("Documentation copied to clipboard!");
  });
}

// Quick Install Switcher
const INSTALL_COMMANDS = {
  unix: "curl -fsSL https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/install.sh | bash",
  win: "irm https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/install.ps1 | iex",
  cargo: "cargo install --git https://github.com/JohnnytheShark/ox-orchestrator ox-cli"
};

function switchInstallTab(osKey) {
  document.querySelectorAll(".inst-tab").forEach(tab => {
    tab.classList.toggle("active", tab.dataset.os === osKey);
  });
  const textEl = document.getElementById("installCommandText");
  if (textEl && INSTALL_COMMANDS[osKey]) {
    textEl.textContent = INSTALL_COMMANDS[osKey];
  }
}

function copyInstallCommand() {
  const textEl = document.getElementById("installCommandText");
  if (!textEl) return;
  navigator.clipboard.writeText(textEl.textContent).then(() => {
    showToast("Command copied to clipboard!");
  });
}

// Toast notification
function showToast(msg) {
  const toast = document.getElementById("toast");
  if (!toast) return;
  toast.textContent = msg;
  toast.classList.add("show");
  setTimeout(() => toast.classList.remove("show"), 2500);
}

// Platform Auto-detection
function autoDetectPlatform() {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) {
    switchInstallTab("win");
    highlightReleaseCard("cardWin");
  } else if (ua.includes("mac")) {
    switchInstallTab("unix");
    highlightReleaseCard("cardMac");
  } else {
    switchInstallTab("unix");
    highlightReleaseCard("cardLinux");
  }
}

function highlightReleaseCard(cardId) {
  const card = document.getElementById(cardId);
  if (card) {
    card.style.borderColor = "var(--c-leaf)";
    card.style.boxShadow = "0 8px 30px rgba(143, 203, 155, 0.2)";
  }
}

// Terminal Simulated Typewriter with Interactive DAG Checkpoints
let termStepIdx = 0;
let termTimeout = null;

function startTerminalDemo() {
  const body = document.getElementById("terminalBody");
  if (!body) return;
  body.innerHTML = "";
  termStepIdx = 0;
  updateDagActivePill(2);
  playNextTerminalStep();
}

function restartTerminalDemo() {
  if (termTimeout) clearTimeout(termTimeout);
  startTerminalDemo();
}

function updateDagActivePill(stepIndex) {
  const track = document.getElementById("dagNodesTrack");
  if (!track) return;
  const pills = track.querySelectorAll(".dag-node-pill");
  pills.forEach((p, idx) => {
    if (idx === stepIndex) {
      p.classList.add("active");
    } else {
      p.classList.remove("active");
    }
  });
}

function jumpToDagCheckpoint(stepIndex) {
  if (termTimeout) clearTimeout(termTimeout);
  const body = document.getElementById("terminalBody");
  if (!body) return;
  body.innerHTML = "";
  updateDagActivePill(stepIndex);

  if (stepIndex === 0) {
    // Root checkpoint
    appendTerminalLine({ type: "prompt", text: "ox chat --model claude-3-7-sonnet-20250219" });
    appendTerminalLine({ type: "banner", text: "   ____  _  __\n  / __ \\| |/ /   ox-orchestrator v0.1.0\n / /_/ /|   /    Minimalist & Secure AI Agent Harness\n \\____//_/|_\\    \n\n Provider  : Anthropic (claude-3-7-sonnet-20250219)\n Workspace : /home/user/project\n Session   : session-a1b2c3d4 (checkpoint: root)\n Security  : Path-Jailed, Env-Scrubbed, Zeroized" });
  } else if (stepIndex === 1) {
    // Turn 1: Workspace scan
    appendTerminalLine({ type: "prompt", text: "ox chat --model claude-3-7-sonnet-20250219" });
    appendTerminalLine({ type: "user", text: "Find all Rust source files and audit them for unsafe blocks." });
    appendTerminalLine({ type: "tool", text: "[TOOL] find_files({\"pattern\": \"*.rs\"}) -> 12 files discovered." });
    appendTerminalLine({ type: "tool", text: "[TOOL] grep_search({\"query\": \"unsafe {\"}) -> 0 unsafe blocks found." });
    appendTerminalLine({ type: "ai", text: "All 12 Rust files were audited. The codebase is 100% safe Rust with zero `unsafe` blocks." });
  } else if (stepIndex === 2) {
    // Active Branch: Jailed tool execution
    TERMINAL_STEPS.forEach(step => appendTerminalLine(step));
  } else if (stepIndex === 3) {
    // Alt branch (/undo checkpoint)
    appendTerminalLine({ type: "prompt", text: "ox > /undo 1" });
    appendTerminalLine({ type: "success", text: "[DAG] Rewound session HEAD to checkpoint 01:scan. Alternative branch 02b active." });
    appendTerminalLine({ type: "user", text: "ox > /tree" });
    appendTerminalLine({ type: "banner", text: "● 00:root (init)\n└── ● 01:scan (audit safe)\n    ├── ● 02:jailed-exec (write src/cache.rs)\n    └── ◉ 02b:alt (HEAD -> current branch)" });
  }
}

function appendTerminalLine(step) {
  const body = document.getElementById("terminalBody");
  if (!body) return;

  const lineEl = document.createElement("div");
  lineEl.className = "term-line";

  if (step.type === "prompt") {
    lineEl.innerHTML = `<span class="term-prompt">$ </span><span class="term-user-text">${step.text}</span>`;
  } else if (step.type === "banner") {
    lineEl.innerHTML = `<pre class="term-banner">${step.text}</pre>`;
  } else if (step.type === "user") {
    lineEl.innerHTML = `<span class="term-prompt">user &gt; </span><span class="term-user-text">${step.text}</span>`;
  } else if (step.type === "tool") {
    lineEl.innerHTML = `<span class="term-tool">${step.text}</span>`;
  } else if (step.type === "ai") {
    lineEl.innerHTML = `<span class="term-ai-text">assistant &gt; ${step.text}</span>`;
  } else if (step.type === "security") {
    lineEl.innerHTML = `<div class="term-security-box">${step.text.replace(/\n/g, '<br>')}</div>`;
  } else if (step.type === "success") {
    lineEl.innerHTML = `<span class="term-success">${step.text}</span>`;
  }

  body.appendChild(lineEl);
  body.scrollTop = body.scrollHeight;
}

function playNextTerminalStep() {
  if (termStepIdx >= TERMINAL_STEPS.length) return;
  const step = TERMINAL_STEPS[termStepIdx];
  appendTerminalLine(step);

  termStepIdx++;
  const delay = step.type === "banner" ? 500 : step.type === "security" ? 1400 : 900;
  termTimeout = setTimeout(playNextTerminalStep, delay);
}

// Search Modal
function setupSearchKeyboardShortcuts() {
  const trigger = document.getElementById("searchTriggerBtn");
  const modal = document.getElementById("searchModal");
  const input = document.getElementById("globalSearchInput");

  if (trigger) trigger.addEventListener("click", openSearchModal);

  window.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      openSearchModal();
    }
    if (e.key === "Escape") {
      closeSearchModal();
    }
  });

  if (modal) {
    modal.addEventListener("click", (e) => {
      if (e.target === modal) closeSearchModal();
    });
  }
}

function openSearchModal() {
  const modal = document.getElementById("searchModal");
  const input = document.getElementById("globalSearchInput");
  if (modal) modal.classList.add("active");
  if (input) {
    input.value = "";
    input.focus();
    handleGlobalSearch();
  }
}

function closeSearchModal() {
  const modal = document.getElementById("searchModal");
  if (modal) modal.classList.remove("active");
}

function handleGlobalSearch() {
  const input = document.getElementById("globalSearchInput");
  const list = document.getElementById("searchResultsList");
  if (!input || !list) return;

  const q = input.value.toLowerCase().trim();
  const keys = Object.keys(DOCS_DATA);
  let results = [];

  keys.forEach(k => {
    const doc = DOCS_DATA[k];
    if (!q || doc.title.toLowerCase().includes(q) || doc.content.toLowerCase().includes(q) || doc.category.toLowerCase().includes(q)) {
      results.push({ key: k, doc });
    }
  });

  if (results.length === 0) {
    list.innerHTML = `<div style="padding: 20px; text-align: center; color: var(--text-muted);">No results found for "${escapeHtml(q)}"</div>`;
    return;
  }

  list.innerHTML = results.map(r => `
    <div class="search-result-item" onclick="selectSearchResult('${r.key}')">
      <div class="sr-category">${r.doc.category}</div>
      <div class="sr-title">${r.doc.title}</div>
      <div class="sr-snippet">${escapeHtml(r.doc.content.slice(0, 120))}...</div>
    </div>
  `).join("");
}

function selectSearchResult(key) {
  closeSearchModal();
  openDiataxisDoc(key);
}

// Mobile Menu
const mobileBtn = document.getElementById("mobileMenuBtn");
const mobileDrawer = document.getElementById("mobileDrawer");
if (mobileBtn && mobileDrawer) {
  mobileBtn.addEventListener("click", () => {
    mobileDrawer.classList.toggle("active");
  });
}

function closeMobileMenu() {
  if (mobileDrawer) mobileDrawer.classList.remove("active");
}

function switchView(view) {
  if (view === "diataxis") {
    const target = document.getElementById("diataxis");
    if (target) target.scrollIntoView({ behavior: "smooth" });
  }
}
