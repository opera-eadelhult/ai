use eyre::Result;
use std::{os::unix::process::CommandExt as _, process};

pub fn run(query: Option<String>, feature_name: String) -> Result<()> {
    // TODO: create git worktree (and branch)

    let mut claude_command = {
        // TODO: don't spawn claude, instead spawn a shell script to add a trap!
        let mut command = process::Command::new("claude");
        if let Some(query) = query {
            command.arg(query);
        }
        command
    };

    let err = claude_command.exec();
    // Note: if "exec" works, the Rust program will *never* continue beyond this point.
    return Err(err.into());
}
