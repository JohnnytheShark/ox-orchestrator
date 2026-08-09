use crate::config::{ConfigResolver, WizardInputs};
use crate::tui::TerminalRenderer;
use crossterm::execute;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ox_providers::{ProviderConfig, ProviderType};
use std::io::{self, Write};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Typed error — P1[K] Kinetic Hygiene
// ---------------------------------------------------------------------------

/// Structured error type for the setup wizard.
/// Allows callers to pattern-match specific failure modes rather than
/// receiving an opaque `Box<dyn Error>`.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("API key cannot be empty.")]
    EmptyApiKey,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to write config: {0}")]
    ConfigWrite(String),
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Runs the interactive setup wizard and returns a ready-to-use `ProviderConfig`.
///
/// - `has_existing_config`: `true` when an existing config was found (i.e. this is
///   `ox setup` run explicitly). In that case the user is offered a "credentials only"
///   shortcut to avoid re-confirming their model on every key rotation.
pub async fn run_setup_wizard(
    has_existing_config: bool,
) -> Result<ProviderConfig, SetupError> {
    print_wizard_banner();

    // P2[S] — key-rotation UX: when re-running setup on an existing config, let the
    // user skip provider/model selection and only update a credential.
    let credentials_only = if has_existing_config {
        prompt_setup_mode()?
    } else {
        false
    };

    let (provider_type, model) = if credentials_only {
        // Load what's already configured so we can echo it back
        prompt_key_rotation_header()
    } else {
        // Full first-run flow: pick provider + model
        let pt = prompt_provider()?;
        let m = prompt_model(&pt)?;
        (pt, m)
    };

    // API key (masked, skipped for Ollama)
    let api_key = if provider_type != ProviderType::Ollama {
        Some(prompt_api_key(&provider_type)?)
    } else {
        None
    };

    // Config save location (only asked on full setup, not credentials-only)
    let config_path = if credentials_only {
        ConfigResolver::global_config_path_unchecked()
    } else {
        prompt_config_location()?
    };

    // P2[I] — persist via ConfigResolver, not inline in the wizard
    let inputs = WizardInputs {
        provider_type,
        model: model.clone(),
        api_key: api_key.clone(),
        config_path,
    };

    ConfigResolver::persist_wizard_inputs(&inputs)
        .map_err(|e| SetupError::ConfigWrite(e.to_string()))?;

    // Echo config path to terminal
    {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        println!("\n  Config saved to: {}", inputs.config_path.display());
        let _ = execute!(stdout, ResetColor);
    }

    // Inject key into current session env so the provider picks it up immediately
    if let (Some(ref key), Some(var)) = (&api_key, env_var_name(&provider_type)) {
        std::env::set_var(var, key);
    }

    // Build and return ProviderConfig
    let mut config = ProviderConfig::new(provider_type, model);
    if let Some(key) = api_key {
        config = config.with_api_key(key);
    }

    print_success();
    Ok(config)
}

// ---------------------------------------------------------------------------
// Prompts
// ---------------------------------------------------------------------------

/// Returns `true` if the user wants credentials-only mode (skip provider/model).
fn prompt_setup_mode() -> Result<bool, SetupError> {
    let mut stdout = io::stdout();
    print_section_header("What would you like to do?");
    println!("  1) Reconfigure everything  (provider, model, and API key)");
    println!("  2) Add or rotate an API key only  (keep current provider & model)");
    println!();

    loop {
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        print!("Enter number [1-2, default 1]: ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" | "" => return Ok(false),
            "2" => return Ok(true),
            _ => TerminalRenderer::print_error("Please enter 1 or 2."),
        }
    }
}

/// Used in credentials-only mode: asks which provider's key to update, returns
/// `(ProviderType, String::new())` — the model is intentionally empty so
/// `persist_wizard_inputs` only updates the credential, not `agent.model`.
///
/// Actually we still need a model for ProviderConfig; we read it from the existing
/// file or fall back to the provider default.
fn prompt_key_rotation_header() -> (ProviderType, String) {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
    println!();
    println!("  Which provider's key would you like to update?");
    println!("  1) Anthropic");
    println!("  2) OpenAI");
    println!("  3) Gemini");
    println!();
    let _ = execute!(stdout, ResetColor);

    loop {
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        print!("Enter number [1-3]: ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let provider = match input.trim() {
            "1" => ProviderType::Anthropic,
            "2" => ProviderType::OpenAi,
            "3" => ProviderType::Gemini,
            _ => {
                TerminalRenderer::print_error("Please enter 1, 2, or 3.");
                continue;
            }
        };
        // For credentials-only, we don't touch agent.model; pass default as placeholder
        return (provider, provider.default_model().to_string());
    }
    (ProviderType::Anthropic, ProviderType::Anthropic.default_model().to_string())
}

