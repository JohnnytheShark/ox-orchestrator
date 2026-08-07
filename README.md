 # ox-orchestrator (`ox`)

> **A high-performance, minimalist, and secure AI coding agent harness built in Rust.**

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Safety](https://img.shields.io/badge/security-path--jailed%20%7C%20env--scrubbed-green.svg)](#security-guarantees)

---

## Highlights

* **Single Standalone Executable**: Native machine binary (~12MB) with zero Node.js/Python dependencies.
* **Kernel-Level Sandboxing (`PathJail`)**: Zero-overhead canonicalization protecting against directory traversal, path escaping, and symlink exploits.
* **Subprocess Environment Scrubbing (`EnvScrubber`)**: Automatically scrubs API keys (`*_API_KEY`, `*TOKEN*`, `*SECRET*`) from subprocesses and MCP child servers.
* **Zeroized Memory Credentials**: Overwrites sensitive keys and memory buffers with zeroes upon drop using compiler fences (`zeroize`).
* **Non-Destructive DAG Session Branching**: Branch, rewind (`/undo`), and checkout historical checkpoints without losing previous reasoning paths.
* **Universal Provider Agnosticism**: First-class streaming support for Anthropic (Claude 3.5 / 3.7 Sonnet), OpenAI (GPT-4o, o1, o3), Google Gemini 2.0, DeepSeek, and local Ollama.
* **Human-in-the-Loop Gate (HITL)**: Read-only inspection tools run smoothly while mutating operations (`write_file`, `edit_file`, `exec_command`, MCP) require explicit confirmation unless overridden.
* **Model Context Protocol (MCP)**: Native stdio JSON-RPC 2.0 client to seamlessly connect SQLite, GitHub, Filesystem, or custom MCP servers.
* **Diátaxis Documentation**: Complete structured documentation under [`docs/`](docs/README.md).

---

## Quick Start

### 1. Installation

```bash
# Build from source
cargo install --path crates/ox-cli
```

### 2. Set Your API Key

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export OPENAI_API_KEY="sk-..."
```

### 3. Start Interactive Chat

```bash
cd /path/to/your/project
ox chat
```

### 4. Non-Interactive CI/CD Run

```bash
ox run "Review all changed files, run cargo check, and fix compiler warnings." -y
```

---

## Architecture Overview

```
ox-orchestrator/
├── crates/
│   ├── ox-core/         # Session DAG, TokenBudgeter, ContextCompactor, AgentEngine
│   ├── ox-security/     # PathJail, EnvScrubber, Zeroize Memory Protection
│   ├── ox-providers/    # Anthropic, OpenAI, Gemini, Ollama SSE Adapters
│   ├── ox-tools/        # Builtin tools (read, write, edit, grep, find, exec) & MCP
│   └── ox-cli/          # Clap CLI, Terminal UI Renderer, HITL Gate
├── docs/                # Diátaxis Documentation (Tutorials, How-To, Reference, Explanation)
└── tests/               # End-to-end integration tests
```

---

## Documentation

Explore the full documentation in [`docs/`](docs/README.md):

* **[Tutorials](docs/tutorials/01_getting_started.md)**: Hands-on learning guides.
* **[How-To Guides](docs/how_to/configuring_providers.md)**: Recipes for providers, DAG branching, HITL policies, and CI/CD.
* **[Reference](docs/reference/cli_reference.md)**: Command line reference, crate APIs, security model, and config schema.
* **[Explanation](docs/explanation/architecture_overview.md)**: System design, why Rust, and evolution from Pi.

---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).
Copyright 2026 Johnny Orellana and ox-orchestrator contributors.
