use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};
use dialoguer::Input;
use eyre::Result;
use rand::RngExt;

mod agent_command;
mod ask_command;
mod do_command;
mod template_parameters;
mod terminal_utils;

#[derive(Parser)]
#[command(
    name = "AI",
    version,
    author,
    about = "An opinionated wrapper around Claude Code to manage worktrees and execute scripts",
    long_about = None
)]
struct Args {
    /// Model to use for Claude (e.g. sonnet, opus, haiku)
    #[arg(short, long, global = true, env = "AI_MODEL")]
    model: Option<String>,
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Spawn an AI agent in a separate git worktree
    Agent {
        /// Name of git worktree & branch
        #[arg(short, long)]
        name: Option<String>,
        /// Command that runs when worktree has been created (e.g. `npm install`)
        #[arg(short, long, env = "AI_SETUP_COMMAND")]
        setup_command: Option<String>,
        /// Parent directory where all worktrees are stored. If omitted, a temporary directory will be created
        #[arg(short, long, env = "AI_WORKTREES_DIR")]
        worktrees_dir: Option<PathBuf>,
        /// Set to true if you want the worktree to be kept even after the agent process exits
        #[arg(short, long, action = ArgAction::SetTrue)]
        keep_worktree: Option<bool>,
        /// Open $VISUAL in the worktree directory (e.g. "zed --wait")
        #[arg(short, long, action = ArgAction::SetTrue)]
        editor: bool,
        query: Option<String>,
    },
    /// Suggest a one-off bash command
    Do { query: Option<String> },
    /// Ask a question
    Ask { query: Option<String> },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Agent {
            name,
            query,
            setup_command,
            worktrees_dir,
            keep_worktree,
            editor,
        } => {
            let feature_name = match (name, &query) {
                (Some(name), _) => name,
                (None, Some(query)) => {
                    // Use the query to generate a feature name if we where given one
                    let mut generate_named = query
                        .chars()
                        .take(20)
                        .map(|c| if c.is_ascii_whitespace() { '-' } else { c })
                        .collect::<String>();
                    generate_named.push('-');
                    generate_named.push_str(&rand::rng().random_range(100..1000).to_string());
                    generate_named
                }
                // Otherwise pick a random name with common nouns
                (None, None) => names::Generator::default().next().unwrap(),
            };

            // Determine the worktree path
            let worktree_path = if let Some(dir) = worktrees_dir {
                std::fs::create_dir_all(&dir)?;
                dir.join(&feature_name)
            } else {
                // Use a temporary directory namespaced by current directory
                let cwd = std::env::current_dir()?;
                let cwd_name = cwd
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                std::env::temp_dir()
                    .join("ai-worktrees")
                    .join(cwd_name)
                    .join(&feature_name)
            };

            agent_command::run(
                query,
                feature_name,
                setup_command,
                worktree_path,
                keep_worktree.unwrap_or(false),
                args.model,
                editor,
            )?;
        }
        SubCommand::Do { query } => {
            let query = if let Some(query) = query {
                query
            } else {
                Input::new().with_prompt("Query").interact_text()?
            };
            do_command::run(&query, args.model.as_deref())?
        }
        SubCommand::Ask { query } => {
            let query = if let Some(query) = query {
                query
            } else {
                Input::new().with_prompt("Query").interact_text()?
            };
            ask_command::run(&query, args.model.as_deref())?
        }
    }

    Ok(())
}
