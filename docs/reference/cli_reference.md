# Reference: Command Line Interface (CLI)

Complete reference for all `ox` command line arguments, flags, and interactive slash commands.

---

## Global Options

| Flag | Long Form | Description | Default |
|---|---|---|---|
| `-m` | `--model <NAME>` | Model identifier (e.g. `claude-3-7-sonnet-20250219`, `gpt-4o`) | Inferred from provider or config |
| `-p` | `--provider <NAME>` | Provider family: `anthropic`, `openai`, `gemini`, `ollama`, `custom` | `anthropic` |
| `-w` | `--workspace <PATH>` | Target workspace root directory | Current directory or `.git` parent |
| | `--base-url <URL>` | Custom API base URL | Standard provider endpoint |
| `-y` | `--auto-approve` | Automatically approve mutating tools | `false` |
| `-v` | `--verbose` | Enable debug logs | `false` |
| `-h` | `--help` | Print help information | |
| `-V` | `--version` | Print version information | |

---

## Subcommands

### 1. `ox chat`
Starts interactive REPL session with DAG branching history.
* `-s, --session <ID>`: Resume an existing session by ID.
* `-p, --prompt <TEXT>`: Initial instruction to execute immediately upon starting.

### 2. `ox run`
Non-interactive single-command batch execution.
* `<PROMPT>`: Prompt text to execute.
* `--max-turns <N>`: Maximum allowed reasoning steps (default: 30).

### 3. `ox session`
* `list`: List all stored sessions.
* `tree <session_id>`: Render ASCII tree of conversational DAG.
* `export <session_id> [-o <path>]`: Export session history to Markdown.

### 4. `ox tools`
Lists all registered built-in tools and connected MCP tools with descriptions and JSON parameter schemas.

---

## Interactive Slash Commands

| Command | Description |
|---|---|
| `/cost` | Displays cumulative session token usage, cache statistics, and estimated cost |
| `/diff` | Inspects current git working tree modifications |
| `/undo` | Rewinds the conversation leaf pointer to the previous turn |
| `/tree` | Visualizes the full DAG tree of branches and turns |
| `/checkout <id>` | Switches the active conversation pointer to node `<id>` |
| `/history` | Displays the linear sequence of messages from root to leaf |
| `/auto` | Toggles automatic approval of mutating tools on/off |
| `/save` | Forces a snapshot save of the session tree to disk |
| `/help` | Prints the slash command quick reference |
| `/exit`, `/quit` | Saves the session and closes the REPL |
