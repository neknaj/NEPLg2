//! Source-level cache key construction for compiler-session output caches.
//!
//! The compiler session may reuse already compiled output when the source text
//! changes only in trivia that the lexer discards.  This module keeps that rule
//! in `nepl-core` as a pure function so every platform frontend can share the
//! same cache boundary.

use alloc::string::{String, ToString};

use crate::ast::{Effect, Visibility};
use crate::lexer::{lex, TokenKind};
use crate::span::FileId;

/// Build a cache key fragment for a NEPL source file.
///
/// The key is based on the lexer token stream when lexing succeeds, and falls
/// back to the raw source text when lexing reports diagnostics.  A successful
/// token-stream key deliberately ignores ordinary/doc comments and
/// position-only span changes, while preserving directives, indentation tokens,
/// raw wasm/llvm text, multiline string lines, and every syntax token payload.
///
/// This boundary is intended only for successful compiled-output caches.  It is
/// not a replacement for diagnostics caching: invalid source uses the raw text
/// key so a previous successful compile cannot hide a later lexer error.
pub fn compiled_source_cache_key_part(source: &str) -> String {
    let lexed = lex(FileId(0), source);
    if !lexed.diagnostics.is_empty() {
        let mut key = String::new();
        push_source_key_part(&mut key, "nepl-source-raw-v1");
        push_source_key_part(&mut key, source);
        return key;
    }

    let mut key = String::new();
    push_source_key_part(&mut key, "nepl-source-token-v1");
    let mut line_has_syntax = false;
    let mut skip_doc_only_newline = false;
    for token in lexed.tokens {
        match &token.kind {
            TokenKind::DocComment(_) => {
                if !line_has_syntax {
                    skip_doc_only_newline = true;
                }
            }
            TokenKind::Newline if skip_doc_only_newline && !line_has_syntax => {
                skip_doc_only_newline = false;
            }
            TokenKind::Newline => {
                skip_doc_only_newline = false;
                line_has_syntax = false;
                push_source_key_part(&mut key, token_kind_cache_repr(&token.kind).as_str());
            }
            TokenKind::Indent | TokenKind::Dedent => {
                push_source_key_part(&mut key, token_kind_cache_repr(&token.kind).as_str());
            }
            _ => {
                skip_doc_only_newline = false;
                line_has_syntax = true;
                push_source_key_part(&mut key, token_kind_cache_repr(&token.kind).as_str());
            }
        }
    }
    key
}

fn token_kind_cache_repr(kind: &TokenKind) -> String {
    let mut out = String::new();
    write_token_kind_cache_repr(&mut out, kind);
    out
}

