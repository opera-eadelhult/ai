use std::{os::unix::process::CommandExt as _, process};

use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Input};
use eyre::Result;
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

            let mut spinner = Spinner::new(spinners::Dots, "Working ...", Color::Blue);
            let DoOutput {
                bash_command,
                short_explanation,
            } = run_do_command(&query)?;

            spinner.success("Done!");

            let confirmation = Confirm::new()
                .with_prompt(format!("{short_explanation}\n{bash_command}\n"))
                .interact()
                .unwrap();

            if confirmation {
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

fn run_do_command(query: &str) -> Result<DoOutput> {
    let process_output = build_do_command(query).output()?;
    // TODO: ask claude again if we fail to parse?
    let parsed_output: DoOutput = serde_json::from_slice(&process_output.stdout)?;
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
You are tasked with suggesting a one-off bash command/script. Only respond in accordance to this JSON Schema, and nothing else (don't wrap it in ``` for example):

Schema
{output_schema}

Query:
{query}
"#));
    cmd
}
