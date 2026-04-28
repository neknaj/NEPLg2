---
id: ISS-20260428T004316513Z-LANGUAGE-LACKS-CHAR-TYPE-AND-SINGLE--63768838
title: "language lacks char type and single-quoted char literals"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/lexer.rs, nepl-core/src/parser.rs, nepl-core/src/ast.rs, nepl-core/src/types.rs, nepl-core/src/typecheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-language/src/lib.rs, nepl-web/src/lib.rs, stdlib/core/traits/copy.nepl, stdlib/neplg2/core/syntax/token.nepl, stdlib/neplg2/core/syntax/lexer.nepl, nepl-core/tests/char.rs, tests/compiler/match_literal_patterns.n.md, tests/stdlib/neplg2_lexer.n.md"
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

## 解決

- `char` を primitive `Copy` 型として `TypeKind` / type atom / backend layout に追加し、WASM/LLVM では Unicode scalar value を `i32` 互換表現で扱うようにした。
- single-quoted char literal を lexer/parser/AST/typecheck に追加し、通常文字、`'\n'` / `'\r'` / `'\t'` / `'\0'` / `'\\'` / `'\''` / `'\"'` / `'\xNN'` / `'\u{...}'` を 1 Unicode scalar として検証するようにした。
- 空 literal、複数 scalar、未終端、invalid escape、surrogate/out-of-range を lexer diagnostic として拒否するようにした。
- char literal の型は既定で `char` とし、明示型注釈または関数引数の期待型が `i32` / `u8` の場合に literal だけ code point へ文脈解決するようにした。`char` 変数から整数への暗黙変換は許可しない。
- char literal match arm を追加し、`char` subject では char literal と wildcard のみを有効な scalar pattern として扱うようにした。
- playground/language/web の token 表示と selfhost lexer/token model に `CharLiteral` / `UnterminatedChar` を追加した。
- `stdlib/core/traits/copy.nepl` に `char` の `Clone` / `Copy` 実装を追加した。

## 検証結果

- `cargo test -p nepl-core --test char`
- `cargo test -p nepl-core`
- `cargo check`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-tests-char.json`
- `node nodesrc/tests.js -i tests/compiler/match_literal_patterns.n.md --no-tree -o tmp/char-match-literal-patterns.json -j 1`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/char-selfhost-lexer.json -j 1`

## 残作業

- stdlib の char API と string 連携は `ISS-20260428T004640143Z-STDLIB-NEEDS-CHAR-API-AND-STRING-INT-547D97E5` で扱う。
- 既存 stdlib/selfhost の character-code magic number 置換は `ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB` で扱う。
