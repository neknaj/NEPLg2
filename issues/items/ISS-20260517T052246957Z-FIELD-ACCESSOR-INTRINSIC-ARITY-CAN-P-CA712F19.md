---
id: ISS-20260517T052246957Z-FIELD-ACCESSOR-INTRINSIC-ARITY-CAN-P-CA712F19
title: "field accessor intrinsic arity can panic before diagnostics"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: nepl-core/src/typecheck/prefix_check.rs
---

# ISS-20260517T052246957Z-FIELD-ACCESSOR-INTRINSIC-ARITY-CAN-P-CA712F19: field accessor intrinsic arity can panic before diagnostics

## 概要

typecheck/prefix_check.rs routes get_field/get_field_ref/set_field through FieldAccessorKind, but the special lowering still indexes args[0], args[1], and args[2] before checking the intrinsic argument count. Malformed user source can crash the compiler instead of producing a structured intrinsic arity diagnostic.

## 対象

- `nepl-core/src/typecheck/prefix_check.rs`

## 根拠

- `FieldAccessorKind` は field accessor intrinsic spelling の typed domain になっていたが、各 intrinsic の必要引数数は enum 側の契約になっていなかった。
- `prefix_check.rs` は `Get` / `GetRef` / `Put` の分岐内で `args[0]` / `args[1]` / `args[2]` を直接参照しており、`#intrinsic "set_field" <> (p,"x")` のような malformed source で diagnostic を出す前に panic し得た。
- 静的検査では user source の誤りを structured diagnostic として扱う必要があり、compiler crash は検査の正確性と信頼性を損なう。

## 問題

typecheck/prefix_check.rs routes get_field/get_field_ref/set_field through FieldAccessorKind, but the special lowering still indexes args[0], args[1], and args[2] before checking the intrinsic argument count. Malformed user source can crash the compiler instead of producing a structured intrinsic arity diagnostic.

## 影響

Compiler crashes are unacceptable for static checking: a malformed field accessor intrinsic bypasses typed diagnostics and makes the checker less trustworthy. The expected arity is also not part of the FieldAccessorKind domain, so future variants can be added without a compiler-enforced arity contract.

## 修正方針

- `FieldAccessorKind::argument_count()` を追加し、field accessor intrinsic の arity contract を enum domain に所属させる。
- `prefix_check.rs` は field accessor lowering の `match` に入る前に `field_accessor.argument_count()` で `args.len()` を検査し、不一致なら `TypeDiagnosticCode::IntrinsicArgArityMismatch` を発行して generic intrinsic expression recovery に進む。
- model unit test と source policy で、arity contract が `FieldAccessorKind` から外へ戻らないよう固定する。
- `set_field` 引数不足の integration regression を追加し、panic ではなく type diagnostic になることを確認する。

## 検証

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core field_accessor_intrinsic --lib -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test neplg2 field_accessor_intrinsic_arg_arity_has_type_code -- --nocapture`: 1 passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: CRLF warnings only
