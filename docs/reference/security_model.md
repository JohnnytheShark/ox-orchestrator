# Reference: Security Model & Guarantees

Security is the foundational design pillar of `ox`. This document specifies the threat models and defense mechanisms implemented across the codebase.

---

## 1. Threat Models & Mitigations

### A. Path Traversal & Symlink Attacks
* **Threat**: Malicious prompts or hallucinated paths (e.g. `../../etc/passwd` or `C:\Windows\System32`) reading or corrupting host system files.
* **Mitigation**: `PathJail` enforces canonicalization on all read, write, edit, and search operations. Any path that does not begin with the canonical root of the workspace is rejected with `SecurityError::PathEscapeAttempt`.

### B. Environment Credential Leakage
* **Threat**: Subprocesses spawned by `exec_command` or untrusted MCP servers reading `ANTHROPIC_API_KEY`, `AWS_SECRET_ACCESS_KEY`, or `GITHUB_TOKEN` from inherited process environments.
* **Mitigation**: `EnvScrubber` strips all matching prefixes and sensitive keywords from child process environment maps before launching.

### C. Secret Lifetime in Memory
* **Threat**: API keys lingering in process memory heap and appearing in crash dumps or debug logs.
* **Mitigation**: `Secret` uses `zeroize` to overwrite key memory buffers with zeroes immediately upon `Drop`.

### D. Unauthorized State Mutation
* **Threat**: Models deleting databases, overwriting sensitive files, or running remote network requests without oversight.
* **Mitigation**: Mutating tools require explicit human approval via the interactive TUI prompt unless `--auto-approve` is explicitly requested by the operator.
