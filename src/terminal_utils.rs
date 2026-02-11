use std::{collections::HashMap, fmt::Display};

use arborium::{AnsiHighlighter, theme::builtin};
use dialoguer::Input;
use eyre::Result;
use spinoff::{Color, Spinner, spinners};

pub fn highlight_bash(command: &str) -> Result<String> {
    let mut hl = AnsiHighlighter::new(builtin::catppuccin_mocha());
    Ok(hl.highlight("bash", &command)?)
}

pub fn highlight_markdown(command: &str) -> Result<String> {
    let mut hl = AnsiHighlighter::new(builtin::catppuccin_mocha());
    Ok(hl.highlight("markdown", &command)?)
}

pub fn thinking_spinner() -> Spinner {
    Spinner::new(spinners::Dots, "Thinking ...", Color::Blue)
}

pub fn collect_form_inputs<K>(entries: Vec<K>) -> Result<HashMap<K, String>>
where
    K: Display + std::hash::Hash + Eq + PartialEq,
{
    let mut values = HashMap::new();
    for entry in entries {
        let value: String = Input::new()
            .with_prompt(format!("{}", entry))
            .interact_text()?;
        values.insert(entry, value);
    }
    Ok(values)
}
