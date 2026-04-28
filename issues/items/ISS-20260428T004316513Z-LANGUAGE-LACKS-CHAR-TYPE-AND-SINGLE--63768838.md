---
id: ISS-20260428T004316513Z-LANGUAGE-LACKS-CHAR-TYPE-AND-SINGLE--63768838
title: "language lacks char type and single-quoted char literals"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/lexer.rs, nepl-core/src/parser.rs, nepl-core/src/ast.rs, nepl-core/src/types.rs, nepl-core/src/typecheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/compiler/char_literals.n.md"
---

# ISS-20260428T004316513Z-LANGUAGE-LACKS-CHAR-TYPE-AND-SINGLE--63768838: language lacks char type and single-quoted char literals

## 概要

NEPLg2 currently has string, integer, float, bool literals but no single-quoted character literal token and no char primitive type. The Rust lexer only handles double-quoted StringLiteral and numeric literals, parser type atoms list i32/u8/f32/f64/bool/str/unit, and TypeKind has no Char variant. Source such as 'a' is therefore not a first-class literal and stdlib code writes ASCII / control bytes as opaque numbers like 10, 34, 92.

## 対象

- `nepl-core/src/lexer.rs, nepl-core/src/parser.rs, nepl-core/src/ast.rs, nepl-core/src/types.rs, nepl-core/src/typecheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, tests/compiler/char_literals.n.md`

## 関連ドキュメント

- [NEPLg2 stdlib char 整備計画](../../doc/neplg2/char_stdlib_integration_plan.md)

## 根拠

- `nepl-core/src/lexer.rs` の `TokenKind` は `FloatLiteral` / `BoolLiteral` / `StringLiteral` を持つが、`CharLiteral` を持たない。
- `nepl-core/src/lexer.rs` の通常 lexer は `b'"'` で string literal を処理する一方、`b'\''` の single quote literal 分岐を持たない。
- `nepl-core/src/parser.rs` の type atom は `i32` / `u8` / `f32` / `f64` / `bool` / `str` / `()` を扱うが、`char` を扱わない。
- `nepl-core/src/types.rs` の `TypeKind` / builtin arena に `Char` がなく、copy / drop / unify / layout の対象にも入っていない。

## 問題

NEPLg2 currently has string, integer, float, bool literals but no single-quoted character literal token and no char primitive type. The Rust lexer only handles double-quoted StringLiteral and numeric literals, parser type atoms list i32/u8/f32/f64/bool/str/unit, and TypeKind has no Char variant. Source such as 'a' is therefore not a first-class literal and stdlib code writes ASCII / control bytes as opaque numbers like 10, 34, 92.

## 影響

Byte and text classifiers in stdlib/selfhost remain hard to review because fixed characters are written as decimal codes. This also blocks idiomatic match arms such as '\n': or '\\': and keeps compiler-workaround-looking code in lexer, parser, string, JSON, nm, HTML, CLI, and byte builder paths. Self-host parser work will accumulate numeric magic constants unless char support lands early.

## 修正方針

Define and implement char support as a language feature. Proposed spec: add a primitive Copy type char represented as a Unicode scalar value with backend storage compatible with i32; add single-quoted char literals with escapes '\n', '\r', '\t', '\0', '\\', '\'', '\"' if accepted, '\xNN', and optionally '\u{H...}'. A char literal must decode to exactly one Unicode scalar; empty, multi-character, unterminated, surrogate, out-of-range, and invalid escape literals are diagnostics. Literal typing is contextual: default type char, expected char gives char, expected u8/i32 gives code point if it fits; char variables do not implicitly coerce to integers. Add char literal match patterns for char/u8/i32 scalar matches with duplicate detection and wildcard requirements for non-finite domains. Update AST/HIR/typecheck/WASM/LLVM/string literal tables/playground traces and selfhost lexer token model.

## 検証

Add tests/compiler/char_literals.n.md covering char variable binding, contextual u8/i32 literals, escapes, invalid literals, match arms, duplicate arms, and no implicit char-to-i32 variable coercion. Run cargo test -p nepl-core, trunk build, focused compiler tests, and stdlib/selfhost lexer tests.
