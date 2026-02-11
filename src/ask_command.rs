use std::process;

use eyre::Result;

use crate::terminal_utils::{highlight_markdown, thinking_spinner};

pub fn run(query: &str) -> Result<()> {
    let mut spinner = thinking_spinner();
    let process_output = build_claude_command(query).output()?;
    spinner.success("Done!");
    let formatted_response =
        highlight_markdown(&String::from_utf8_lossy(process_output.stdout.as_slice()))?;
    println!("{formatted_response}");
    Ok(())
}

fn build_claude_command(query: &str) -> process::Command {
    let mut cmd = process::Command::new("claude");
    cmd.arg("--print");
    cmd.arg("--no-session-persistence");
    cmd.arg("--tools=Read,Grep");
    cmd.arg(format!(r#"
You are tasked with responding to a user query. Answer succinctly. If your answer is longer than 1-2 sentences,
use Markdown formatting. Especially for code blocks.

Query:
{query}
"#));
    cmd
}
