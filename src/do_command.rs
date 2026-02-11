use dialoguer::Confirm;
use eyre::Context;
use eyre::Result;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use spinoff::{Color, Spinner, spinners};
use std::cell::LazyCell;
use std::process;

use crate::{
    template_parameters::TemplateParameters,
    terminal_utils::{collect_form_inputs, highlight_bash},
};

#[derive(JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DoOutput {
    bash_command: String,
    comment: Option<String>,
}

pub fn run(query: String) -> Result<()> {
    let mut spinner = Spinner::new(spinners::Dots, "Thinking ...", Color::Blue);

    let DoOutput {
        mut bash_command,
        comment,
    } = run_claude(&query)?;

    spinner.success("Done!");

    // Print output
    if let Some(comment) = comment {
        println!("{comment}");
    }
    let highlighted_bash_command = highlight_bash(&bash_command)?;
    println!("{highlighted_bash_command}");

    let should_execute = Confirm::new().with_prompt("Execute").interact().unwrap();
    if !should_execute {
        // Nothing else to do
        return Ok(());
    }

    // Fill any template parameters if needed
    if let Some(template) = TemplateParameters::parse(&bash_command) {
        let values = collect_form_inputs(template.parameters())?;
        bash_command = template.apply_parameter_values(values);
        // Print the command with the template parameters replaced to make it easier to copy for later use
        println!("\n{bash_command}");
    }

    process::Command::new("bash")
        .arg("-c")
        .arg(bash_command)
        .status()?;

    Ok(())
}

fn run_claude(query: &str) -> Result<DoOutput> {
    let process_output = build_claude_command(query).output()?;
    // FIXME: ask claude again if we fail to parse?
    let output_str = String::from_utf8_lossy(&process_output.stdout).to_string();
    let stripped_output = strip_markdown_code_block(output_str);

    let parsed_output: DoOutput = serde_json::from_str(&stripped_output).wrap_err_with(|| {
        format!(
            r#"AI did not respond in conformance to the schema instead we got "{}""#,
            stripped_output
        )
    })?;

    Ok(parsed_output)
}

fn strip_markdown_code_block(s: String) -> String {
    const RE: LazyCell<regex::Regex> =
        LazyCell::new(|| regex::Regex::new(r"(?s)```[^\n]*\n(.*?)\n\s*```").unwrap());

    if let Some(captures) = RE.captures(&s) {
        captures.get(1).unwrap().as_str().trim().to_string()
    } else {
        s
    }
}

fn build_claude_command(query: &str) -> process::Command {
    let output_schema = schema_for!(DoOutput).to_value().to_string();
    let mut cmd = process::Command::new("claude");
    cmd.arg("--print");
    cmd.arg("--no-session-persistence");
    cmd.arg("--tools=Read,Grep");
    // FIXME: For some reason the claude CLI just crashes when I provide a schema,
    // for now, I'll just give the schema in the prompt.
    //cmd.arg(format!("--json-schema='{output_schema}'"));
    cmd.arg(format!(r#"
You are tasked with suggesting a one-off bash command/script.
Only respond in accordance to this JSON Schema, and wrap it in a markdown code block i.e. "```json".

If the information you where given is not sufficient you may use template parameters using angle brackets,
for example "git switch <branch-name>".

You don't have to leave a comment, and if you do. Only include relevant remarks, there is no
need to repeat the users query.

Schema
```
{output_schema}
```

Query:
{query}
"#));
    cmd
}

#[cfg(test)]
mod tests {
    use crate::do_command::DoOutput;

    #[test]
    fn parse_claude_output() {
        let s = r#"
            ```json
           {
             "bashCommand": "ls | awk '{print $0}'",
             "comment": "If you want all files including hidden ones, use: ls -a | awk '{print $0}'. For just filenames from ls -l, use: ls -l | awk 'NR>1 {print $NF}'"
           }
           ```
    "#.to_string();

        let stripped_output = super::strip_markdown_code_block(s);
        let _: DoOutput = serde_json::from_str(&stripped_output).unwrap();
    }
}
