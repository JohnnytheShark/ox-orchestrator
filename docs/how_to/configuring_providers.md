# How-To Guide: Configuring Providers

This guide shows how to configure and switch between different LLM providers in `ox`.

---

## Supported Providers

| Provider | Provider Flag | Default Model | Environment Variable |
|---|---|---|---|
| **Anthropic** | `--provider anthropic` | `claude-3-7-sonnet-20250219` | `ANTHROPIC_API_KEY` |
| **OpenAI** | `--provider openai` | `gpt-4o` | `OPENAI_API_KEY` |
| **Google Gemini** | `--provider gemini` | `gemini-2.0-flash` | `GEMINI_API_KEY` |
| **DeepSeek** | `--provider openai` | `deepseek-chat` | `DEEPSEEK_API_KEY` |
| **Ollama (Local)** | `--provider ollama` | `llama3.3` | None (Local) |

---

## 0. Using the Setup Wizard (Recommended)

The easiest way to configure a provider is the interactive setup wizard:

```bash
ox setup
```

The wizard saves your choice to `~/.config/ox/config.toml` under `[agent]` and stores the API key under `[credentials]`:

```toml
[agent]
provider = "anthropic"
model    = "claude-3-7-sonnet-20250219"

[credentials]
anthropic_api_key = "sk-ant-..."
```

Running `ox setup` a second time to add a new provider's key **merges** — your existing keys are preserved.

---

## 1. Using CLI Flags

Pass `--provider` and `--model` directly to `ox`:

```bash
# Use OpenAI GPT-4o
ox chat --provider openai --model gpt-4o

# Use DeepSeek via OpenAI-compatible API
ox chat --provider openai --model deepseek-chat --base-url https://api.deepseek.com/v1

# Use Local Ollama
ox chat --provider ollama --model qwen2.5-coder:14b
```

---

## 2. Using Workspace Configuration

To persist default model choices for a project, define them in `ox.toml`:

```toml
[agent]
provider = "openai"
model    = "gpt-4o"
base_url = "https://api.openai.com/v1"

[credentials]
openai_api_key = "sk-..."
```

When you launch `ox chat` in that directory, it will automatically load these defaults.
