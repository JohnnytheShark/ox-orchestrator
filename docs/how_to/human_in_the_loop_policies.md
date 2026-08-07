# How-To Guide: Human-in-the-Loop Security & Policies

`ox` enforces strict boundaries between read-only discovery tools and mutating tools that change the state of your computer.

---

## Tool Classification Matrix

| Tool | Type | Default Policy | Requires Approval? |
|---|---|---|---|
| `read_file` | Read-only | Path-jailed to workspace | No |
| `grep_search` | Read-only | Jailed to workspace | No |
| `find_files` | Read-only | Jailed to workspace | No |
| `write_file` | Mutating | Atomic temp-file swap | **Yes** |
| `edit_file` | Mutating | Precise surgical patch | **Yes** |
| `exec_command` | Mutating | Env-scrubbed subprocess | **Yes** |
| MCP tools | Mutating | Stdio sandboxed | **Yes** |

---

## Interactive Authorization Options

When prompted during chat:
- `y` / `Enter`: Approve this single invocation.
- `n`: Deny this action. The rejection reason is returned to the model so it can propose an alternative.
- `a`: Auto-approve all subsequent mutating actions for the rest of the current session.

---

## Toggling Auto-Approve

To toggle auto-approval during chat, type:

```text
user > /auto
Auto-approve mutating tools: true
```

Or pass `-y` / `--auto-approve` when launching:

```bash
ox chat -y
```
