---
id: ISS-20260517T102052427Z-TYPECHECK-UNIT-LIKE-STRUCT-SHAPE-IS--F13DA6CD
title: "typecheck unit-like struct shape is duplicated by string checks"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/model.rs"
---

# ISS-20260517T102052427Z-TYPECHECK-UNIT-LIKE-STRUCT-SHAPE-IS--F13DA6CD: typecheck unit-like struct shape is duplicated by string checks

## 概要

Typecheck classifies unit-like structs by repeating the same tag field string and positional indexing in driver.rs and constructor_apply.rs. The constructor function arity and constructor lowering can drift because the shape is not stored as a typed compiler fact.

## 対象

- `nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/constructor_apply.rs, nepl-core/src/typecheck/model.rs`

## 根拠

- 開発方針では、静的検査の実装自体も文字列や添字に依存した分岐を避け、enum と `match` で網羅性が効く形にする必要がある。
- `tag <()>` による unit-like struct constructor は、constructor 関数登録時の arity と、constructor 適用時の HIR field 補完の両方に関わる。
- 同じ shape 判定を別々の箇所で再実装すると、constructor signature と constructor lowering が drift しても compiler 自身の型で検出できない。

## 問題

Typecheck classifies unit-like structs by repeating the same tag field string and positional indexing in driver.rs and constructor_apply.rs. The constructor function arity and constructor lowering can drift because the shape is not stored as a typed compiler fact.

## 影響

A future struct shape rule can update the registered constructor signature without updating constructor application, or the reverse, leaving static checking dependent on duplicated string checks instead of an enum/match domain.

## 修正方針

Introduce a typed StructConstructorShape classification, store it in StructInfo, and make both constructor signature registration and constructor application consume that shape.

## 検証

cargo fmt/check for nepl-core, focused constructor/unit-like struct tests, and source policy checks that reject direct tag string/index shape checks outside the classifier.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-17 解決内容

- `typecheck/struct_shape.rs` を追加し、`StructConstructorShape::{UnitLikeTag, FieldList}` が struct constructor shape を分類するようにした。
- `StructInfo` に `constructor_shape` を保持し、driver の constructor 関数登録と constructor application が同じ shape fact を参照するようにした。
- `driver.rs` から `f_names[0] == "tag"`、`constructor_apply.rs` から `field_names[0] == "tag"` の重複判定を削除した。
- `StructConstructorShape` の model unit test と `nodesrc/test_static_check_boundary_responsibility.js` の source policy を追加し、直接添字・文字列判定への退行を監視するようにした。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core typecheck::struct_shape::tests --lib -- --nocapture`
- `cargo test -p nepl-core --test overload unit_like_struct_constructor_is_a_value -- --exact --nocapture`
- `cargo test -p nepl-core --test overload grouped_argument_overload_uses_later_items_before_reduction -- --exact --nocapture`
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --exact --nocapture`
- `cargo test -p nepl-core --test neplg2 stdlib_reimported_definition_does_not_warn_same_signature_shadow -- --exact --nocapture`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `node nodesrc/issues.js check --dir issues`
- `trunk build`
