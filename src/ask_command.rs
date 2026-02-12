use std::process;

use eyre::Result;

use crate::terminal_utils::thinking_spinner;

pub fn run(query: &str, model: Option<&str>) -> Result<()> {
    let mut spinner = thinking_spinner();
    let process_output = build_claude_command(query, model).output()?;
    spinner.success("Done!");
    println!(
        "{}",
        String::from_utf8_lossy(process_output.stdout.as_slice())
    );
    Ok(())
}

fn build_claude_command(query: &str, model: Option<&str>) -> process::Command {
    let mut cmd = process::Command::new("claude");
    cmd.arg("--print");
    cmd.arg("--no-session-persistence");
    cmd.arg("--tools=Read,Grep,Glob,WebFetch,WebSearch");
    cmd.arg("--allowedTools=Read,Grep,Glob,WebFetch,WebSearch");
    if let Some(model) = model {
        cmd.arg(format!("--model={}", model));
    }
    cmd.arg(format!(r#"
You are tasked with responding to a user query. Answer succinctly. If your answer is longer than 1-2 sentences,
use Markdown formatting. Especially for code blocks.

Query:
{query}
"#));
    cmd
}
