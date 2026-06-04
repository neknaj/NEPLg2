---
id: ISS-20260604T034256529Z-SELFHOST-PARSER-MIXES-CURRENT-PERCEN-3647B103
title: "selfhost parser mixes current percent type syntax with legacy paren and angle syntax boundaries"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl, stdlib/neplg2/core/syntax/parser/module_parser/action.nepl, stdlib/neplg2/core/syntax/parser/module_parser/header_boundary.nepl, stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl, stdlib/neplg2/core/proof/solver/module.nepl"
---

# ISS-20260604T034256529Z-SELFHOST-PARSER-MIXES-CURRENT-PERCEN-3647B103: selfhost parser mixes current percent type syntax with legacy paren and angle syntax boundaries

## 概要

Subagent audit found LParen still classified as expression start and declaration parsing still oriented around legacy angle/generic paths while Percent is only partially connected. This conflicts with the current NEPLg2.1 prefix type syntax and the user requirement that parentheses are not used in NEPLg2 source.

## 対象

- `stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl, stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl`

## 根拠

- `doc/neplg2/self_host_neplg21_compiler_design.md` は、NEPLg2.1 の正規構文を `%` type annotation / prefix type / `void` zero-argument marker とし、括弧 grouping と旧 angle type syntax を正規構文から外している。
- `stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl` は `LParen` を expression start として扱っていた。
- `stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl` は `LAngle` を declaration head の `GenericParams` evidence として後段へ渡していた。

## 問題

Subagent audit found LParen still classified as expression start and declaration parsing still oriented around legacy angle/generic paths while Percent is only partially connected. This conflicts with the current NEPLg2.1 prefix type syntax and the user requirement that parentheses are not used in NEPLg2 source.

## 影響

Parser and syntax tooling can disagree about whether % type annotations, legacy angle forms, and parentheses belong to current source, producing stale examples or incorrect range diagnostics.

## 修正方針

Unify token predicates and declaration parsing around current % type syntax, move legacy forms to explicit compatibility or compile-fail diagnostics, and document migration boundaries.

## 検証

Add regular tests for % type annotation ranges, function type ranges, legacy angle rejection, LParen rejection in current source, and diagnostic spans.

## 対応

- `LParen` は expression start ではないと分類し、`Percent` は current type annotation / expression ascription の開始候補として維持した。
- module parser action に `LegacySyntax` を追加し、`LParen` / `RParen` / `LAngle` / `RAngle` が通常 source に現れた場合は `SelfhostParserDiagnosticCode::LegacySyntaxToken` と stable code `parser.syntax.legacy_token` を返すようにした。
- declaration header の `GenericParams` evidence を削除し、旧 `<...>` generic parameter list を後段 proof solver の authority として渡さないようにした。
- `%fn ... fn ...` の内部 `fn` を top-level declaration と誤分類しないよう、statement boundary / `pub` modifier boundary 判定を `module_parser/header_boundary.nepl` へ分離した。
- parser positive fixture を `fn add %fn i32 fn i32 i32 \a\b:` へ更新し、旧 angle syntax / parenthesized grouping の拒否、primary label span、current header span を doctest で固定した。
- `nodesrc/test_selfhost_parser_current_syntax_boundary.js` を追加し、typed diagnostic code、legacy token action、`LParen=false` / `Percent=true`、`GenericParams` absence、fixture drift を source policy で検出する。