fn prompt_provider() -> Result<ProviderType, SetupError> {
    let mut stdout = io::stdout();

    print_section_header("Select a provider");
    println!("  1) Anthropic   (claude-3-5-sonnet, claude-3-7-sonnet, …)");
    println!("  2) OpenAI      (gpt-4o, o3-mini, …)");
    println!("  3) Gemini      (gemini-2.0-flash, gemini-2.5-pro, …)");
    println!("  4) Ollama      (llama3, mistral, qwen, …  — local, no key needed)");
    println!();

    loop {
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        print!("Enter number [1-4]: ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => return Ok(ProviderType::Anthropic),
            "2" => return Ok(ProviderType::OpenAi),
            "3" => return Ok(ProviderType::Gemini),
            "4" => return Ok(ProviderType::Ollama),
            _ => TerminalRenderer::print_error("Please enter a number between 1 and 4."),
        }
    }
}

fn prompt_model(provider: &ProviderType) -> Result<String, SetupError> {
    let default = provider.default_model();
    let mut stdout = io::stdout();

    print_section_header("Model name");
    let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
    println!("  Default: {}", default);
    let _ = execute!(stdout, ResetColor);
    println!();

    let _ = execute!(stdout, SetForegroundColor(Color::Green));
    print!("Model [press Enter for default]: ");
    let _ = execute!(stdout, ResetColor);
    let _ = stdout.flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_api_key(provider: &ProviderType) -> Result<String, SetupError> {
    let env_hint = env_var_name(provider).unwrap_or("API_KEY");
    let mut stdout = io::stdout();

    print_section_header("API key");
    let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
    println!("  Stored in plain text in your config file (same as GitHub CLI / AWS CLI).");
    println!("  You can also set the {} environment variable instead.", env_hint);
    let _ = execute!(stdout, ResetColor);
    println!();

    let _ = execute!(stdout, SetForegroundColor(Color::Green));
    print!("Paste your API key (input hidden): ");
    let _ = execute!(stdout, ResetColor);
    let _ = stdout.flush();

    let key = rpassword::read_password()?;
    if key.trim().is_empty() {
        return Err(SetupError::EmptyApiKey);
    }
    Ok(key.trim().to_string())
}

fn prompt_config_location() -> Result<PathBuf, SetupError> {
    let mut stdout = io::stdout();

    print_section_header("Save configuration");

    let global_path = ConfigResolver::global_config_path_unchecked();
    let local_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("ox.toml");

    println!("  1) Global  — {}", global_path.display());
    println!("  2) Local   — {}", local_path.display());
    println!();

    loop {
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        print!("Enter number [1-2, default 1]: ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" | "" => return Ok(global_path),
            "2" => return Ok(local_path),
            _ => TerminalRenderer::print_error("Please enter 1 or 2."),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn env_var_name(provider: &ProviderType) -> Option<&'static str> {
    match provider {
        ProviderType::Anthropic => Some("ANTHROPIC_API_KEY"),
        ProviderType::OpenAi | ProviderType::Custom => Some("OPENAI_API_KEY"),
        ProviderType::Gemini => Some("GEMINI_API_KEY"),
        ProviderType::Ollama => None,
    }
}

// ---------------------------------------------------------------------------
// Visual helpers
// ---------------------------------------------------------------------------

fn print_wizard_banner() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
    println!(
        r#"
   ____  _  __
  / __ \| |/ /   ox-orchestrator — First-Run Setup
 / /_/ /|   /    Let's get you configured in 60 seconds.
 \____//_/|_\
"#
    );
    let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
    println!(" Your settings will be saved to a config file and can be changed at any time.");
    println!(" You can re-run this wizard later with:  ox setup\n");
    let _ = execute!(stdout, ResetColor);
}

fn print_section_header(title: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
    println!("\n── {} ──", title);
    let _ = execute!(stdout, ResetColor);
}

fn print_success() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, SetForegroundColor(Color::Green));
    println!("\n✓ Setup complete! Starting ox...\n");
    let _ = execute!(stdout, ResetColor);
}
