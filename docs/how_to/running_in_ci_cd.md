# How-To Guide: Running in CI/CD & Automated Pipelines

`ox run` enables non-interactive, automated execution of prompts for code review, lint fixing, test generation, and migration scripts.

---

## 1. Syntax

```bash
ox run "<prompt>" [--max-turns <N>] [--auto-approve]
```

---

## 2. GitHub Actions Example

```yaml
name: AI Code Review & Lint Fix

on:
  pull_request:
    branches: [ main ]

jobs:
  ox-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build ox
        run: cargo install --path crates/ox-cli

      - name: Run automated audit
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          ox run "Audit all modified files in this PR, run cargo check, and fix any compiler warnings." -y
```

---

## 3. Scripting and Piping

You can pipe outputs from standard tools directly into `ox`:

```bash
cargo check 2>&1 | ox run "Analyze compiler errors and apply fixes to resolve them" -y
```
