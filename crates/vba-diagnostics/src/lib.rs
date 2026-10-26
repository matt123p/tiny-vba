//! Source-aware diagnostics shared by the compiler, VM and frontends.
#![forbid(unsafe_code)]

use std::fmt;

/// A stable machine-readable diagnostic with a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    pub fn unsupported(feature: &str) -> Self {
        Self {
            code: "VBA0001",
            message: format!("UNSUPPORTED: {feature} is not implemented in this scaffold"),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Diagnostic {}
