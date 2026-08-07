# Reference: Crate Architecture

The `ox-orchestrator` repository is organized as a modular Cargo workspace designed for high cohesion and low coupling.

```
ox-orchestrator/
├── crates/
│   ├── ox-core/         # Core abstractions, DAG session tree, context window manager, engine loop
│   ├── ox-security/     # Sandboxing, PathJail canonicalization, EnvScrubber, Secret zeroization
│   ├── ox-providers/    # LLM provider adapters (Anthropic, OpenAI, Gemini, Ollama) and SSE streaming
│   ├── ox-tools/        # Built-in tools and stdio MCP client
│   └── ox-cli/          # Command line binary, TUI streaming renderer, HITL authorization gate
```

---

## 1. `ox-core`
- **`types`**: Common domain types (`Message`, `ContentBlock`, `ToolCall`, `ToolResult`, `TokenUsage`).
- **`session`**: Branching Directed Acyclic Graph (`SessionTree`, `SessionNode`, `NodeId`, `SessionStorage`).
- **`context`**: Context window optimizer (`TokenBudgeter`, `ContextCompactor`, `SystemPromptBuilder`).
- **`agent`**: Re-entrant reasoning engine (`AgentEngine`, `AgentConfig`, `StreamEvent`).

---

## 2. `ox-security`
- **`PathJail`**: Zero-overhead filesystem boundary enforcement ensuring no symlinks, relative traversal (`..`), or drive escaping access files outside workspace.
- **`EnvScrubber`**: Blacklists credential variables (`*_API_KEY`, `*TOKEN*`, `*SECRET*`) before spawning child processes.
- **`Secret`**: Zeroizes memory upon drop using compiler fences to prevent memory dumps from leaking keys.

---

## 3. `ox-providers`
- **`LlmProvider`**: Async streaming trait defining `stream_chat`.
- **`AnthropicClient`**: Claude 3.5 / 3.7 SSE parser, thinking token block parser, tool mapper.
- **`OpenAiClient`**: GPT-4o, DeepSeek, and Ollama SSE chunk decoder.
- **`GeminiClient`**: Google Gemini REST SSE streamer.

---

## 4. `ox-tools`
- **`Tool`**: Unified async trait for tool execution and JSON schema definitions.
- **`ToolDispatcher`**: Registry and router for active tools.
- **`builtin`**: `read_file`, `write_file`, `edit_file`, `exec_command`, `grep_search`, `find_files`.
- **`mcp`**: JSON-RPC 2.0 stdio MCP client and dynamic tool adapter.

---

## 5. `ox-cli`
- **`cli_args`**: Clap argument definition.
- **`tui`**: Colorized terminal output and interactive HITL prompt.
- **`commands`**: `chat`, `run`, `session`, `tools`.
