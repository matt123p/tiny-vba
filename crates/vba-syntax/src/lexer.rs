use crate::{MAX_DIAGNOSTICS, MAX_TOKENS};
use vba_diagnostics::{Diagnostic, SourceFile, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Number,
    String,
    Symbol,
    Newline,
    Whitespace,
    Comment,
    Continuation,
    Eof,
}
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
impl Token {
    pub fn text<'a>(&self, source: &'a SourceFile) -> Result<&'a str, Diagnostic> {
        source.slice(self.span)
    }
    pub fn is_trivia(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Whitespace | TokenKind::Comment | TokenKind::Continuation
        )
    }
}
#[derive(Clone, Debug)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &SourceFile) -> Lexed {
    let mut scanner = Scanner {
        source,
        pos: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        logical_has_code: false,
    };
    scanner.scan();
    Lexed {
        tokens: scanner.tokens,
        diagnostics: scanner.diagnostics,
    }
}
struct Scanner<'a> {
    source: &'a SourceFile,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    logical_has_code: bool,
}
impl Scanner<'_> {
    fn remaining(&self) -> &str {
        self.source.text().get(self.pos..).unwrap_or_default()
    }
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }
    fn bump(&mut self) {
        if let Some(c) = self.peek() {
            self.pos += c.len_utf8();
        }
    }
    fn emit(&mut self, kind: TokenKind, start: usize) {
        if let (Ok(a), Ok(b)) = (u32::try_from(start), u32::try_from(self.pos))
            && let Ok(span) = self.source.span(a, b)
        {
            self.tokens.push(Token { kind, span });
            if kind == TokenKind::Newline {
                self.logical_has_code = false;
            } else if !matches!(
                kind,
                TokenKind::Whitespace
                    | TokenKind::Comment
                    | TokenKind::Continuation
                    | TokenKind::Eof
            ) {
                self.logical_has_code = true;
            }
        }
    }
    fn error(&mut self, code: &'static str, message: &str, start: usize) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            let diagnostic = Diagnostic::new(code, message);
            let diagnostic = match (u32::try_from(start), u32::try_from(self.pos)) {
                (Ok(a), Ok(b)) => self
                    .source
                    .span(a, b)
                    .map_or(diagnostic.clone(), |s| diagnostic.at(s)),
                _ => diagnostic,
            };
            self.diagnostics.push(diagnostic);
        }
    }
    fn newline(&mut self) {
        if self.peek() == Some('\r') {
            self.bump();
        }
        if self.peek() == Some('\n') {
            self.bump();
        }
    }
    fn scan(&mut self) {
        while let Some(c) = self.peek() {
            if self.tokens.len() >= MAX_TOKENS - 1 {
                self.error("VBA0102", "frontend token budget exceeded", self.pos);
                break;
            }
            let start = self.pos;
            match c {
                ' ' | '\t' => {
                    while matches!(self.peek(), Some(' ' | '\t')) {
                        self.bump();
                    }
                    self.emit(TokenKind::Whitespace, start);
                }
                '\r' | '\n' => {
                    self.newline();
                    self.emit(TokenKind::Newline, start);
                }
                '\'' => {
                    while self.peek().is_some_and(|c| c != '\r' && c != '\n') {
                        self.bump();
                    }
                    self.emit(TokenKind::Comment, start);
                }
                '_' => {
                    let previous_space = self
                        .source
                        .text()
                        .get(..start)
                        .and_then(|s| s.chars().next_back())
                        .is_some_and(|c| c == ' ' || c == '\t');
                    self.bump();
                    while matches!(self.peek(), Some(' ' | '\t')) {
                        self.bump();
                    }
                    let has_code = self.logical_has_code;
                    if previous_space && has_code && matches!(self.peek(), Some('\r' | '\n')) {
                        self.newline();
                        self.emit(TokenKind::Continuation, start);
                    } else {
                        self.error("VBA0100", "invalid explicit line continuation", start);
                        self.emit(TokenKind::Symbol, start);
                    }
                }
                '"' => {
                    self.bump();
                    let mut closed = false;
                    while let Some(c) = self.peek() {
                        if c == '\r' || c == '\n' {
                            break;
                        }
                        self.bump();
                        if c == '"' {
                            if self.peek() == Some('"') {
                                self.bump();
                            } else {
                                closed = true;
                                break;
                            }
                        }
                    }
                    if !closed {
                        self.error("VBA0100", "unterminated string literal", start);
                    }
                    self.emit(TokenKind::String, start);
                }
                c if c.is_ascii_alphabetic() => {
                    self.bump();
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        self.bump();
                    }
                    let word = self.source.text().get(start..self.pos).unwrap_or_default();
                    if word.eq_ignore_ascii_case("Rem")
                        && self.peek().is_none_or(|c| c.is_whitespace())
                    {
                        while self.peek().is_some_and(|c| c != '\r' && c != '\n') {
                            self.bump();
                        }
                        self.emit(TokenKind::Comment, start);
                    } else {
                        if self.type_suffix(true) {
                            self.bump();
                        }
                        self.emit(TokenKind::Word, start);
                    }
                }
                c if c.is_ascii_digit()
                    || (c == '.'
                        && self
                            .remaining()
                            .chars()
                            .nth(1)
                            .is_some_and(|c| c.is_ascii_digit())) =>
                {
                    self.decimal(start);
                    self.emit(TokenKind::Number, start);
                }
                '&' if self
                    .remaining()
                    .chars()
                    .nth(1)
                    .is_some_and(|c| matches!(c, 'h' | 'H' | 'o' | 'O')) =>
                {
                    self.bump();
                    let hex = matches!(self.peek(), Some('h' | 'H'));
                    self.bump();
                    let digits = self.pos;
                    while self.peek().is_some_and(|c| {
                        if hex {
                            c.is_ascii_hexdigit()
                        } else {
                            matches!(c, '0'..='7')
                        }
                    }) {
                        self.bump();
                    }
                    if digits == self.pos {
                        self.error("VBA0100", "expected radix digits", start);
                    }
                    if self.type_suffix(false) {
                        self.bump();
                    }
                    self.emit(TokenKind::Number, start);
                }
                '(' | ')' | ',' | '.' | ':' | '+' | '-' | '*' | '/' | '\\' | '^' | '&' | '='
                | '<' | '>' => {
                    self.bump();
                    if matches!(
                        (c, self.peek()),
                        (':', Some('=')) | ('<', Some('=' | '>')) | ('>', Some('='))
                    ) {
                        self.bump();
                    }
                    self.emit(TokenKind::Symbol, start);
                }
                _ => {
                    self.bump();
                    self.error(
                        "VBA0001",
                        "UNSUPPORTED: character or literal form in M1",
                        start,
                    );
                    self.emit(TokenKind::Symbol, start);
                }
            }
        }
        self.emit(TokenKind::Eof, self.pos);
    }
    fn type_suffix(&self, identifier: bool) -> bool {
        match self.peek() {
            Some('%' | '!' | '#' | '@') => true,
            Some('$') => identifier,
            // Ambiguous adjacent &, ^ are operators; separated suffix forms are preserved.
            Some('&' | '^') => self
                .remaining()
                .chars()
                .nth(1)
                .is_none_or(|c| c.is_whitespace() || matches!(c, ',' | ')' | ':' | '=')),
            _ => false,
        }
    }
    fn decimal(&mut self, start: usize) {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
            }
        }
        if matches!(self.peek(), Some('e' | 'E' | 'd' | 'D')) {
            self.bump();
            if matches!(self.peek(), Some('+' | '-')) {
                self.bump();
            }
            let exponent = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
            }
            if self.pos == exponent {
                self.error("VBA0100", "expected exponent digits", start);
            }
        }
        if self.type_suffix(false) {
            self.bump();
        }
    }
}
