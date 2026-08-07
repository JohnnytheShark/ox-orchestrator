# Contributing to ox-orchestrator

Thank you for your interest in contributing to `ox-orchestrator`! We welcome contributions from the community to help make `ox` the most secure, performant, and reliable AI agent harness.

---

## Code of Conduct

All contributors and maintainers are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## Development Setup

### Prerequisites
* Rust 1.80 or later (`rustup update stable`)
* Git
* `cargo fmt` and `cargo clippy` (`rustup component add rustfmt clippy`)

### Building the Workspace
```bash
git clone https://github.com/yak-herd/ox-orchestrator.git
cd ox-orchestrator
cargo build --workspace
```

---

## Coding Standards & Guidelines

1. **Security-First**:
   - Never bypass `PathJail` for workspace file access.
   - Always sanitize child process environments with `EnvScrubber`.
   - Never log raw secrets, API keys, or full auth headers.
2. **Minimal & Clean Code**:
   - Avoid bloated dependencies. If a small utility can be cleanly implemented internally, prefer native standard library implementation.
   - Keep crate interfaces small, decoupled, and strictly typed.
3. **Linting & Formatting**:
   - All code must pass `cargo clippy --workspace -- -D warnings`.
   - All code must be formatted using `cargo fmt --check`.
4. **Testing**:
   - Add unit tests for new features or bug fixes.
   - Run the full test suite before opening a pull request:
     ```bash
     cargo test --workspace
     ```

---

## Pull Request Process

1. Fork the repository and create a new feature branch (`git checkout -b feature/my-feature`).
2. Implement your changes following our coding and security guidelines.
3. Verify formatting, linter, and tests pass:
   ```bash
   cargo fmt --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   ```
4. Commit your changes with clear, descriptive commit messages.
5. Push to your fork and submit a Pull Request against `main`.
