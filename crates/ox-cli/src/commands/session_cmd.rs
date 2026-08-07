use crate::cli_args::SessionCommands;
use crate::tui::TerminalRenderer;
use ox_core::session::SessionStorage;
use std::fs;
use std::path::{Path, PathBuf};

pub fn handle_session_command(
    cmd: SessionCommands,
    workspace_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sessions_dir = workspace_root.join(".ox").join("sessions");
    let storage = SessionStorage::new(&sessions_dir)?;

    match cmd {
        SessionCommands::List => {
            let sessions = storage.list_sessions()?;
            if sessions.is_empty() {
                println!("No saved sessions found in {}", sessions_dir.display());
            } else {
                println!(
                    "Saved Sessions ({}) in {}:",
                    sessions.len(),
                    sessions_dir.display()
                );
                for s in sessions {
                    println!("  - {}", s);
                }
            }
        }
        SessionCommands::Tree { session_id } => {
            let tree = storage.load(&session_id)?;
            TerminalRenderer::print_ascii_dag(&tree);
        }
        SessionCommands::Export { session_id, output } => {
            let tree = storage.load(&session_id)?;
            let history = tree.linear_history();

            let mut md = format!("# Session Export: {}\n\n", tree.id);
            for msg in history {
                md.push_str(&format!("### {:?}\n\n{}\n\n", msg.role, msg.text_content()));
            }

            let out_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.md", session_id)));
            fs::write(&out_path, md)?;
            println!("Exported session to {}", out_path.display());
        }
    }

    Ok(())
}
