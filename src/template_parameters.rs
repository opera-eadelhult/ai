use std::{cell::LazyCell, collections::HashMap};

use regex::{Captures, Regex};

/// Find and replace instances of `<parameter-name>` in a string.
#[derive(Debug)]
pub struct TemplateParameters<'a> {
    bash_command: &'a str,
    captures: Vec<Captures<'a>>,
}

impl<'a> TemplateParameters<'a> {
    pub fn parse(bash_command: &'a str) -> Option<Self> {
        const PARAMETER_TEMPLATE: LazyCell<Regex> =
            LazyCell::new(|| Regex::new(r"<([\w-]+)>").unwrap());

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

    pub fn parameters(&self) -> Vec<&'a str> {
        self.captures
            .iter()
            .map(|capture| capture.get(1).unwrap().as_str())
            .collect()
    }

    pub fn apply_parameter_values(self, values: HashMap<&'a str, String>) -> String {
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
