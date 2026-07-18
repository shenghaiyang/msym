use heck::{ToSnakeCase, ToUpperCamelCase};
use std::fmt::{Display, Formatter};

/// A normalized Material Symbols name.
#[derive(Debug, Clone)]
pub struct SymbolName(String);

impl SymbolName {
    pub fn new(raw: String) -> Self {
        Self(raw)
    }

    /// The normalized snake_case name (used in URLs).
    pub fn to_url_name(&self) -> String {
        self.0.to_snake_case()
    }

    /// Convert to a PascalCase Kotlin file name (without extension).
    pub fn to_filename(&self) -> String {
        self.0.to_upper_camel_case()
    }
}

impl Display for SymbolName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
