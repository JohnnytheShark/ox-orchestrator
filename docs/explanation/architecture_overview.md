# Explanation: Architecture Overview

`ox` is a reimagining of developer-centric AI agent harnesses built natively in Rust. This document discusses the overarching system design and design choices.

---

## Why Rust?

Traditional agent frameworks built on Node.js/TypeScript or Python suffer from three critical problems in production:
1. **Supply Chain Vulnerability Surface**: Heavy dependency trees with thousands of transitive JavaScript/Python packages exposing users to malicious post-install hooks, prototype pollution, and telemetry trackers.
2. **Runtime Overhead & Resource Footprint**: Large V8 or Python runtime heaps (often 100MB+ for simple CLI tasks) and slow startup latency.
3. **Weak Memory & Sandboxing Guarantees**: Difficulty enforcing strict memory zeroization for API secrets and deterministic isolation of path traversal.

Rust provides:
* Single self-contained static binary (~12MB release).
* Sub-millisecond startup time.
* Zero-cost security abstractions (`PathJail`, `EnvScrubber`, `zeroize`).
* Compile-time type safety for streaming protocol events and DAG trees.

---

## Architectural Data Flow

```
[ Developer Terminal ]
         │
         ▼
    [ ox-cli ] ─── (CLI arguments, TUI rendering, HITL approval gate)
         │
         ▼
   [ ox-core ] ─── (DAG SessionTree, TokenBudgeter, AgentEngine)
    ┌────┴──────────────────────────┐
    ▼                               ▼
[ ox-providers ]                [ ox-tools ]
 (Anthropic, OpenAI,             (Builtin & MCP tools
  Gemini, Ollama SSE)             jailed via ox-security)
```
