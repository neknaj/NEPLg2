//! Indentation-aware lexer for NEPLG2.

#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic::Diagnostic;
use crate::span::{FileId, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // structural
    Indent,
    Dedent,
    Newline,
    Eof,

    // punctuation / operators
    Colon,
    Semicolon,
    Pipe,
    LParen,
    RParen,
    Comma,
    LAngle,
    RAngle,
    Arrow(Effect), // -> (Pure) or *> (Impure)
    PathSep,       // ::
    Dot,
    Ampersand, // &
    Star,      // *
    Minus,     // -
    Equals,    // =

    // literals / identifiers
    Ident(String),
    IntLiteral(String),
    FloatLiteral(String),
    BoolLiteral(bool),
    StringLiteral(String),
    UnitLiteral,

    // keywords
    KwFn,
    KwLet,
    KwMut,
    KwSet,
    KwIf,
    KwWhile,
    KwStruct,
    KwEnum,
    KwMatch,
    KwTrait,
    KwImpl,
    KwFor,

    // directives
    DirEntry(String),
    DirTarget(String),
    DirImport(String),
    DirUse(String),
    DirIfTarget(String),
    DirIfProfile(String),
    DirWasm,
    DirIndentWidth(usize),
    DirInclude(String),
    DirExtern {
        module: String,
        name: String,
        func: String,
        signature: String,
    },
    DirIntrinsic,

    // wasm text line (inside #wasm: block)
    WasmText(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub struct LexResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
    pub indent_width: usize,
}

struct LexState<'a> {
    file_id: FileId,
    src: &'a str,
    indent_stack: Vec<usize>,
    indent_unit: usize,
    expect_indent: bool,
    diagnostics: Vec<Diagnostic>,
    tokens: Vec<Token>,
    wasm_base: Option<usize>,
    pending_wasm_base: Option<usize>,
}

pub fn lex(file_id: FileId, src: &str) -> LexResult {
    let mut state = LexState {
        file_id,
        src,
        indent_stack: vec![0],
        indent_unit: 4,
        expect_indent: false,
        diagnostics: Vec::new(),
        tokens: Vec::new(),
        wasm_base: None,
        pending_wasm_base: None,
    };

    let mut offset = 0usize;
    for part in src.split_inclusive('\n') {
        let has_newline = part.ends_with('\n');
        let mut line = if has_newline {
            &part[..part.len() - 1]
        } else {
            part
        };
        if line.ends_with('\r') {
            line = &line[..line.len() - 1];
        }
        state.process_line(line, offset);
        offset += part.len();
    }

    // Handle possible trailing line without newline
    if !src.ends_with('\n') && !src.is_empty() && (src.as_bytes()[src.len() - 1] != b'\n') {
        // already processed by split_inclusive; nothing extra needed
    }

    state.flush_dedent(offset);
    state.push_token(TokenKind::Eof, offset, offset);

    LexResult {
        tokens: state.tokens,
        diagnostics: state.diagnostics,
        indent_width: state.indent_unit,
    }
}

impl<'a> LexState<'a> {
    fn process_line(&mut self, line: &str, line_start: usize) {
        // Strip comments
        let content_owned = match line.find("//") {
            Some(idx) => line[..idx].to_string(),
            None => line.to_string(),
        };
        let content = content_owned.as_str();

        // Skip empty lines (do not affect indent stack)
        if content.trim().is_empty() {
            return;
        }

        let allow_indent = self.expect_indent || self.pending_wasm_base.is_some();
        self.expect_indent = false;

        // compute indentation locally to avoid borrowing issues
        let mut width = 0usize;
        let mut idx = 0usize;
        for ch in content.as_bytes() {
            match ch {
                b' ' => {
                    width += 1;
                    idx += 1;
                }
                b'\t' => {
                    let span = Span::new(
                        self.file_id,
                        (line_start + idx) as u32,
                        (line_start + idx + 1) as u32,
                    );
                    self.diagnostics.push(Diagnostic::error(
                        "tabs are not allowed for indentation",
                        span,
                    ));
                    width += self.indent_unit;
                    idx += 1;
                }
                _ => break,
            }
        }
        let actual_indent = width;
        let rest_owned = content[idx..].to_string();
        let rest = rest_owned.as_str();

        // Handle wasm block short-circuit
        let mut in_wasm = false;
        if let Some(base) = self.wasm_base {
            if actual_indent >= base {
                in_wasm = true;
            } else {
                self.wasm_base = None;
            }
        }

        // When #wasm: was seen on previous line, lock expected base
        let mut effective_indent = actual_indent;
        if let Some(expected) = self.pending_wasm_base.take() {
            if actual_indent < expected {
                let span = Span::new(self.file_id, line_start as u32, line_start as u32);
                self.diagnostics.push(Diagnostic::error(
                    "expected indented block after #wasm",
                    span,
                ));
            } else {
                self.wasm_base = Some(expected);
                in_wasm = true;
                effective_indent = expected;
            }
        }

        let is_directive = !in_wasm && rest.trim_start().starts_with('#');
        if is_directive && !allow_indent {
            let current_indent = *self.indent_stack.last().unwrap();
            if actual_indent > current_indent {
                effective_indent = current_indent;
            }
        }

        // Always emit INDENT/DEDENT to keep parser block structure even inside #wasm.
        self.adjust_indent(effective_indent, line_start);

        let line_offset = line_start + (content.len() - rest.len());

        if in_wasm {
            let text = rest.trim_end().to_string();
            let end = line_start + content.len();
            self.push_token(TokenKind::WasmText(text), line_offset, end);
        } else if is_directive {
            self.lex_directive(rest.trim_start(), line_offset, content.len());
        } else {
            self.lex_regular(rest, line_offset);
        }

        let newline_pos = line_start + content.len();
        self.tokens.push(Token {
            kind: TokenKind::Newline,
            span: Span::new(self.file_id, newline_pos as u32, newline_pos as u32),
        });

        if !in_wasm && !is_directive && content.trim_end().ends_with(':') {
            self.expect_indent = true;
        }
    }

