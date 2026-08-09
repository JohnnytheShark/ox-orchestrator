use crate::tui::TerminalRenderer;
use ox_providers::{ProviderConfig, ProviderType};
use ox_tools::mcp::{McpClient, McpToolAdapter};
use ox_tools::ToolDispatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// API credentials stored in the config file, one per provider.
/// Env vars always take priority; these serve as a fallback and are written by the
/// setup wizard so users can switch providers without losing other keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CredentialsConfig {
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
}

impl CredentialsConfig {
    /// Merges self with a fallback, preserving any key already set in self.
    pub fn merge_with(&mut self, fallback: CredentialsConfig) {
        if self.anthropic_api_key.is_none() {
            self.anthropic_api_key = fallback.anthropic_api_key;
        }
        if self.openai_api_key.is_none() {
            self.openai_api_key = fallback.openai_api_key;
        }
        if self.gemini_api_key.is_none() {
            self.gemini_api_key = fallback.gemini_api_key;
        }
    }

    /// Returns the stored key for the given provider type, if any.
    pub fn get_key_for(&self, provider: &ox_providers::ProviderType) -> Option<&str> {
        match provider {
            ox_providers::ProviderType::Anthropic => self.anthropic_api_key.as_deref(),
            ox_providers::ProviderType::OpenAi | ox_providers::ProviderType::Custom => {
                self.openai_api_key.as_deref()
            }
            ox_providers::ProviderType::Gemini => self.gemini_api_key.as_deref(),
            ox_providers::ProviderType::Ollama => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OxConfigFile {
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub credentials: CredentialsConfig,

    // Backward-compatibility flat fields
    pub default_model: Option<String>,
    pub default_provider: Option<String>,
    pub base_url: Option<String>,
    pub auto_approve: Option<bool>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub max_turns: Option<usize>,
    pub auto_approve: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Collected inputs from the interactive setup wizard.
/// Separates the "gather" phase (TUI prompts) from the "persist" phase (file I/O),
/// allowing each to be tested and evolved independently.
#[derive(Debug, Clone)]
pub struct WizardInputs {
    pub provider_type: ProviderType,
    pub model: String,
    /// `None` for Ollama (no key required).
    pub api_key: Option<String>,
    pub config_path: PathBuf,
}

impl OxConfigFile {
    /// Merges self with a lower-priority fallback configuration.
    pub fn merge_with(&mut self, fallback: OxConfigFile) {
        if self.agent.model.is_none() {
            self.agent.model = fallback.agent.model.or(fallback.default_model);
        }
        if self.agent.provider.is_none() {
            self.agent.provider = fallback.agent.provider.or(fallback.default_provider);
        }
        if self.agent.base_url.is_none() {
            self.agent.base_url = fallback.agent.base_url.or(fallback.base_url);
        }
        if self.agent.max_turns.is_none() {
            self.agent.max_turns = fallback.agent.max_turns;
        }
        if self.agent.auto_approve.is_none() {
            self.agent.auto_approve = fallback.agent.auto_approve.or(fallback.auto_approve);
        }

        // Merge credentials — preserve any key already set in self
        self.credentials.merge_with(fallback.credentials);

        for (k, v) in fallback.mcp.servers {
            self.mcp.servers.entry(k).or_insert(v);
        }
        for (k, v) in fallback.mcp_servers {
            self.mcp.servers.entry(k).or_insert(v);
        }
    }

    pub fn get_model(&self) -> Option<&str> {
        self.agent
            .model
            .as_deref()
            .or(self.default_model.as_deref())
    }

    pub fn get_provider(&self) -> Option<&str> {
        self.agent
            .provider
            .as_deref()
            .or(self.default_provider.as_deref())
    }

    pub fn get_base_url(&self) -> Option<&str> {
        self.agent.base_url.as_deref().or(self.base_url.as_deref())
    }

    pub fn get_max_turns(&self) -> Option<usize> {
        self.agent.max_turns
    }

    pub fn get_auto_approve(&self) -> Option<bool> {
        self.agent.auto_approve.or(self.auto_approve)
    }

    pub fn all_mcp_servers(&self) -> HashMap<String, McpServerConfig> {
        let mut map = self.mcp.servers.clone();
        for (k, v) in &self.mcp_servers {
            map.entry(k.clone()).or_insert_with(|| v.clone());
        }
        map
    }
}

pub struct ConfigResolver;

impl ConfigResolver {
    /// Discovers workspace root directory by locating `.git`, `ox.toml`, or `.ox`.
    pub fn find_workspace_root(override_path: Option<PathBuf>) -> PathBuf {
        if let Some(p) = override_path {
            return p;
        }

        let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut dir = current.as_path();

        loop {
            if dir.join(".git").exists() || dir.join("ox.toml").exists() || dir.join(".ox").exists()
            {
                return dir.to_path_buf();
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }

        current
    }

    /// Returns the canonical global config path without checking if it exists.
    /// Priority: XDG_CONFIG_HOME → HOME → USERPROFILE → APPDATA → fallback relative.
    /// Single source of truth used by both `global_config_path` and the setup wizard.
    pub fn global_config_path_unchecked() -> PathBuf {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return PathBuf::from(xdg).join("ox").join("config.toml");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".config")
                .join("ox")
                .join("config.toml");
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            return PathBuf::from(userprofile)
                .join(".config")
                .join("ox")
                .join("config.toml");
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("ox").join("config.toml");
        }
        PathBuf::from(".config").join("ox").join("config.toml")
    }

    /// Locates standard user global configuration path (`~/.config/ox/config.toml`).
    /// Returns `Some` only if the file already exists.
    pub fn global_config_path() -> Option<PathBuf> {
        let p = Self::global_config_path_unchecked();
        p.exists().then_some(p)
    }

    /// Parses a TOML or JSON config file based on file extension.
    pub fn load_file_config(path: &Path) -> Option<OxConfigFile> {
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        if path.extension().is_some_and(|ext| ext == "toml") {
            toml::from_str::<OxConfigFile>(&content).ok()
        } else {
            serde_json::from_str::<OxConfigFile>(&content).ok()
        }
    }

    /// Loads workspace config (`ox.toml` preferred, falling back to `.ox/config.json`).
    pub fn load_workspace_config(root: &Path) -> Option<OxConfigFile> {
        let toml_path = root.join("ox.toml");
        if toml_path.exists() {
            if let Some(cfg) = Self::load_file_config(&toml_path) {
                return Some(cfg);
            }
        }

        let json_path = root.join(".ox").join("config.json");
        if json_path.exists() {
            if let Some(cfg) = Self::load_file_config(&json_path) {
                return Some(cfg);
            }
        }

        None
    }

    /// Hierarchically resolves configuration: Global -> Workspace (overrides Global).
    pub fn load_hierarchical_config(workspace_root: &Path) -> OxConfigFile {
        let mut resolved = OxConfigFile::default();

        if let Some(global_path) = Self::global_config_path() {
            if let Some(global_cfg) = Self::load_file_config(&global_path) {
                resolved.merge_with(global_cfg);
            }
        }

        if let Some(workspace_cfg) = Self::load_workspace_config(workspace_root) {
            let mut ws = workspace_cfg;
            ws.merge_with(resolved);
            resolved = ws;
        }

        resolved
    }

    /// Builds a ProviderConfig by combining CLI flags (priority 1) with configuration
    /// file (priority 2). Returns `None` for the model when no configuration is found,
    /// so callers can detect a first-run state and trigger the setup wizard.
    pub fn resolve_provider_config(
        cli_model: Option<&str>,
        cli_provider: Option<&str>,
        cli_base_url: Option<&str>,
        file_config: &OxConfigFile,
    ) -> ProviderConfig {
        // No hard-coded default — if nothing is configured, model will be an empty
        // string, which signals main.rs to run the setup wizard.
        let model = cli_model
            .map(|s| s.to_string())
            .or_else(|| file_config.get_model().map(|s| s.to_string()))
            .unwrap_or_default();

        let provider_type = if let Some(p_str) = cli_provider {
            ProviderType::from_str_name(p_str)
        } else if let Some(p_str) = file_config.get_provider() {
            ProviderType::from_str_name(p_str)
        } else {
            ProviderType::infer_from_model_name(&model)
        };

        let mut config = ProviderConfig::new(provider_type, model);

        // Fall back to credentials stored in config file if env var wasn't set
        if config.get_api_key().is_none() {
            if let Some(key) = file_config.credentials.get_key_for(&config.provider_type) {
                config = config.with_api_key(key);
            }
        }

        if let Some(url) = cli_base_url {
            config = config.with_base_url(url);
        } else if let Some(url) = file_config.get_base_url() {
            config = config.with_base_url(url);
        }

        config
    }

    /// Writes or merges wizard inputs into a TOML config file on disk.
    /// Existing credentials for other providers are preserved.
    pub(crate) fn persist_wizard_inputs(
        inputs: &WizardInputs,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = &inputs.config_path;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Load existing config so we can merge rather than overwrite
        let mut existing: OxConfigFile = if path.exists() {
            Self::load_file_config(path).unwrap_or_default()
        } else {
            OxConfigFile::default()
        };

        // Update agent section
        existing.agent.provider = Some(inputs.provider_type.to_str_name().to_string());
        existing.agent.model = Some(inputs.model.clone());

        // Merge credential — only write the key for the chosen provider; keep others intact
        if let Some(ref key) = inputs.api_key {
            match inputs.provider_type {
                ProviderType::Anthropic => {
                    existing.credentials.anthropic_api_key = Some(key.clone())
                }
                ProviderType::OpenAi | ProviderType::Custom => {
                    existing.credentials.openai_api_key = Some(key.clone())
                }
                ProviderType::Gemini => existing.credentials.gemini_api_key = Some(key.clone()),
                ProviderType::Ollama => {}
            }
        }

        let toml_str = toml::to_string_pretty(&existing)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    /// Auto-registers all configured MCP servers into the tool dispatcher.
    pub async fn register_mcp_servers(
        dispatcher: &mut ToolDispatcher,
        servers: &HashMap<String, McpServerConfig>,
    ) {
        for (srv_name, mcp_cfg) in servers {
            match McpClient::launch_stdio(
                srv_name,
                &mcp_cfg.command,
                &mcp_cfg.args,
                mcp_cfg.env.clone(),
            )
            .await
            {
                Ok(client) => {
                    let client_arc = Arc::new(client);
                    if let Ok(tools_list) = client_arc.list_tools().await {
                        for tool_info in tools_list.tools {
                            let adapter =
                                McpToolAdapter::new(srv_name, tool_info, client_arc.clone());
                            dispatcher.register(Arc::new(adapter));
                        }
                    }
                }
                Err(e) => {
                    TerminalRenderer::print_error(&format!(
                        "Failed to connect to MCP server '{}': {}",
                        srv_name, e
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_ox_toml() {
        let toml_str = r#"
[agent]
provider = "openai"
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
max_turns = 30
auto_approve = true

[mcp.servers.sqlite]
command = "mcp-server-sqlite"
args = ["--db-path", "./data.db"]
"#;
        let cfg: OxConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.get_provider(), Some("openai"));
        assert_eq!(cfg.get_model(), Some("gpt-4o"));
        assert_eq!(cfg.get_base_url(), Some("https://api.openai.com/v1"));
        assert_eq!(cfg.get_max_turns(), Some(30));
        assert_eq!(cfg.get_auto_approve(), Some(true));

        let servers = cfg.all_mcp_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers["sqlite"].command, "mcp-server-sqlite");
        assert_eq!(servers["sqlite"].args, vec!["--db-path", "./data.db"]);
    }

    #[test]
    fn test_hierarchical_merge() {
        let mut global_cfg = OxConfigFile::default();
        global_cfg.agent.model = Some("claude-3-5-haiku".to_string());
        global_cfg.agent.max_turns = Some(10);
        global_cfg.agent.auto_approve = Some(false);

        let mut ws_cfg = OxConfigFile::default();
        ws_cfg.agent.model = Some("claude-3-7-sonnet-20250219".to_string());
        // ws_cfg does not specify max_turns, so it should inherit from global

        let mut merged = ws_cfg;
        merged.merge_with(global_cfg);

        assert_eq!(merged.get_model(), Some("claude-3-7-sonnet-20250219"));
        assert_eq!(merged.get_max_turns(), Some(10));
        assert_eq!(merged.get_auto_approve(), Some(false));
    }

    #[test]
    fn test_cli_precedence() {
        let mut cfg = OxConfigFile::default();
        cfg.agent.model = Some("config-model".to_string());
        cfg.agent.provider = Some("anthropic".to_string());

        let prov =
            ConfigResolver::resolve_provider_config(Some("cli-override-model"), None, None, &cfg);
        assert_eq!(prov.model, "cli-override-model");
    }

    #[test]
    fn test_load_workspace_ox_toml() {
        let dir = tempdir().unwrap();
        let ox_toml = dir.path().join("ox.toml");
        std::fs::write(
            &ox_toml,
            r#"
[agent]
model = "gemini-2.0-flash"
provider = "gemini"
"#,
        )
        .unwrap();

        let cfg = ConfigResolver::load_workspace_config(dir.path()).unwrap();
        assert_eq!(cfg.get_model(), Some("gemini-2.0-flash"));
        assert_eq!(cfg.get_provider(), Some("gemini"));
    }

    #[test]
    fn test_credentials_merge_preserves_existing_key() {
        let mut primary = CredentialsConfig {
            anthropic_api_key: Some("existing-ant".to_string()),
            openai_api_key: None,
            gemini_api_key: None,
        };
        let fallback = CredentialsConfig {
            anthropic_api_key: Some("should-not-overwrite".to_string()),
            openai_api_key: Some("new-openai-key".to_string()),
            gemini_api_key: None,
        };
        primary.merge_with(fallback);
        assert_eq!(primary.anthropic_api_key.as_deref(), Some("existing-ant"));
        assert_eq!(primary.openai_api_key.as_deref(), Some("new-openai-key"));
        assert!(primary.gemini_api_key.is_none());
    }

    #[test]
    fn test_credentials_get_key_for() {
        let creds = CredentialsConfig {
            anthropic_api_key: Some("ant-key".to_string()),
            openai_api_key: Some("oai-key".to_string()),
            gemini_api_key: None,
        };
        assert_eq!(creds.get_key_for(&ProviderType::Anthropic), Some("ant-key"));
        assert_eq!(creds.get_key_for(&ProviderType::OpenAi), Some("oai-key"));
        assert_eq!(creds.get_key_for(&ProviderType::Custom), Some("oai-key"));
        assert_eq!(creds.get_key_for(&ProviderType::Gemini), None);
        assert_eq!(creds.get_key_for(&ProviderType::Ollama), None);
    }

    #[test]
    fn test_persist_wizard_inputs_merges_not_overwrites() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ox.toml");

        // Seed an existing config that already has an Anthropic key
        let mut initial = OxConfigFile::default();
        initial.credentials.anthropic_api_key = Some("ant-key-original".to_string());
        std::fs::write(&path, toml::to_string_pretty(&initial).unwrap()).unwrap();

        // Wizard adds an OpenAI key — must not touch the existing Anthropic key
        let inputs = WizardInputs {
            provider_type: ProviderType::OpenAi,
            model: "gpt-4o".to_string(),
            api_key: Some("oai-key-new".to_string()),
            config_path: path.clone(),
        };
        ConfigResolver::persist_wizard_inputs(&inputs).unwrap();

        let result = ConfigResolver::load_file_config(&path).unwrap();
        assert_eq!(
            result.credentials.anthropic_api_key.as_deref(),
            Some("ant-key-original"),
            "Anthropic key must not be overwritten"
        );
        assert_eq!(
            result.credentials.openai_api_key.as_deref(),
            Some("oai-key-new"),
            "OpenAI key must be written"
        );
        assert_eq!(result.agent.model.as_deref(), Some("gpt-4o"));
        assert_eq!(result.agent.provider.as_deref(), Some("openai"));
    }
}
