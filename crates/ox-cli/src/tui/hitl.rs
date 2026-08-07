use crossterm::execute;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use ox_core::types::ToolCall;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
    AlwaysApprove,
}

pub struct HitlPrompter;

impl HitlPrompter {
    /// Asks the human user interactively whether to approve or deny a mutating tool call.
    pub fn prompt_for_approval(call: &ToolCall, auto_approve_all: bool) -> ApprovalDecision {
        if auto_approve_all {
            return ApprovalDecision::Approved;
        }

        let mut stdout = io::stdout();

        let _ = execute!(stdout, SetForegroundColor(Color::Yellow));
        println!("\n[SECURITY AUDIT] Tool Execution Request:");
        let _ = execute!(stdout, SetForegroundColor(Color::Cyan));
        println!("  Tool: {}", call.name);
        let _ = execute!(stdout, SetForegroundColor(Color::DarkGrey));
        println!(
            "  Arguments: {}",
            serde_json::to_string_pretty(&call.arguments).unwrap_or_default()
        );
        let _ = execute!(stdout, SetForegroundColor(Color::Yellow));
        print!("Authorize execution? [y]es / [n]o / [a]lways approve: ");
        let _ = execute!(stdout, ResetColor);
        let _ = stdout.flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return ApprovalDecision::Denied;
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" | "" => ApprovalDecision::Approved,
            "a" | "all" | "always" => ApprovalDecision::AlwaysApprove,
            _ => ApprovalDecision::Denied,
        }
    }
}
