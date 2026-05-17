---
id: ISS-20260517T100117040Z-TYPECHECK-CONTROL-SPECIAL-FUNCTIONS--D04B8CA1
title: "typecheck control special functions use direct string branches"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/typecheck/control_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T100117040Z-TYPECHECK-CONTROL-SPECIAL-FUNCTIONS--D04B8CA1: typecheck control special functions use direct string branches

## 概要

Typecheck handles if and while special application by matching HirExprKind::Var names against string literals in control_apply.rs, while prefix_check.rs separately constructs the same Var names from keyword symbols. This duplicates language control spelling in multiple checker consumers instead of routing through a typed domain.

## 対象

- `nepl-core/src/typecheck/control_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 開発方針では、静的検査の分岐は数値や文字列の散在ではなく enum と `match` に寄せ、checker 実装自体の誤りを発見しやすくする必要がある。
- `if` / `while` は source keyword として parser には現れるが、typecheck 内では prefix lowering と control special application が同じ language control spelling を共有する。
- その spelling を複数 consumer の direct string branch に置くと、型検査上の特殊 form 追加や spelling 変更時に片方だけ更新される drift を Rust の網羅性で検出できない。

## 問題

Typecheck handles if and while special application by matching HirExprKind::Var names against string literals in control_apply.rs, while prefix_check.rs separately constructs the same Var names from keyword symbols. This duplicates language control spelling in multiple checker consumers instead of routing through a typed domain.

## 影響

A future control form or spelling adjustment can drift between prefix lowering and special application without Rust enum exhaustiveness catching the mismatch. Because these branches decide type and effect safety for control flow, direct string guards make the static checker implementation itself harder to audit.

## 修正方針

Introduce a ControlSpecialFunction enum with name/from_name mapping, make prefix_check construct control variables from that enum, and make control_apply dispatch through an exhaustive match on the enum. Add source policy checks that reject returning to `name == "if"` / `name == "while"` guards.

## 検証

cargo fmt/check for nepl-core, source responsibility policy, and focused compiler control-flow tests.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-17 解決内容

- `typecheck/control_special.rs` を追加し、`ControlSpecialFunction::{If, While}` が typecheck 内の control special spelling を所有するようにした。
- `prefix_check.rs` は `Symbol::If` / `Symbol::While` から `HirExprKind::Var` を作る際、`ControlSpecialFunction::If.name()` / `ControlSpecialFunction::While.name()` を使うようにした。
- `control_apply.rs` は `HirExprKind::Var(name)` を `ControlSpecialFunction::from_name(name)` で分類し、`ControlSpecialFunction` の `match` で `if` / `while` 特殊適用へ dispatch するようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に `control_special.rs`、enum、canonical name/from_name、`control_apply.rs` の direct `name == "if"` / `name == "while"` 禁止、`prefix_check.rs` の enum 経由 spelling 使用を追加した。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core --test if -- --nocapture`
- `cargo test -p nepl-core --test block_if_semantics -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_returns_concrete_unit_place_for_while -- --exact --nocapture`
