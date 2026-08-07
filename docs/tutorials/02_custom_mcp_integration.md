# Tutorial: Custom MCP Integration

In this tutorial, you will learn how to connect an external Model Context Protocol (MCP) server to `ox` to provide database queries, git operations, or browser automation tools.

---

## What is MCP in ox?

The Model Context Protocol (MCP) is an open standard allowing external processes to provide tools dynamically to AI models. In `ox`, MCP servers run as isolated child processes communicating via JSON-RPC 2.0 over standard input and output (stdio).

---

## Step 1: Create a Workspace Config File

Inside your project root directory, create `.ox/config.json`:

```json
{
  "default_model": "claude-3-7-sonnet-20250219",
  "mcp_servers": {
    "sqlite": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "app.db"]
    }
  }
}
```

---

## Step 2: Inspecting Discovered Tools

Launch `ox tools` to verify that `ox` connects to the MCP server, completes the handshake, and discovers its tools:

```bash
ox tools
```

Output:
```text
Registered Tools (7):

  * read_file        [READ-ONLY / SAFE]
  * write_file       [MUTATING - REQUIRES HITL APPROVAL]
  * edit_file        [MUTATING - REQUIRES HITL APPROVAL]
  * exec_command     [MUTATING - REQUIRES HITL APPROVAL]
  * grep_search      [READ-ONLY / SAFE]
  * find_files       [READ-ONLY / SAFE]
  * sqlite__query    [MUTATING - REQUIRES HITL APPROVAL]
    Description: Execute a SQL query against the SQLite database
```

---

## Step 3: Using MCP Tools in Chat

Start an interactive chat session:

```bash
ox chat
```

Prompt:
```text
user > Show me the schema of the users table and list the last 5 signups.
```

`ox` will invoke `sqlite__query` and prompt you for confirmation before running the query against your database.
