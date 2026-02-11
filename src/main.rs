use std::{cell::LazyCell, collections::HashMap, os::unix::process::CommandExt as _, process};

use arborium::AnsiHighlighter;
use arborium::theme::builtin;
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Input};
use eyre::{Context, Result};
use regex::{Captures, Regex};
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use spinoff::{Color, Spinner, spinners};

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

            let mut spinner = Spinner::new(spinners::Dots, "Thinking ...", Color::Blue);
            let DoOutput {
                mut bash_command,
                short_explanation,
            } = run_do_command(&query)?;

            spinner.success("Done!");

            let mut hl = AnsiHighlighter::new(builtin::catppuccin_mocha());
            let highlighted_bash_command = hl.highlight("bash", &bash_command)?;

            println!("{short_explanation}\n{highlighted_bash_command}\n");
            let confirmation = Confirm::new().with_prompt("Execute").interact().unwrap();

            if confirmation {
                if let Some(template) = TemplateParameters::parse(&bash_command) {
                    let values = collect_parameter_values(template.parameters())?;
                    bash_command = template.apply_parameter_values(values);
                }

                // TODO: Would be neat to store previously executed
                // commands so that we can deterministically call them again
                process::Command::new("bash")
                    .arg("-c")
                    .arg(bash_command)
                    .spawn()?;
            }
        }
        SubCommand::Ask => todo!(),
    }

    Ok(())
}

#[derive(JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DoOutput {
    bash_command: String,
    short_explanation: String,
}

#[derive(Debug)]
struct TemplateParameters<'a> {
    bash_command: &'a str,
    captures: Vec<Captures<'a>>,
}

const PARAMETER_TEMPLATE: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"<([\w-]+)>").unwrap());

impl<'a> TemplateParameters<'a> {
    fn parse(bash_command: &'a str) -> Option<Self> {
        let captures: Vec<_> = PARAMETER_TEMPLATE.captures_iter(bash_command).collect();
        if captures.len() > 0 {
            Some(Self {
                bash_command,
                captures,
            })
        } else {
            None
        }
    }

    fn parameters(&self) -> Vec<&'a str> {
        self.captures
            .iter()
            .map(|capture| capture.get(1).unwrap().as_str())
            .collect()
    }

    fn apply_parameter_values(self, values: HashMap<&'a str, String>) -> String {
        let mut result = self.bash_command.to_string();
        for capture in self.captures.iter().rev() {
            let full_match = capture.get(0).unwrap();
            let param_name = capture.get(1).unwrap().as_str();
            if let Some(value) = values.get(param_name) {
                result.replace_range(full_match.range(), value);
            }
        }
        result
    }
}

fn collect_parameter_values<'a>(parameters: Vec<&'a str>) -> Result<HashMap<&'a str, String>> {
    let mut values = HashMap::new();
    for param in parameters {
        let value: String = Input::new()
            .with_prompt(format!("{}?", param))
            .interact_text()?;
        values.insert(param, value);
    }
    Ok(values)
}

fn run_do_command(query: &str) -> Result<DoOutput> {
    let process_output = build_do_command(query).output()?;
    // TODO: ask claude again if we fail to parse?
    let parsed_output: DoOutput =
        serde_json::from_slice(&process_output.stdout).wrap_err_with(|| {
            format!(
                r#"AI did not respond in conformance to the schema instead we got "{}""#,
                String::from_utf8_lossy(process_output.stdout.as_slice())
            )
        })?;

    Ok(parsed_output)
}

fn build_do_command(query: &str) -> process::Command {
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
