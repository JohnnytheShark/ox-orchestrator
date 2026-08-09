# Reference: Configuration Schema & Workspace Rules

`ox-orchestrator` supports declarative configuration via TOML files, automatic repository instruction file injection, and file masking via `.oxignore`.

---

## 1. Hierarchical Configuration (`ox.toml`)

Configurations are merged hierarchically with the following precedence order (highest to lowest):
1. **CLI Flags** (e.g. `--provider anthropic --model claude-3-7-sonnet`)
2. **Workspace Configuration** (`./ox.toml`)
3. **Global User Configuration** (`~/.config/ox/config.toml`)
4. **Provider defaults (via** `ox setup`**)**

### Example `ox.toml`

```toml
[agent]
provider     = "anthropic"
model        = "claude-3-7-sonnet-20250219"
base_url     = "https://api.anthropic.com/v1"
auto_approve = false
max_turns    = 40
max_context_tokens = 200000

[credentials]
anthropic_api_key = "sk-ant-..."
# openai_api_key  = "sk-..."   # add other providers as needed
# gemini_api_key  = "AIza..."

[mcp_servers.filesystem]
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-filesystem", "./data"]
env     = { DEBUG = "1" }

[mcp_servers.memory]
command = "npx"
args    = ["-y", "@modelcontextprotocol/server-memory"]
```

### Configuration Fields

| Section | Field | Type | Description |
|---|---|---|---|
| `[agent]` | `provider` | String | Provider identifier (`anthropic`, `openai`, `gemini`, `ollama`, `custom`) |
| `[agent]` | `model` | String | LLM model identifier |
| `[agent]` | `base_url` | String (Optional) | Custom base URL for self-hosted or proxy endpoints |
| `[agent]` | `auto_approve` | Boolean | Whether to skip HITL prompts for mutating tools (default `false`) |
| `[agent]` | `max_turns` | Integer | Max reasoning loop turns per session (default `30`) |
| `[agent]` | `max_context_tokens` | Integer | Token budget cap for compaction triggers (default `128000`) |
| `[credentials]` | `anthropic_api_key` | String (Optional) | Anthropic API key (written by `ox setup`, takes lower priority than `ANTHROPIC_API_KEY` env var) |
| `[credentials]` | `openai_api_key` | String (Optional) | OpenAI API key |
| `[credentials]` | `gemini_api_key` | String (Optional) | Google Gemini API key |
| `[mcp_servers.<name>]` | `command` | String | Executable command to launch (e.g. `npx`, `python`, `cargo`) |
| `[mcp_servers.<name>]` | `args` | Array of Strings | Command-line arguments passed to the MCP server process |
| `[mcp_servers.<name>]` | `env` | Table of Key/Values | Custom environment variables passed to the child process |

---

## 2. Repository Instruction Files (`AGENTS.md` / `OX.md`)

When starting any agent session, `ox-orchestrator` automatically searches the workspace hierarchy for project instruction files:
1. `AGENTS.md` in workspace root or subdirectories
2. `OX.md` in workspace root or subdirectories
3. `.agents.md` / `.ox.md` (hidden variant)

If discovered, instructions are automatically appended to the system prompt inside a dedicated `<repository_instructions>` XML block.

---

## 3. File Masking (`.oxignore`)

`ox-orchestrator` enforces workspace boundaries and shields sensitive files from being indexed, read, or modified by LLM agents.

### Precedence & Sources
1. Standard `.git/`, `target/`, `node_modules/` are excluded by default.
2. Standard `.gitignore` rules in the workspace are respected.
3. Custom `.oxignore` files in the workspace root define additional agent-specific masks.

### Affected Tools
- `find_files`: Excluded files/folders are not returned in listings.
- `grep_search`: Excluded files/folders are bypassed during search scans.
- `read_file`, `edit_file`, `write_file`: Refuse access with a safe error message indicating the target is ignored.

