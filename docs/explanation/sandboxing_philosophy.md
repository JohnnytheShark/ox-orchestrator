# Explanation: Sandboxing Philosophy & Defense in Depth

This document explains the security architecture of `ox` and how it implements defense in depth against prompt injection, unauthorized mutations, and data exfiltration.

---

## 1. The Jail Principle: Filesystem Containment

In modern agent development, models are tasked with creating, editing, and deleting files. An LLM may attempt to access sensitive paths outside the project root (e.g. `~/.ssh/id_rsa`, `~/.aws/credentials`, or `/etc/hosts`) due to hallucination or malicious instructions injected into issues or code comments.

`ox` prevents this via `PathJail`:
1. Every path argument is resolved relative to the jail root.
2. Symbolic links are resolved using OS canonicalization.
3. If the resulting path does not start with the workspace root, execution is aborted immediately before any filesystem syscall occurs.

---

## 2. The Clean-Room Principle: Environment Scrubbing

When an agent needs to execute tests or build tools (`cargo test`, `npm run build`), spawning a subprocess with the default `std::env::vars()` leaks the operator's private API keys (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, cloud credentials).

If a dependency build script or untrusted test inspects `process.env`, credentials could be silently exfiltrated.

`ox` sanitizes every subprocess execution through `EnvScrubber`, stripping all known sensitive prefixes and secret keywords before spawning.

---

## 3. Human-in-the-Loop as a First-Class Guardrail

Rather than relying purely on probabilistic model self-censorship, `ox` uses hard deterministic gates:
* Read-only inspection tools (`read_file`, `grep_search`, `find_files`) run without interrupting flow.
* State-altering tools (`write_file`, `edit_file`, `exec_command`, dynamic MCP tools) stop execution and render an interactive confirmation dialog with diff preview and parameter inspection.
