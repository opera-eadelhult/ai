use std::{os::unix::process::CommandExt as _, process};

use clap::{Parser, Subcommand};
use dialoguer::Input;
use eyre::Result;

mod do_command;
mod template_parameters;
mod terminal_utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Spawn an AI agent in a separate git worktree
    Agent,
    /// Suggest a one-off bash command.
    Do { query: Option<String> },
    /// Ask a question
    Ask,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        SubCommand::Agent => {
            let err = process::Command::new("claude").exec();
            return Err(err.into());
        }
        SubCommand::Do { query } => {
            let query = if let Some(query) = query {
                query
            } else {
                Input::new().with_prompt("Query").interact_text()?
            };
            do_command::run(query)?
        }
        SubCommand::Ask => todo!(),
    }

    Ok(())
}
