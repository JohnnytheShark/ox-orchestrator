use crossterm::execute;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ox_core::session::{NodeId, SessionTree};
use ox_core::types::{TokenUsage, ToolCall, ToolResult};
use std::io::{self, Write};
use std::path::Path;

pub struct TerminalRenderer;

impl TerminalRenderer {
    pub fn render_banner(model: &str, provider: &str, session_id: &str, workspace: &Path) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
        println!(
            r#"
   ____  _  __
  / __ \| |/ /   ox-orchestrator v0.1.0
 / /_/ /|   /    Minimalist & Secure AI Agent Harness
 \____//_/|_\    
"#
        );
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        println!(" Provider  : {} ({})", provider, model);
        println!(" Workspace : {}", workspace.display());
        println!(" Session   : {}", session_id);
        println!(" Security  : Path-Jailed, Env-Scrubbed, Zeroized Secrets");
        println!(" Commands  : /help, /cost, /diff, /undo, /tree, /history, /checkout, /auto, /save, /exit\n");
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_user_prompt() {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        print!("\nuser > ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();
    }

    pub fn print_assistant_prefix() {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Magenta));
        print!("\nox > ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();
    }

    pub fn print_text_delta(text: &str) {
        let mut stdout = io::stdout();
        print!("{}", text);
        let _ = stdout.flush();
    }

    pub fn print_thinking_delta(thinking: &str) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        print!("{}", thinking);
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();
    }

    pub fn print_tool_start(call: &ToolCall) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Blue));
        println!("\n-> [tool:call] {} (args: {})", call.name, call.arguments);
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_tool_result(res: &ToolResult) {
        let mut stdout = io::stdout();
        if res.is_error {
            let _ = execute!(stdout, SetForegroundColor(Color::Red));
            println!("<- [tool:error] {}\n{}", res.tool_name, res.content);
        } else {
            let _ = execute!(stdout, SetForegroundColor(Color::DarkCyan));
            let preview = if res.content.len() > 300 {
                format!("{}... ({} bytes)", &res.content[..200], res.content.len())
            } else {
                res.content.clone()
            };
            println!("<- [tool:ok] {}\n{}", res.tool_name, preview);
        }
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_turn_summary(node_id: &NodeId, usage: Option<&TokenUsage>) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        if let Some(u) = usage {
            println!(
                "\n-- [turn: {}] in: {} tokens | out: {} tokens --",
                node_id.short(),
                u.input_tokens,
                u.output_tokens
            );
        } else {
            println!("\n-- [turn: {}] --", node_id.short());
        }
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_error(msg: &str) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Red));
        eprintln!("\n[ERROR] {}", msg);
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_ascii_dag(tree: &SessionTree) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Yellow));
        println!(
            "\nSession DAG [id: {} | total nodes: {}]:",
            tree.id,
            tree.nodes.len()
        );

        let active_set: std::collections::HashSet<_> =
            tree.active_path().iter().map(|n| &n.id).collect();

        for (id, node) in &tree.nodes {
            let is_current = tree.current_leaf_id.as_ref() == Some(id);
            let is_active = active_set.contains(id);

            let marker = if is_current {
                "* (ACTIVE LEAF)"
            } else if is_active {
                "|"
            } else {
                " "
            };

            let parent_str = match &node.parent_id {
                Some(p) => format!("<- parent: {}", p.short()),
                None => "(ROOT)".to_string(),
            };

            let role_str = match node.message.role {
                ox_core::types::Role::User => "USER",
                ox_core::types::Role::Assistant => "ASSISTANT",
                ox_core::types::Role::System => "SYSTEM",
                ox_core::types::Role::Tool => "TOOL",
            };

            let preview = node.message.text_content();
            let truncated = if preview.len() > 40 {
                format!("{}...", &preview[..37])
            } else {
                preview
            };

            println!(
                "  {} [{}] {} [{}] \"{}\"",
                marker,
                id.short(),
                parent_str,
                role_str,
                truncated.replace('\n', " ")
            );
        }
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_cost_summary(model: &str, turns: usize, usage: &TokenUsage, cost_usd: f64) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
        println!("\n+------------------------------------------------------+");
        println!("|               Session Token & Cost Summary           |");
        println!("+------------------------------------------------------+");
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        println!("  Model           : {}", model);
        println!("  Total Turns     : {}", turns);
        println!("  Input Tokens    : {}", usage.input_tokens);
        println!("  Output Tokens   : {}", usage.output_tokens);
        println!("  Total Tokens    : {}", usage.total_tokens());
        if let Some(cache_read) = usage.cache_read_tokens {
            println!("  Cache Read Tokens : {}", cache_read);
        }
        let _ = execute!(stdout, SetForegroundColor(Color::Green));
        println!("  Estimated Cost  : ${:.4} USD", cost_usd);
        let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
        println!("+------------------------------------------------------+\n");
        let _ = execute!(stdout, ResetColor);
    }

    pub fn print_diff(diff_output: &str) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, SetForegroundColor(Color::Yellow));
        println!("\n--- Working Tree Git Diff ---");
        for line in diff_output.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                let _ = execute!(stdout, SetForegroundColor(Color::Green));
                println!("{}", line);
            } else if line.starts_with('-') && !line.starts_with("---") {
                let _ = execute!(stdout, SetForegroundColor(Color::Red));
                println!("{}", line);
            } else if line.starts_with("@@") {
                let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
                println!("{}", line);
            } else if line.starts_with("diff --git")
                || line.starts_with("index ")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
            {
                let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
                println!("{}", line);
            } else {
                let _ = execute!(stdout, ResetColor);
                println!("{}", line);
            }
        }
        let _ = execute!(stdout, SetForegroundColor(Color::Yellow));
        println!("-----------------------------\n");
        let _ = execute!(stdout, ResetColor);
    }
}
