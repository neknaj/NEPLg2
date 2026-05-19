---
id: ISS-20260519T201031870Z-DROP-IMPL-CAN-TARGET-COPY-TYPES-AND--988FC60A
title: "Drop impl can target Copy types and become unreachable"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-20
target: "nepl-core/src/typecheck/driver.rs; nepl-core/src/diagnostic_codes.rs; nepl-core/tests/drop.rs"
---

# ISS-20260519T201031870Z-DROP-IMPL-CAN-TARGET-COPY-TYPES-AND--988FC60A: Drop impl can target Copy types and become unreachable

## 概要

Drop capability represents an owned-resource finalizer, but the typechecker does not reject Drop impls whose target is already Copy or overlaps with a Copy impl target. Resource auto-drop intentionally skips Copy locals, so such Drop impls are accepted but never run automatically.

## 対象

- `nepl-core/src/typecheck/driver.rs; nepl-core/src/diagnostic_codes.rs; nepl-core/tests/drop.rs`

## 根拠

- `nepl-core/src/resource/drop_plan.rs` は end-scope auto-drop candidate から Copy local を除外する。
- `nepl-core/src/typecheck/driver.rs` は従来、Copy impl target の構造制約と Clone 要求だけを検査し、Drop impl target と Copy capability の衝突を検査していなかった。
- `stdlib/core/traits/drop.nepl` の Drop capability は owned resource finalizer であり、Copy capability と同一 target に載せると free/drop obligation の意味が壊れる。

## 問題

Drop capability represents an owned-resource finalizer, but the typechecker does not reject Drop impls whose target is already Copy or overlaps with a Copy impl target. Resource auto-drop intentionally skips Copy locals, so such Drop impls are accepted but never run automatically.

## 影響

A type can be simultaneously copyable and droppable, making free/drop obligations unverifiable and hiding memory-safety bugs behind an unreachable Drop implementation.

## 修正方針

Reject Drop/Copy capability overlap in the impl table before registration. Use a dedicated TypeDiagnosticCode enum variant and type-pattern overlap checks so built-in Copy types, explicit Copy impls, and generic overlaps are all covered.

## 検証

Add Rust integration regression tests for Drop on a built-in Copy target, Copy after Drop, and Drop after Copy; run the focused drop tests and issue validation.

## 対応結果

- `TypeDiagnosticCode::DropImplTargetCopy` を追加し、診断 ID を `type.drop_impl.target_copy` として enum で管理するようにした。
- typecheck の impl registration で Drop capability target を pending check に集め、Clone 要求を満たさず rejected になる Copy impl を取り除いた後、残った Copy impl target と Drop impl target の type-pattern overlap を検査するようにした。
- built-in / bound 由来で既に Copy と証明できる Drop target は `TypeCtx::is_copy` で拒否し、generic `impl<.T> Drop for .T` のように copy impl pattern と重なる target も同じ規則で拒否する。
- rejected Drop impl は impl table 登録と impl method typecheck から除外するため、Drop/Copy 衝突後に `has_drop` へ残らない。

## 回帰テスト

- `drop_impl_rejects_copy_primitive_target`
- `drop_impl_rejects_copy_impl_declared_before_drop`
- `drop_impl_rejects_copy_impl_declared_after_drop`
- `drop_impl_rejects_generic_target_overlapping_copy_impl`

## 検証結果

- `cargo test -p nepl-core --test drop drop_impl_rejects -- --nocapture`: passed
- `cargo test -p nepl-core --test drop -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 copy_impl -- --nocapture`: passed
- `cargo test -p nepl-core diagnostic_codes --lib -- --nocapture`: passed