fn write_token_kind_cache_repr(out: &mut String, kind: &TokenKind) {
    match kind {
        TokenKind::Indent => out.push_str("Indent"),
        TokenKind::Dedent => out.push_str("Dedent"),
        TokenKind::Newline => out.push_str("Newline"),
        TokenKind::Eof => out.push_str("Eof"),
        TokenKind::Colon => out.push_str("Colon"),
        TokenKind::Semicolon => out.push_str("Semicolon"),
        TokenKind::Pipe => out.push_str("Pipe"),
        TokenKind::LParen => out.push_str("LParen"),
        TokenKind::RParen => out.push_str("RParen"),
        TokenKind::Comma => out.push_str("Comma"),
        TokenKind::LAngle => out.push_str("LAngle"),
        TokenKind::RAngle => out.push_str("RAngle"),
        TokenKind::Percent => out.push_str("Percent"),
        TokenKind::Backslash => out.push_str("Backslash"),
        TokenKind::Arrow(effect) => {
            out.push_str("Arrow");
            push_source_key_part(out, effect_cache_repr(*effect));
        }
        TokenKind::PathSep => out.push_str("PathSep"),
        TokenKind::At => out.push_str("At"),
        TokenKind::Dot => out.push_str("Dot"),
        TokenKind::Ampersand => out.push_str("Ampersand"),
        TokenKind::Star => out.push_str("Star"),
        TokenKind::Minus => out.push_str("Minus"),
        TokenKind::Equals => out.push_str("Equals"),
        TokenKind::Ident(value) => {
            out.push_str("Ident");
            push_source_key_part(out, value);
        }
        TokenKind::IntLiteral(value) => {
            out.push_str("IntLiteral");
            push_source_key_part(out, value);
        }
        TokenKind::FloatLiteral(value) => {
            out.push_str("FloatLiteral");
            push_source_key_part(out, value);
        }
        TokenKind::BoolLiteral(value) => {
            out.push_str("BoolLiteral");
            push_source_key_part(out, if *value { "true" } else { "false" });
        }
        TokenKind::CharLiteral(value) => {
            out.push_str("CharLiteral");
            push_source_key_part(out, &value.to_string());
        }
        TokenKind::StringLiteral(value) => {
            out.push_str("StringLiteral");
            push_source_key_part(out, value);
        }
        TokenKind::UnitLiteral => out.push_str("UnitLiteral"),
        TokenKind::KwFn => out.push_str("KwFn"),
        TokenKind::KwLet => out.push_str("KwLet"),
        TokenKind::KwMut => out.push_str("KwMut"),
        TokenKind::KwNoShadow => out.push_str("KwNoShadow"),
        TokenKind::KwSet => out.push_str("KwSet"),
        TokenKind::KwIf => out.push_str("KwIf"),
        TokenKind::KwWhile => out.push_str("KwWhile"),
        TokenKind::KwCond => out.push_str("KwCond"),
        TokenKind::KwThen => out.push_str("KwThen"),
        TokenKind::KwElse => out.push_str("KwElse"),
        TokenKind::KwDo => out.push_str("KwDo"),
        TokenKind::KwStruct => out.push_str("KwStruct"),
        TokenKind::KwEnum => out.push_str("KwEnum"),
        TokenKind::KwMatch => out.push_str("KwMatch"),
        TokenKind::KwTrait => out.push_str("KwTrait"),
        TokenKind::KwImpl => out.push_str("KwImpl"),
        TokenKind::KwFor => out.push_str("KwFor"),
        TokenKind::KwPub => out.push_str("KwPub"),
        TokenKind::KwBlock => out.push_str("KwBlock"),
        TokenKind::KwTuple => out.push_str("KwTuple"),
        TokenKind::KwMlstr => out.push_str("KwMlstr"),
        TokenKind::DirEntry(value) => {
            out.push_str("DirEntry");
            push_source_key_part(out, value);
        }
        TokenKind::DirTarget(value) => {
            out.push_str("DirTarget");
            push_source_key_part(out, value);
        }
        TokenKind::DirImport(value) => {
            out.push_str("DirImport");
            push_source_key_part(out, value);
        }
        TokenKind::DirUse(value) => {
            out.push_str("DirUse");
            push_source_key_part(out, value);
        }
        TokenKind::DirIfTarget(value) => {
            out.push_str("DirIfTarget");
            push_source_key_part(out, value);
        }
        TokenKind::DirIfProfile(value) => {
            out.push_str("DirIfProfile");
            push_source_key_part(out, value);
        }
        TokenKind::DirCapability(value) => {
            out.push_str("DirCapability");
            push_source_key_part(out, value);
        }
        TokenKind::DirWasm => out.push_str("DirWasm"),
        TokenKind::DirLlvmIr => out.push_str("DirLlvmIr"),
        TokenKind::DirIndentWidth(value) => {
            out.push_str("DirIndentWidth");
            push_source_key_part(out, &value.to_string());
        }
        TokenKind::DirInclude(value) => {
            out.push_str("DirInclude");
            push_source_key_part(out, value);
        }
        TokenKind::DirExtern {
            vis,
            module,
            name,
            func,
            signature,
        } => {
            out.push_str("DirExtern");
            push_source_key_part(out, &vis_cache_repr(*vis));
            push_source_key_part(out, module);
            push_source_key_part(out, name);
            push_source_key_part(out, func);
            push_source_key_part(out, signature);
        }
        TokenKind::DirIntrinsic => out.push_str("DirIntrinsic"),
        TokenKind::DirPrelude(value) => {
            out.push_str("DirPrelude");
            push_source_key_part(out, value);
        }
        TokenKind::DirNoPrelude => out.push_str("DirNoPrelude"),
        TokenKind::WasmText(value) => {
            out.push_str("WasmText");
            push_source_key_part(out, value);
        }
        TokenKind::LlvmIrText(value) => {
            out.push_str("LlvmIrText");
            push_source_key_part(out, value);
        }
        TokenKind::MlstrLine(value) => {
            out.push_str("MlstrLine");
            push_source_key_part(out, value);
        }
        TokenKind::DocComment(value) => {
            out.push_str("DocComment");
            push_source_key_part(out, value);
        }
    }
}

fn effect_cache_repr(effect: Effect) -> &'static str {
    match effect {
        Effect::Pure => "Pure",
        Effect::Impure => "Impure",
    }
}

fn vis_cache_repr(vis: Visibility) -> String {
    match vis {
        Visibility::Pub => "Pub".to_string(),
        Visibility::Private => "Private".to_string(),
    }
}

fn push_source_key_part(key: &mut String, value: &str) {
    key.push_str(&value.len().to_string());
    key.push(':');
    key.push_str(value);
    key.push('\n');
}

#[cfg(test)]
mod tests {
    use super::compiled_source_cache_key_part;

    #[test]
    fn compiled_source_cache_key_ignores_ordinary_comments() {
        let base = "#target std\nfn main <()->i32> ():\n    1\n";
        let commented =
            "// file comment\n#target std\nfn main <()->i32> (): // trailing comment\n    1 // value\n";

        assert_eq!(
            compiled_source_cache_key_part(base),
            compiled_source_cache_key_part(commented)
        );
    }

    #[test]
    fn compiled_source_cache_key_ignores_doc_comments_for_compiled_output() {
        let first = "//: first contract\nfn value <()->i32> ():\n    1\n";
        let second = "//: second contract\nfn value <()->i32> ():\n    1\n";

        assert_eq!(
            compiled_source_cache_key_part(first),
            compiled_source_cache_key_part(second)
        );
    }

    #[test]
    fn compiled_source_cache_key_keeps_structure_after_doc_comment_line() {
        let without_doc = "fn value <()->i32> ():\n    1\n";
        let with_doc = "fn value <()->i32> ():\n    //: branch contract\n    1\n";

        assert_eq!(
            compiled_source_cache_key_part(without_doc),
            compiled_source_cache_key_part(with_doc)
        );
    }

    #[test]
    fn compiled_source_cache_key_preserves_offside_tokens() {
        let indented = "fn value <()->i32> ():\n    1\n";
        let flat = "fn value <()->i32> ():\n1\n";

        assert_ne!(
            compiled_source_cache_key_part(indented),
            compiled_source_cache_key_part(flat)
        );
    }

    #[test]
    fn compiled_source_cache_key_uses_raw_source_for_lexer_errors() {
        let invalid = "fn value <()->i32> ():\n    ?\n";

        assert!(
            compiled_source_cache_key_part(invalid).contains("nepl-source-raw-v1"),
            "invalid source must not share a token-only key with a successful compile"
        );
    }
}
