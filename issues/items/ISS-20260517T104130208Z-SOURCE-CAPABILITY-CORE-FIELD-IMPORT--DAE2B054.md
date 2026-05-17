---
id: ISS-20260517T104130208Z-SOURCE-CAPABILITY-CORE-FIELD-IMPORT--DAE2B054
title: "source capability core field import proof uses ad hoc path string matching"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/owner_aggregate/field_imports.rs, nepl-core/src/source_capability.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T104130208Z-SOURCE-CAPABILITY-CORE-FIELD-IMPORT--DAE2B054: source capability core field import proof uses ad hoc path string matching

## 概要

Owner aggregate field source proof recognizes core/field imports by stripping only .nepl and comparing a raw string inside field_imports.rs. This keeps import module classification as an ad hoc string check instead of a typed source capability domain.

## 対象

- `nepl-core/src/source_capability/owner_aggregate/field_imports.rs, nepl-core/src/source_capability.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 開発方針では、静的検査の証明器が個別箇所の文字列条件に依存せず、enum と `match` で分類 domain を明示する必要がある。
- `core/field` import は owner aggregate field accessor の source evidence に直結し、証明可能性を左右する。
- import path の正規化と module 分類を `field_imports.rs` の局所文字列比較に置くと、resolver の import path 形式や `.n.md` source 形式との差分を静的に監視しにくい。

## 問題

Owner aggregate field source proof recognizes core/field imports by stripping only .nepl and comparing a raw string inside field_imports.rs. This keeps import module classification as an ad hoc string check instead of a typed source capability domain.

## 影響

Field accessor source evidence can drift from import resolution rules, for example with normalized paths or n.md module forms, and the proof program cannot use enum exhaustiveness to track supported source capability import modules.

## 修正方針

Introduce a SourceCapabilityImportModule enum that owns import path normalization and core/field classification, and make field import evidence consume that typed module classifier.

## 検証

cargo fmt/check for nepl-core, source capability unit tests for import path normalization, source responsibility policy, and focused loader/source capability regressions.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-17 解決内容

- `source_capability/import_path.rs` を追加し、`SourceCapabilityImportModule::CoreField` が proof-relevant import module classification を所有するようにした。
- `SourceCapabilityImportModule::from_path` が slash 正規化、`.` / `..` component 正規化、`.nepl` / `.n.md` extension 正規化を行うようにした。
- `owner_aggregate/field_imports.rs` は `SourceCapabilityImportModule::from_path(path)` を消費するだけにし、局所的な `strip_suffix(".nepl").unwrap_or(path) == "core/field"` 判定を削除した。
- `nodesrc/test_static_check_boundary_responsibility.js` に import path module、enum、classification、extension normalization、field import consumer、旧 path string check 禁止を追加した。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core source_capability::import_path::tests --lib -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundary_accepts_field --lib -- --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
