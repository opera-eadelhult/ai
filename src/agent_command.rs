use eyre::{Result, eyre};
use std::process::Command;
use std::{os::unix::process::CommandExt as _, path::PathBuf};

pub fn run(
    query: Option<String>,
    feature_name: String,
    setup_command: Option<String>,
    worktree_path: PathBuf,
    keep_worktree: bool,
) -> Result<()> {
    // Capture uncommitted changes from the main directory
    let diff_output = Command::new("git")
        .args(["diff"])
        .output()?;

    let uncommitted_changes = String::from_utf8_lossy(&diff_output.stdout).to_string();

    // Create git worktree and branch
    let branch_name = format!("ai/{}", feature_name);
    let status = Command::new("git")
        .args(["worktree", "add", "-b", &branch_name])
        .arg(&worktree_path)
        .status()?;

    if !status.success() {
        return Err(eyre!("Failed to create git worktree"));
    }

    // Apply uncommitted changes to the new worktree if there are any
    if !uncommitted_changes.is_empty() {
        let apply_status = Command::new("git")
            .args(["apply"])
            .current_dir(&worktree_path)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(uncommitted_changes.as_bytes())?;
                }
                child.wait()
            })?;

        if !apply_status.success() {
            return Err(eyre!("Failed to apply uncommitted changes to worktree"));
        }
    }

    // Run setup command if provided
    if let Some(setup_cmd) = setup_command {
        let status = Command::new("sh")
            .arg("-c")
            .arg(&setup_cmd)
            .current_dir(&worktree_path)
            .status()?;

        if !status.success() {
            return Err(eyre!("Setup command failed: {}", setup_cmd));
        }
    }

    // Create a shell script that sets up a trap to cleanup the worktree
    let cleanup_script = if keep_worktree {
        // No cleanup needed
        format!(
            r#"#!/bin/sh
cd "{}"
claude {}
"#,
            worktree_path.display(),
            query.as_deref().unwrap_or("")
        )
    } else {
        // Add trap to remove worktree on exit
        format!(
            r#"#!/bin/sh
cleanup() {{
    cd -
    git worktree remove "{}" --force
    echo "Removed worktree: {}"
}}
trap cleanup EXIT
cd "{}"
claude {}
"#,
            worktree_path.display(),
            worktree_path.display(),
            worktree_path.display(),
            query.as_deref().unwrap_or("")
        )
    };

    // Execute the shell script
    let err = Command::new("sh").arg("-c").arg(&cleanup_script).exec();
    // Note: if "exec" works, the Rust program will *never* continue beyond this point.
    Err(err.into())
}
