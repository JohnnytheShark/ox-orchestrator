# ox Documentation

Welcome to the documentation for **`ox-orchestrator`** (`ox`), a high-performance, minimalist, and secure AI coding agent harness built in Rust.

This documentation follows the [Diátaxis](https://diataxis.fr/) framework, organizing content into four distinct categories based on user needs:

```
                      PRACTICAL
                         ▲
                         │
     [Tutorials]         │        [How-To Guides]
   Learning-oriented     │       Problem-oriented
                         │
 ACQUISITION ────────────┼──────────── APPLICATION
                         │
     [Explanation]       │         [Reference]
 Understanding-oriented  │     Information-oriented
                         │
                         ▼
                    THEORETICAL
```

---

## 1. 🎓 Tutorials (Learning-Oriented)
Step-by-step learning journeys for newcomers:
* [Getting Started](tutorials/01_getting_started.md) — Install `ox`, configure your first provider, and execute your first prompt.
* [Custom MCP Integration](tutorials/02_custom_mcp_integration.md) — Connect an external Model Context Protocol (MCP) server over stdio.

## 2. 🛠️ How-To Guides (Problem-Oriented)
Practical step-by-step recipes for specific tasks:
* [Configuring Providers](how_to/configuring_providers.md) — Setting up Anthropic, OpenAI, Gemini, DeepSeek, and local Ollama.
* [Session Branching & DAG Checkpoints](how_to/session_branching_and_checkpoints.md) — Navigating branches, rewinding, and checking out checkpoints.
* [Human-in-the-Loop Security](how_to/human_in_the_loop_policies.md) — Managing tool approval gates, auto-approve flags, and security boundaries.
* [Running in CI/CD](how_to/running_in_ci_cd.md) — Automating codebase tasks in non-interactive batch mode with `ox run`.

## 3. 📖 Reference (Information-Oriented)
Technical facts, APIs, schemas, and specifications:
* [CLI Reference](reference/cli_reference.md) — All flags, subcommands, and interactive slash commands.
* [Crate Architecture](reference/crate_architecture.md) — Internal crate breakdown and API contracts.
* [Security Model](reference/security_model.md) — Path jail guarantees, environment scrubber rules, and zeroized secrets.
* [Configuration Schema](reference/configuration_schema.md) — Format and options for `.ox/config.json`.

## 4. 💡 Explanation (Understanding-Oriented)
High-level context, design rationale, and background:
* [Architecture Overview](explanation/architecture_overview.md) — Design principles and why Rust was chosen over TypeScript.
* [Session DAG vs Linear History](explanation/session_dag_vs_linear_history.md) — The mechanics of tree-structured conversational state.
* [Comparison with Pi](explanation/pi_comparison.md) — Evolution from Pi's TypeScript agent harness to ox.
* [Sandboxing Philosophy](explanation/sandboxing_philosophy.md) — Defending against prompt injection, rogue writes, and secret leakage.
