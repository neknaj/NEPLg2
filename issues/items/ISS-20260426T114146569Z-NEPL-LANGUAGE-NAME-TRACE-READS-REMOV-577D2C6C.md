---
id: ISS-20260426T114146569Z-NEPL-LANGUAGE-NAME-TRACE-READS-REMOV-577D2C6C
title: "nepl-language name trace reads removed MatchArm.bind field"
area: core
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-language/src/lib.rs, nepl-core/src/ast.rs"
---

# ISS-20260426T114146569Z-NEPL-LANGUAGE-NAME-TRACE-READS-REMOV-577D2C6C: nepl-language name trace reads removed MatchArm.bind field

## 概要

cargo check --workspace fails because nepl-language/src/lib.rs still reads arm.bind, but nepl-core MatchArm now stores payload bindings inside MatchPattern::Variant { bind }.

## 対象

- `nepl-language/src/lib.rs, nepl-core/src/ast.rs`

## 根拠

- `cargo check --workspace` が `nepl-language/src/lib.rs:1253` の `arm.bind` 参照で `E0609 no field bind on type &MatchArm` を返した。
- `nepl-core/src/ast.rs` では match arm binding は `MatchArm` 直下ではなく `MatchPattern::Variant { bind }` に格納される。

## 問題

cargo check --workspace fails because nepl-language/src/lib.rs still reads arm.bind, but nepl-core MatchArm now stores payload bindings inside MatchPattern::Variant { bind }.

## 影響

Workspace-level Rust checks fail before warning debt can be measured, hiding regressions outside nepl-core/nepl-cli.

## 修正方針

Read match-arm bindings from arm.pattern and only define match_bind for variant patterns that carry a bind identifier.

## 検証

cargo check -p nepl-language

## 対応

- `nepl-language/src/lib.rs` の name resolution trace で `MatchPattern` を import し、variant pattern の `bind` だけを `match_bind` として scope に定義するようにした。
- literal / wildcard arm では binding を作らないため、現在の AST 仕様と一致する。

## 検証結果

- `cargo check -p nepl-language`: pass（既存 `nepl-core` warnings は残存）
- `cargo check --workspace`: pass（既存 warnings は残存）
- `cargo test -p nepl-language`: pass（3 passed）
- `node nodesrc/issues.js index` / `node nodesrc/issues.js check`: pass
- `cargo fmt --all --check`: pass
- `trunk build`: pass（既存 warnings は残存）
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-nepl-language-match-bind.json`: `13/13 passed`
- `git diff --check`: pass（issue index と `nepl-language/src/lib.rs` の CRLF warning のみ）
