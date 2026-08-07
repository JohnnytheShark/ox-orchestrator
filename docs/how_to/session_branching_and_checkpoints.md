# How-To Guide: Session Branching & DAG Checkpoints

`ox` records every conversation as a Directed Acyclic Graph (DAG), similar to Git commits. This allows you to explore multiple approaches, rewind mistakes, and switch between conversation branches without losing prior context.

---

## 1. Inspecting the Session Tree

Type `/tree` during any chat session to inspect all turns and branches:

```text
user > /tree

Session DAG [id: session-12345678 | total nodes: 5]:
  | [a1b2c3d4] (ROOT) [USER] "Build auth middleware"
  | [e5f6g7h8] <- parent: a1b2c3d4 [ASSISTANT] "Here is approach A with JWT"
  | [i9j0k1l2] <- parent: e5f6g7h8 [USER] "Let's try approach B with session cookies"
  * (ACTIVE LEAF) [m3n4o5p6] <- parent: i9j0k1l2 [ASSISTANT] "Implemented session cookies"
```

---

## 2. Rewinding with `/undo`

To undo the last turn and revert the conversation pointer to the parent node:

```text
user > /undo
Rewound to turn: i9j0k1l2
```

The next message you send will create a new branch branching off from `i9j0k1l2`, leaving `m3n4o5p6` intact in history.

---

## 3. Checking out a Historical Node

You can switch the active conversation context to any node in the graph:

```text
user > /checkout a1b2c3d4
Checked out node a1b2c3d4
```

Now you can start a completely new line of exploration from the root question.

---

## 4. Exporting Sessions

To export a session to markdown documentation:

```bash
ox session export <session_id> --output notes.md
```
