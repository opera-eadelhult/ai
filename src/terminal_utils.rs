use std::{collections::HashMap, fmt::Display};

use arborium::{AnsiHighlighter, theme::builtin};
use dialoguer::Input;
use eyre::Result;

pub fn highlight_bash(command: &str) -> Result<String> {
    let mut hl = AnsiHighlighter::new(builtin::catppuccin_mocha());
    Ok(hl.highlight("bash", &command)?)
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
