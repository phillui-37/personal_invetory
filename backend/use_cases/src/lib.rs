pub mod ebook;
pub mod location;
pub mod web_reader;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field_errors: BTreeMap<String, Vec<String>>,
}

impl ValidationError {
    pub fn new() -> Self {
        Self {
            field_errors: BTreeMap::new(),
        }
    }

    pub fn add_field_error(&mut self, field: &str, message: &str) {
        self.field_errors
            .entry(field.to_string())
            .or_default()
            .push(message.to_string());
    }

    pub fn is_empty(&self) -> bool {
        self.field_errors.is_empty()
    }
}

pub fn use_cases_ready() -> bool {
    domain::domain_ready()
}