    fn adjust_indent(&mut self, indent: usize, line_start: usize) {
        let current = *self.indent_stack.last().unwrap();
        if indent > current {
            if indent % self.indent_unit != 0 {
                let span = Span::new(self.file_id, line_start as u32, line_start as u32);
                self.diagnostics.push(Diagnostic::error(
                    "indentation is not aligned to #indent width",
                    span,
                ));
            }
            self.indent_stack.push(indent);
            self.push_token(TokenKind::Indent, line_start, line_start);
        } else if indent < current {
            while let Some(&top) = self.indent_stack.last() {
                if top == indent {
                    break;
                }
                self.indent_stack.pop();
                self.push_token(TokenKind::Dedent, line_start, line_start);
            }
            if *self.indent_stack.last().unwrap() != indent {
                let span = Span::new(self.file_id, line_start as u32, line_start as u32);
                self.diagnostics.push(Diagnostic::error(
                    "indentation level does not match any previous indent",
                    span,
                ));
                self.indent_stack.push(indent);
            }
        }
    }

    fn lex_directive(&mut self, text: &str, line_offset: usize, content_len: usize) {
        let body = text.trim_start_matches('#').trim();
        if body.starts_with("entry") {
            let name = body.strip_prefix("entry").unwrap().trim();
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirEntry(name.to_string()),
                span,
            });
        } else if body.starts_with("indent") {
            let arg = body.strip_prefix("indent").unwrap().trim();
            if let Ok(width) = arg.parse::<usize>() {
                self.indent_unit = width.max(1);
                let span = Span::new(
                    self.file_id,
                    line_offset as u32,
                    (line_offset + body.len()) as u32,
                );
                self.tokens.push(Token {
                    kind: TokenKind::DirIndentWidth(width),
                    span,
                });
            } else {
                let span = Span::new(
                    self.file_id,
                    line_offset as u32,
                    (line_offset + body.len()) as u32,
                );
                self.diagnostics
                    .push(Diagnostic::error("invalid #indent argument", span));
            }
        } else if body.starts_with("import") {
            let arg = body
                .strip_prefix("import")
                .unwrap()
                .trim()
                .trim_matches('"');
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirImport(arg.to_string()),
                span,
            });
        } else if body.starts_with("target") {
            let arg = body.strip_prefix("target").unwrap().trim();
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirTarget(arg.to_string()),
                span,
            });
        } else if body.starts_with("include") {
            let arg = body
                .strip_prefix("include")
                .unwrap()
                .trim()
                .trim_matches('"');
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirInclude(arg.to_string()),
                span,
            });
        } else if body.starts_with("use") {
            let arg = body.strip_prefix("use").unwrap().trim();
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirUse(arg.to_string()),
                span,
            });
        } else if body.starts_with("if[target=") {
            if let Some(end) = body.find(']') {
                let target = &body[10..end];
                let span = Span::new(
                    self.file_id,
                    line_offset as u32,
                    (line_offset + end + 1) as u32,
                );
                self.tokens.push(Token {
                    kind: TokenKind::DirIfTarget(target.to_string()),
                    span,
                });
            }
        } else if body.starts_with("if[profile=") {
            if let Some(end) = body.find(']') {
                let profile = &body[11..end];
                let span = Span::new(
                    self.file_id,
                    line_offset as u32,
                    (line_offset + end + 1) as u32,
                );
                self.tokens.push(Token {
                    kind: TokenKind::DirIfProfile(profile.to_string()),
                    span,
                });
            }
        } else if body.starts_with("extern") {
            // format: extern "env" "sym" fn name <signature>
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            let parts: Vec<&str> = body.split_whitespace().collect();
            if parts.len() >= 5
                && parts[0] == "extern"
                && parts[2].starts_with('"')
                && parts[1].starts_with('"')
                && parts[3] == "fn"
            {
                let module = parts[1].trim_matches('"').to_string();
                let name = parts[2].trim_matches('"').to_string();
                let func = parts[4].to_string();
                let sig_start = body.find('<');
                let sig = if let Some(idx) = sig_start {
                    body[idx..].to_string()
                } else {
                    String::new()
                };
                self.tokens.push(Token {
                    kind: TokenKind::DirExtern {
                        module,
                        name,
                        func,
                        signature: sig,
                    },
                    span,
                });
            } else {
                self.diagnostics
                    .push(Diagnostic::error("invalid #extern syntax", span));
            }
        } else if body.starts_with("wasm") {
            // expect trailing colon
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + body.len()) as u32,
            );
            self.tokens.push(Token {
                kind: TokenKind::DirWasm,
                span,
            });
            // Expect body one indent deeper
            let current_indent = *self.indent_stack.last().unwrap();
            self.pending_wasm_base = Some(current_indent + self.indent_unit);
        } else if body.starts_with("intrinsic") {
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + 10) as u32, // #intrinsic
            );
            self.tokens.push(Token {
                kind: TokenKind::DirIntrinsic,
                span,
            });
            let rest = body.strip_prefix("intrinsic").unwrap();
            // Lex the rest of the line as regular tokens (args)
            let rest_start = line_offset + 10; // length of "#intrinsic"
            self.lex_regular(rest, rest_start);
        } else {
            let span = Span::new(
                self.file_id,
                line_offset as u32,
                (line_offset + content_len) as u32,
            );
            self.diagnostics
                .push(Diagnostic::error("unknown directive", span));
        }
    }

    fn lex_regular(&mut self, text: &str, offset: usize) {
        let bytes = text.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            let c = bytes[i];
            if c.is_ascii_whitespace() {
                i += 1;
                continue;
            }
            match c {
                b'(' => {
                    self.push_token(TokenKind::LParen, offset + i, offset + i + 1);
                    i += 1;
                }
                b')' => {
                    self.push_token(TokenKind::RParen, offset + i, offset + i + 1);
                    i += 1;
                }
                b'&' => {
                    self.push_token(TokenKind::Ampersand, offset + i, offset + i + 1);
                    i += 1;
                }
                b',' => {
                    self.push_token(TokenKind::Comma, offset + i, offset + i + 1);
                    i += 1;
                }
                b';' => {
                    self.push_token(TokenKind::Semicolon, offset + i, offset + i + 1);
                    i += 1;
                }
                b'|' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                        self.push_token(TokenKind::Pipe, offset + i, offset + i + 2);
                        i += 2;
                    } else {
                        self.unknown(offset + i, offset + i + 1);
                        i += 1;
                    }
                }
                b':' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b':' {
                        self.push_token(TokenKind::PathSep, offset + i, offset + i + 2);
                        i += 2;
                    } else {
                        self.push_token(TokenKind::Colon, offset + i, offset + i + 1);
                        i += 1;
                    }
                }
                b'<' => {
                    self.push_token(TokenKind::LAngle, offset + i, offset + i + 1);
                    i += 1;
                }
                b'.' => {
                    self.push_token(TokenKind::Dot, offset + i, offset + i + 1);
                    i += 1;
                }
                b'=' => {
                    self.push_token(TokenKind::Equals, offset + i, offset + i + 1);
                    i += 1;
                }
                b'>' => {
                    self.push_token(TokenKind::RAngle, offset + i, offset + i + 1);
                    i += 1;
                }
                b'-' => {
                    // allow optional whitespace between '-' and '>'
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    if i < bytes.len() && bytes[i] == b'>' {
                        self.push_token(
                            TokenKind::Arrow(Effect::Pure),
                            offset + start,
                            offset + i + 1,
                        );
                        i += 1;
                    } else {
                        self.push_token(TokenKind::Minus, offset + start, offset + start + 1);
                        i = start + 1;
                    }
                }
                b'*' => {
                    // allow optional whitespace between '*' and '>'
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    if i < bytes.len() && bytes[i] == b'>' {
                        self.push_token(
                            TokenKind::Arrow(Effect::Impure),
                            offset + start,
                            offset + i + 1,
                        );
                        i += 1;
                    } else {
                        self.push_token(TokenKind::Star, offset + start, offset + start + 1);
                        i = start + 1;
                    }
                }
                b'"' => {
                    let start = i;
                    i += 1;
                    let mut buf = String::new();
                    let mut closed = false;
                    while i < bytes.len() {
                        match bytes[i] {
                            b'\"' => {
                                closed = true;
                                i += 1;
                                break;
                            }
                            b'\\' if i + 1 < bytes.len() => {
                                let esc = bytes[i + 1];
                                let ch = match esc {
                                    b'n' => '\n',
                                    b'r' => '\r',
                                    b't' => '\t',
                                    b'\\' => '\\',
                                    b'"' => '"',
                                    b'0' => '\0',
                                    other => {
                                        self.diagnostics.push(Diagnostic::error(
                                            "invalid escape in string literal",
                                            Span::new(
                                                self.file_id,
                                                (offset + i) as u32,
                                                (offset + i + 2) as u32,
                                            ),
                                        ));
                                        other as char
                                    }
                                };
                                buf.push(ch);
                                i += 2;
                            }
                            _ => {
                                if let Some(ch) = text[i..].chars().next() {
                                    buf.push(ch);
                                    i += ch.len_utf8();
                                } else {
                                    i += 1;
                                }
                            }
                        }
                    }
                    if closed {
                        self.push_token(TokenKind::StringLiteral(buf), offset + start, offset + i);
                    } else {
                        self.diagnostics.push(Diagnostic::error(
                            "unterminated string literal",
                            Span::new(self.file_id, (offset + start) as u32, (offset + i) as u32),
                        ));
                    }
                }
                b'0'..=b'9' => {
                    let start = i;
                    let mut has_dot = false;
                    while i < bytes.len() {
                        match bytes[i] {
                            b'0'..=b'9' => i += 1,
                            b'.' if !has_dot => {
                                has_dot = true;
                                i += 1;
                            }
                            _ => break,
                        }
                    }
                    let lexeme = &text[start..i];
                    if has_dot {
                        self.push_token(
                            TokenKind::FloatLiteral(lexeme.to_string()),
                            offset + start,
                            offset + i,
                        );
                    } else {
                        self.push_token(
                            TokenKind::IntLiteral(lexeme.to_string()),
                            offset + start,
                            offset + i,
                        );
                    }
                }
                _ => {
                    if is_ident_start(c) {
                        let start = i;
                        i += 1;
                        while i < bytes.len() && is_ident_continue(bytes[i]) {
                            i += 1;
                        }
                        let lexeme = &text[start..i];
                        let span_start = offset + start;
                        let span_end = offset + i;
                        match lexeme {
                            "fn" => self.push_token(TokenKind::KwFn, span_start, span_end),
                            "let" => self.push_token(TokenKind::KwLet, span_start, span_end),
                            "mut" => self.push_token(TokenKind::KwMut, span_start, span_end),
                            "set" => self.push_token(TokenKind::KwSet, span_start, span_end),
                            "if" => self.push_token(TokenKind::KwIf, span_start, span_end),
                            "while" => self.push_token(TokenKind::KwWhile, span_start, span_end),
                            "struct" => self.push_token(TokenKind::KwStruct, span_start, span_end),
                            "enum" => self.push_token(TokenKind::KwEnum, span_start, span_end),
                            "match" => self.push_token(TokenKind::KwMatch, span_start, span_end),
                            "trait" => self.push_token(TokenKind::KwTrait, span_start, span_end),
                            "impl" => self.push_token(TokenKind::KwImpl, span_start, span_end),
                            "for" => self.push_token(TokenKind::KwFor, span_start, span_end),
                            "true" => {
                                self.push_token(TokenKind::BoolLiteral(true), span_start, span_end)
                            }
                            "false" => {
                                self.push_token(TokenKind::BoolLiteral(false), span_start, span_end)
                            }
                            _ => self.push_token(
                                TokenKind::Ident(lexeme.to_string()),
                                span_start,
                                span_end,
                            ),
                        }
                    } else {
                        self.unknown(offset + i, offset + i + 1);
                        i += 1;
                    }
                }
            }
        }
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(self.file_id, start as u32, end as u32),
        });
    }

    fn unknown(&mut self, start: usize, end: usize) {
        let span = Span::new(self.file_id, start as u32, end as u32);
        self.diagnostics
            .push(Diagnostic::error("unknown token", span));
    }

    fn flush_dedent(&mut self, pos: usize) {
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.push_token(TokenKind::Dedent, pos, pos);
        }
    }
}

fn is_ident_start(b: u8) -> bool {
    (b as char).is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    (b as char).is_ascii_alphanumeric() || b == b'_'
}
