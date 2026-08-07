# Security Policy & Responsible Disclosure

The `ox-orchestrator` project is built with a security-first architecture to protect developers running autonomous and semi-autonomous AI coding agents in local workspaces.

---

## Supported Versions

Only the latest release of `ox-orchestrator` receives active security updates and vulnerability patches.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Security Guarantees & Built-in Defenses

`ox-orchestrator` enforces strict defense-in-depth guarantees:

1. **Path Containment (`PathJail`)**:
   - All filesystem tools (`read_file`, `write_file`, `edit_file`, `grep_search`, `find_files`) strictly validate target paths against the canonical root of the workspace.
   - Relative paths escaping the root via `../` sequences or symbolic links pointing outside the sandbox are rejected with `SecurityError::PathEscapeAttempt`.

2. **Subprocess Credential Scrubbing (`EnvScrubber`)**:
   - Shell commands executed via `exec_command` or child MCP servers are spawned with sanitized environments.
   - Sensitive variables matching patterns such as `*API_KEY*`, `*SECRET*`, `*TOKEN*`, `*PASSWORD*`, and cloud credentials are removed before process launch.

3. **Memory Safety & Key Zeroization (`Secret`)**:
   - Sensitive credentials and keys stored in memory are wrapped in zeroizing containers that overwrite heap buffers with zeroes immediately upon `Drop`.

4. **Human-in-the-Loop (HITL) Guardrail**:
   - Any mutating action (file writes, edits, terminal commands) halts for operator confirmation in the interactive CLI unless explicit bypass flags (`-y` / `--auto-approve`) are passed.

---

## Reporting a Vulnerability

If you discover a security vulnerability, please **do not open a public issue**. Instead, follow these steps:

1. **Email Disclosure**: Send details to `security@yakherd.dev` (or open a Private Vulnerability Advisory on GitHub).
2. **Details to Include**:
   - Description of the vulnerability and attack vector.
   - Step-by-step proof-of-concept (PoC) or minimal reproduction script.
   - Impact assessment on host filesystems or developer environments.
   - Potential fix or mitigation (if known).
3. **Response Timeline**:
   - Initial acknowledgement within 48 hours.
   - Status update and vulnerability validation within 5 business days.
   - Coordinated public disclosure and patch release once resolved.
