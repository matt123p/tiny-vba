//! Bounded, source-preserving M1 frontend. Parsing never implies binding or execution.
#![forbid(unsafe_code)]
mod ast;
mod lexer;
mod parser;

pub use ast::*;
pub use lexer::{Lexed, Token, TokenKind, lex};
pub use parser::{ParseResult, parse};

pub const MAX_TOKENS: usize = 65_536;
pub const MAX_NESTING: usize = 64;
pub const MAX_DIAGNOSTICS: usize = 32;
