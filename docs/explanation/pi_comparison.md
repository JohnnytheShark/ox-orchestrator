# Explanation: Evolution from Pi to ox

This document outlines the design lineage from Pi (a TypeScript agent framework) to `ox` (a secure, minimalist Rust harness).

---

## Comparative Matrix

| Dimension | Pi (TypeScript / Node.js) | ox (Rust) |
|---|---|---|
| **Runtime** | Node.js / Bun | Native machine binary (Zero runtime dependencies) |
| **Startup Latency** | ~200ms - 800ms | < 5ms |
| **Binary / Bundle Size** | ~150MB+ (node_modules) | ~12MB single static executable |
| **Path Security** | Userland string parsing | Kernel-level path canonicalization (`PathJail`) |
| **Credential Scrubbing** | Partial environment filtering | Strict blacklist + regex scrubbing (`EnvScrubber`) |
| **Memory Security** | Garbage collected (strings linger indefinitely) | Heap zeroization on drop (`zeroize`) |
| **Session Model** | Flat linear list or JSON log | Directed Acyclic Graph (DAG) with non-destructive branches |
| **Extensibility** | JavaScript plugins / MCP | Dynamic stdio MCP + compiled native tools |

---

## Why Migrate from TypeScript to Rust?

1. **Security in Untrusted Workspaces**: Running an agent on open-source repositories downloaded from GitHub carries significant risks of prompt injection and arbitrary command execution. Rust allows deterministic, sandboxed file and process boundaries that are impossible to bypass via JavaScript prototype pollution.
2. **Minimalist Developer Experience**: Developers do not need to install Node.js, manage npm versions, or debug dependency mismatches. `ox` is distributed as a single standalone executable.
