---
id: ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F
title: "move_check doctests are stale after Resource IR field projection gates"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-06
target: "nepl-core/src/typecheck/field_apply.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage_hir_projection.rs, tests/compiler/move_check.n.md"
---

# ISS-20260506T022705566Z-MOVE-CHECK-DOCTESTS-ARE-STALE-AFTER--EDD8402F: move_check doctests are stale after Resource IR field projection gates

## 概要

Focused move_check doctests after the Resource IR gate migration still failed because legacy resource.move/resource.borrow diag_code expectations were stale, and field::get_ref lowering could lose initialized field state for compiler-lowered reference projections.

## 対象

- `nepl-core/src/typecheck/field_apply.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_aggregate.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage_hir_projection.rs, tests/compiler/move_check.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -o output/move_check_after_resource_projection.json --runner wasm --no-tree -j 1` で 52 件中 37 passed / 15 failed。
- `field::get_ref &p "count"` は Resource IR dump 上で `raw_address_view tmp6[+4] -> tmp8` として表現され、後続 `*tmp8` が `resource.cell.uninit` になっていた。
- `field::get_ref &p "token"` の offset 0 経路は `&Pair` の式 kind を `&LocalToken` 型として再利用しており、typed HIR の式意味と型がずれていた。
- `Result::Err` arm が `Never` を返す match でも、その path の cell state が initialized-state merge に参加し、到達不能 path が到達可能 path を汚染していた。

## 問題

Focused move_check doctests after the Resource IR gate migration still failed because legacy resource.move/resource.borrow diag_code expectations were stale, and field::get_ref lowering could lose initialized field state for compiler-lowered reference projections.

## 影響

Stage 4 cannot treat Resource IR as the static-check authority while field reference projection can be represented as untyped raw address arithmetic or while move_check fixtures assert legacy diagnostic buckets.

## 修正方針

Keep get_ref as typed get_field_ref intrinsic at typecheck, lower reference field address projections to Resource IR Borrow operations, exclude Never-valued control-flow arms from initialized-state merges, update HIR coverage and move_check diag expectations to Resource IR cell taxonomy.

## 検証

cargo check -p nepl-core --tests; cargo test -p nepl-core --test resource_ir resource_ir_field_get_ref_deref_uses_borrowed_field_cell -- --nocapture; cargo test -p nepl-core --test resource_ir resource_ir_match_never_arm_does_not_poison_initialized_state -- --nocapture; trunk build; node nodesrc/tests.js -i tests/compiler/move_check.n.md -o output/move_check_after_diag_refresh.json --runner wasm --no-tree -j 1; node nodesrc/tests.js -i tests/compiler/move_effect.n.md -o output/move_effect_after_projection_fix.json --runner wasm --no-tree -j 1

## 対応結果

- `typecheck/field_apply.rs` の `get_ref` lowering を typed `get_field_ref` intrinsic に戻し、offset 0 field reference が owner 全体の `AddrOf` を field reference 型で再利用しないようにした。
- Resource IR lowering は `get_ref` / `get_field_ref` と compiler-lowered `add &owner offset` reference projection を `ResourceOp::Borrow` へ下げ、raw address view と initialized cell state を混同しないようにした。
- HIR/Resource IR coverage は reference field projection を borrow + deref projection として数えるようにし、coverage gate を弱めずに新しい lowering と一致させた。
- `Never` 型の branch / match value は initialized-state merge path から除外し、到達不能 arm が reachable path の cell state を汚染しないようにした。
- `tests/compiler/move_check.n.md` は legacy `resource.move.*` / `resource.borrow.*` bucket ではなく Resource IR authority の `resource.cell.*` code を期待する形へ更新した。

## 検証結果

- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_field_get_ref_deref_uses_borrowed_field_cell -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_match_never_arm_does_not_poison_initialized_state -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -o output/move_check_after_diag_refresh.json --runner wasm --no-tree -j 1`: total=52, passed=52
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md -o output/move_effect_after_projection_fix.json --runner wasm --no-tree -j 1`: total=110, passed=110
