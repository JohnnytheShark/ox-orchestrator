 # ox-orchestrator (`ox`)

> **A high-performance, minimalist, and secure AI coding agent harness built in Rust.**

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/JohnnytheShark/ox-orchestrator?color=green&label=release)](https://github.com/JohnnytheShark/ox-orchestrator/releases)
[![Docs](https://img.shields.io/badge/docs-Diátaxis%20Framework-purple.svg)](https://JohnnytheShark.github.io/ox-orchestrator/)
[![Safety](https://img.shields.io/badge/security-path--jailed%20%7C%20env--scrubbed-green.svg)](#security-guarantees)

---

## Quick Install

### Linux & macOS (One-Liner)
```bash
curl -fsSL https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/JohnnytheShark/ox-orchestrator/main/install.ps1 | iex
```

### From Source via Cargo
```bash
cargo install --git https://github.com/JohnnytheShark/ox-orchestrator ox-cli
```

---

## Highlights

* **Single Standalone Executable**: Native machine binary (~5MB) with zero Node.js/Python dependencies.
* **Kernel-Level Sandboxing (`PathJail`)**: Zero-overhead canonicalization protecting against directory traversal, path escaping, and symlink exploits.
* **Subprocess Environment Scrubbing (`EnvScrubber`)**: Automatically scrubs API keys (`*_API_KEY`, `*TOKEN*`, `*SECRET*`) from subprocesses and MCP child servers.
* **Zeroized Memory Credentials**: Overwrites sensitive keys and memory buffers with zeroes upon drop using compiler fences (`zeroize`).
* **Non-Destructive DAG Session Branching**: Branch, rewind (`/undo`), and checkout historical checkpoints without losing previous reasoning paths.
* **Universal Provider Agnosticism**: First-class streaming support for Anthropic (Claude 3.5 / 3.7 Sonnet), OpenAI (GPT-4o, o1, o3), Google Gemini 2.0, DeepSeek, and local Ollama.
* **Human-in-the-Loop Gate (HITL)**: Read-only inspection tools run smoothly while mutating operations (`write_file`, `edit_file`, `exec_command`, MCP) require explicit confirmation unless overridden.
* **Model Context Protocol (MCP)**: Native stdio JSON-RPC 2.0 client to seamlessly connect SQLite, GitHub, Filesystem, or custom MCP servers.
* **Diátaxis Documentation & Portal**: Complete structured documentation and interactive web app at [JohnnytheShark.github.io/ox-orchestrator](https://JohnnytheShark.github.io/ox-orchestrator/).

---

## Precompiled Binary Downloads

Official standalone binaries with Link-Time Optimization (LTO):

| Platform | Architecture | Standalone Package | Checksum |
|---|---|---|---|
| **Linux** | x86_64 (glibc) | [`ox-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-unknown-linux-gnu.tar.gz) | [SHA-256](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256) |
| **Linux** | x86_64 (musl static) | [`ox-v0.1.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-unknown-linux-musl.tar.gz) | [SHA-256](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-unknown-linux-musl.tar.gz.sha256) |
| **Linux** | ARM64 / AArch64 | [`ox-v0.1.0-aarch64-unknown-linux-gnu.tar.gz`](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-aarch64-unknown-linux-gnu.tar.gz) | [SHA-256](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-aarch64-unknown-linux-gnu.tar.gz.sha256) |
| **macOS** | Apple Silicon (M1–M4) | [`ox-v0.1.0-aarch64-apple-darwin.tar.gz`](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-aarch64-apple-darwin.tar.gz) | [SHA-256](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-aarch64-apple-darwin.tar.gz.sha256) |
| **Windows** | x86_64 | [`ox-v0.1.0-x86_64-pc-windows-msvc.zip`](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-pc-windows-msvc.zip) | [SHA-256](https://github.com/JohnnytheShark/ox-orchestrator/releases/download/v0.1.0/ox-v0.1.0-x86_64-pc-windows-msvc.zip.sha256) |

> Check out all releases and historical changelogs on the **[Releases Page](https://github.com/JohnnytheShark/ox-orchestrator/releases)**.


---

## Quick Start

### 1. Set Your API Key

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export OPENAI_API_KEY="sk-..."
```

### 2. Start Interactive Chat

```bash
cd /path/to/your/project
ox chat
```

### 3. Non-Interactive CI/CD Run

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
├── site/                # Diátaxis GitHub Pages Static Web App Portal
├── docs/                # Diátaxis Markdown Documentation
└── tests/               # End-to-end integration tests
```

---

## Documentation

Explore the full documentation online at **[JohnnytheShark.github.io/ox-orchestrator](https://JohnnytheShark.github.io/ox-orchestrator/)** or in [`docs/`](docs/README.md):

* **[Tutorials](docs/tutorials/01_getting_started.md)**: Hands-on learning guides.
* **[How-To Guides](docs/how_to/configuring_providers.md)**: Recipes for providers, DAG branching, HITL policies, and CI/CD.
* **[Reference](docs/reference/cli_reference.md)**: Command line reference, crate APIs, security model, and config schema.
* **[Explanation](docs/explanation/architecture_overview.md)**: System design, why Rust, and evolution from Pi.

---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).
Copyright 2026 Johnny Orellana and ox-orchestrator contributors.


