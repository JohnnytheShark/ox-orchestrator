# Explanation: Session DAG vs Linear History

Most conversational agent interfaces maintain history as a flat, linear array of messages (`Vec<Message>`). This document explains why `ox` models conversations as a Directed Acyclic Graph (DAG) of immutable turn nodes.

---

## The Problem with Flat Lists

When working on complex software engineering tasks, developers frequently encounter dead ends or want to evaluate competing implementation strategies:
- *"What if we use Tokio instead of Async-std?"*
- *"Let's try regex parsing vs a hand-written recursive descent parser."*

In a flat array:
* Undoing a turn permanently deletes all downstream reasoning and code snippets.
* Branching requires copying the entire conversation history into a new disconnected file.
* There is no checkpoint lineage tracking parent-child relationships.

---

## The DAG Model in ox

In `ox`, every turn is an immutable `SessionNode` referenced by a cryptographic or unique `NodeId`:

```
           [Node A: "Refactor database queries"]
                           │
             ┌─────────────┴─────────────┐
             ▼                           ▼
[Node B: "Use SQLx with async"]   [Node C: "Use Diesel ORM"]
             │
             ▼
[Node D: "Added connection pool"]
```

### Key Benefits
1. **Zero Data Loss**: Undoing a turn (`/undo`) simply moves the `current_leaf_id` pointer back to the parent node. The child branches remain in disk storage and can be inspected or checked out at any time (`/checkout <id>`).
2. **Context Path Projection**: When preparing the context window for an LLM prompt, `ox` traces from the active leaf up to the root to produce the active linear history.
3. **Session Replayability**: Complete diagnostic replays of agent decisions can be generated for debugging or audit compliance.
