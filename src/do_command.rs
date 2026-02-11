use dialoguer::Confirm;
use eyre::Context;
use eyre::Result;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use spinoff::{Color, Spinner, spinners};
use std::process;

use crate::{
    template_parameters::TemplateParameters,
    terminal_utils::{collect_form_inputs, highlight_bash},
};

#[derive(JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DoOutput {
    bash_command: String,
    short_explanation: String,
}

pub fn run(query: String) -> Result<()> {
    let mut spinner = Spinner::new(spinners::Dots, "Thinking ...", Color::Blue);

    let DoOutput {
        mut bash_command,
        short_explanation,
    } = run_claude(&query)?;

    spinner.success("Done!");

    let highlighted_bash_command = highlight_bash(&bash_command)?;
    println!("{short_explanation}\n{highlighted_bash_command}\n");

    let should_execute = Confirm::new().with_prompt("Execute").interact().unwrap();

    if !should_execute {
        // Nothing else to do
        return Ok(());
    }

    // Fill any template parameters if needed
    if let Some(template) = TemplateParameters::parse(&bash_command) {
        let values = collect_form_inputs(template.parameters())?;
        bash_command = template.apply_parameter_values(values);
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
    // FIXME: handle when AI escapes code in ```
    let parsed_output: DoOutput =
        serde_json::from_slice(&process_output.stdout).wrap_err_with(|| {
            format!(
                r#"AI did not respond in conformance to the schema instead we got "{}""#,
                String::from_utf8_lossy(process_output.stdout.as_slice())
            )
        })?;

    Ok(parsed_output)
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
Only respond in accordance to this JSON Schema, and don't wrap it in anything else (don't wrap it in ``` for example).
If the information you where given is not sufficient you may use template parameters using angle brackets,
for example "git switch <branch-name>".

Schema
{output_schema}

Query:
{query}
"#));
    cmd
}
