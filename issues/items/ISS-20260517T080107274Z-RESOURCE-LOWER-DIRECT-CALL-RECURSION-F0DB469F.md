---
id: ISS-20260517T080107274Z-RESOURCE-LOWER-DIRECT-CALL-RECURSION-F0DB469F
title: "Resource lower direct call recursion duplicates field accessor spelling"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/resource/lower.rs; nepl-core/src/intrinsic_kinds.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260517T080107274Z-RESOURCE-LOWER-DIRECT-CALL-RECURSION-F0DB469F: Resource lower direct call recursion duplicates field accessor spelling

## 概要

Resource IR lowering の direct_call_needs_recursive_lowering が get/get_ref/get_field/get_field_ref を直接列挙し、FieldAccessorKind の shared classifier を使っていない。

## 対象

- `nepl-core/src/resource/lower.rs; nepl-core/src/intrinsic_kinds.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `FieldAccessorKind::from_call_base_name` は field accessor の source member spelling と intrinsic spelling を同じ enum domain で分類する。
- `resource/lower.rs` の direct call recursion gate だけが `get` / `get_ref` / `get_field` / `get_field_ref` を直接列挙していた。

## 問題

Resource IR lowering の direct_call_needs_recursive_lowering が get/get_ref/get_field/get_field_ref を直接列挙し、FieldAccessorKind の shared classifier を使っていない。

## 影響

field accessor spelling の追加や変更時に lowering recursion gate だけが古い列挙に残り、field accessor の内部 lowering が他の Resource IR classifier と drift する。

## 修正方針

direct_call_needs_recursive_lowering を FieldAccessorKind::from_call_base_name に接続し、直接文字列列挙を責務検査で禁止する。

## 対応内容

- `direct_call_needs_recursive_lowering` を `FieldAccessorKind::from_call_base_name` に接続し、`Get` / `GetRef` を enum match で再帰 lowering 対象にするようにした。
- `get` / `get_ref` / `get_field` / `get_field_ref` の direct string 列挙を削除した。
- Resource checker responsibility policy に、`lower.rs` が `FieldAccessorKind` を使うことと direct spelling 列挙を戻さないことを追加した。
- 関連計画: [doc/neplg2/static_check_complexity_reduction_plan.md](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 検証

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core field_accessor_call_base_name --lib -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
