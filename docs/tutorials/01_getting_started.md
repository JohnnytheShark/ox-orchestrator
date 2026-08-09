# Tutorial: Getting Started with ox

In this tutorial, you will learn how to build, configure, and use `ox` to assist in exploring and modifying a software repository.

---

## Prerequisites
- Rust 1.80+ (`rustup default stable`)
- An API key for your preferred LLM provider (Anthropic Claude, OpenAI, Google Gemini, DeepSeek, or a local Ollama instance).

---

## Step 1: Building and Installing ox

Clone the repository and build the release binary:

```bash
cargo build --release -p ox-cli
```

Optionally install `ox` to your cargo bin directory:

```bash
cargo install --path crates/ox-cli
```

Verify that `ox` is ready:

```bash
ox --help
```

---

## Step 2: Configure Your Provider

Run the interactive setup wizard to configure your provider, model, and API key:

```bash
ox setup
```

The wizard will guide you through:
1. Choosing your LLM provider (Anthropic, OpenAI, Gemini, or local Ollama)
2. Selecting a model (sensible defaults suggested per provider)
3. Entering your API key with **hidden input** (key is never echoed to the terminal)
4. Saving to a global config (`~/.config/ox/config.toml`) or a local `ox.toml`

> **Prefer env vars?** You can skip the wizard by setting `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `GEMINI_API_KEY` before running `ox`. The wizard reads these automatically.

> **Re-run anytime:** `ox setup` on an existing config offers a credentials-only shortcut — rotate a key for one provider without re-confirming your model.

---

## Step 3: Launching an Interactive Session

Navigate to any codebase directory and start `ox`:

```bash
cd /path/to/your/project
ox
```

You will be greeted by the `ox` banner showing your active provider, workspace path, and session ID:

```text
   ____  _  __
  / __ \| |/ /   ox-orchestrator v0.1.0
 / /_/ /|   /    Minimalist & Secure AI Agent Harness
 \____//_/|_\    

 Provider  : Anthropic (claude-3-7-sonnet-20250219)
 Workspace : /path/to/your/project
 Session   : session-a1b2c3d4
 Security  : Path-Jailed, Env-Scrubbed, Zeroized Secrets

user >
```

---

## Step 4: Interacting with Code and Tools

Ask `ox` to inspect your repository:

```text
user > Find all Rust files and summarize their responsibilities.
```

`ox` will invoke `find_files`, read relevant files with `read_file`, and provide a concise summary.

When `ox` wishes to perform a mutating action (such as writing or modifying a file), you will be prompted by the Human-in-the-Loop security gate:

```text
[SECURITY AUDIT] Tool Execution Request:
  Tool: write_file
  Arguments: {
    "path": "src/utils.rs",
    "content": "..."
  }
Authorize execution? [y]es / [n]o / [a]lways approve:
```

Type `y` to approve, `n` to reject, or `a` to approve all subsequent actions in this session.

---

## Step 5: Session Checkpoints and Rewind

If `ox` takes an unwanted path, use `/undo` to step back one turn, or `/tree` to inspect all branches in the session DAG:

```text
user > /tree

Session DAG [id: session-a1b2c3d4 | total nodes: 4]:
  | [d4e5f6a1] (ROOT) [USER] "Find all Rust files..."
  | [b2c3d4e5] <- parent: d4e5f6a1 [ASSISTANT] "Found 6 files..."
  * (ACTIVE LEAF) [c3d4e5f6] <- parent: b2c3d4e5 [USER] "Now refactor utils.rs"

user > /undo
Rewound to turn: b2c3d4e5
```

---

## Next Steps
- [Custom MCP Integration](02_custom_mcp_integration.md) — Connect external tool servers.
- [Configuring Providers](../how_to/configuring_providers.md) — Switch models and endpoints.
