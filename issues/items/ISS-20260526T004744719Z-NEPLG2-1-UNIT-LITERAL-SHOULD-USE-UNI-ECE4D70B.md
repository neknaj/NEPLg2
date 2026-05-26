---
id: ISS-20260526T004744719Z-NEPLG2-1-UNIT-LITERAL-SHOULD-USE-UNI-ECE4D70B
title: "NEPLg2.1 unit literal should use unit keyword"
area: core
status: fixed
resolved: true
priority: P0
type: architecture
created: 2026-05-26
updated: 2026-05-26
target: "nepl-core/src/lexer.rs; nepl-core/src/parser.rs; nepl-core/src/parser/neplg21_type_expr.rs; nodesrc/neplg21_syntax_migrate.js; stdlib/**; tests/**; tutorials/**"
---

# ISS-20260526T004744719Z-NEPLG2-1-UNIT-LITERAL-SHOULD-USE-UNI-ECE4D70B: NEPLg2.1 unit literal should use unit keyword

## 概要

NEPLg2.1 currently keeps the old () spelling for unit type, unit value, and zero-argument lambda/function syntax. This leaves a parenthesized island in the prefix surface syntax and conflicts with the requested switch to unit keyword notation.

## 対象

- `nepl-core/src/lexer.rs; nepl-core/src/parser.rs; nepl-core/src/parser/neplg21_type_expr.rs; nodesrc/neplg21_syntax_migrate.js; stdlib/**; tests/**; tutorials/**`

## 根拠

- The Rust lexer already has `TokenKind::UnitLiteral`, but `unit` was not classified as that token, so NEPLg2.1 source could not spell unit as a keyword.
- The NEPLg2.1 prefix type parser already knows `UnitLiteral`, while zero-argument function types and lambdas still used the old `()` marker.
- The NEPLg2.1 migrator still emitted `fn () T` and `\()` and therefore preserved obsolete unit spelling in newly migrated corpus.
- The requested NEPLg2.1 design treats `unit` as a keyword replacing current `()` unit notation. `fn unit T` and `\unit` are zero-argument markers, not a unit-typed parameter.

## 問題

NEPLg2.1 currently keeps the old () spelling for unit type, unit value, and zero-argument lambda/function syntax. This leaves a parenthesized island in the prefix surface syntax and conflicts with the requested switch to unit keyword notation.

## 影響

Corpus and docs would keep obsolete NEPLg2.0-style unit spelling, and new examples such as fn main %impure fn unit i32 \\unit cannot be represented without frontend support.

## 修正方針

Classify `unit` as a keyword-backed `UnitLiteral`, parse `fn unit T` as a zero-argument function type, parse `\unit` as a zero-argument lambda/function marker, update the NEPLg2.1 migrator, and migrate source/docs away from `()` where it denotes unit rather than grouping or intrinsic argument delimiters.

NEPLg2.1 does not add a surface syntax for a function that takes unit as an ordinary parameter. The existing `()` role is replaced by `unit`: type `unit`, value `unit`, zero-argument function type marker `fn unit T`, and zero-argument lambda marker `\unit`.

## 検証

Add focused Rust frontend tests for `%unit`, unit value `unit`, `fn unit T`, and `\unit`. Run the NEPLg2.1 migrator check, issue checker, diff whitespace checks, and focused compiler tests.

## 解決

- Rust lexer と selfhost lexer keyword table で `unit` を `UnitLiteral` として分類するようにした。
- NEPLg2.1 parser は `fn unit T` を 0 引数関数型、`\unit` を 0 引数関数リテラル marker として既存の空 parameter list へ正規化する。
- `expect_ident` の予約語診断に `unit` を追加し、`let unit` のような誤用を `parser.identifier.reserved_keyword` として報告する。
- `nodesrc/neplg21_syntax_migrate.js` は unit 型・unit 値・0 引数 marker を `unit` へ変換し、旧 `%()*T>` 形と空白なしの `():` 形も補正する。
- `#intrinsic "..." <> ()` の `()` は unit 値ではなく directive の引数区切りなので、移行ツールで保持・復元する。
- 実行対象 corpus は `unit` 表記へ機械変換した。selfhost parser 用の source string fixture に残る旧 `()` は、selfhost parser の NEPLg2.1 対応単位で扱う。

検証:

- `cargo test -p nepl-core --test functions function_neplg21 -- --nocapture`: passed, 9/9.
- `cargo test -p nepl-core --test typeannot test_neplg21 -- --nocapture`: passed, 6/6.
- `cargo test -p nepl-core --test functions function_neplg21_unit_keyword_marks_zero_arg_signature_and_lambda -- --nocapture`: passed.
- `cargo test -p nepl-core --test typeannot test_neplg21_unit_keyword_type_annotation_and_value -- --nocapture`: passed.
- `trunk build`: passed.
- `node nodesrc/tests.js -i tests/compiler/keywords_reserved.n.md --no-tree -o tmp/neplg21-unit-keyword-reserved.json -j 1 --dist web/dist --assert-io`: passed, 7/7.
- `node nodesrc/neplg21_syntax_migrate.js --check`: passed, would update 0 file(s).
- `node nodesrc/issues.js check --dir issues`: passed.
- `git diff --check`: passed.
