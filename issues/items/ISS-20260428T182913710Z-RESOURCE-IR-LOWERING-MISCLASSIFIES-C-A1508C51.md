---
id: ISS-20260428T182913710Z-RESOURCE-IR-LOWERING-MISCLASSIFIES-C-A1508C51
title: "Resource IR lowering misclassifies compiler field reads as raw memory loads"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/compiler.rs, nepl-core/src/resource/shadow.rs, nepl-core/src/resource/mod.rs, nepl-core/tests/resource_ir.rs, tests/compiler/move_check.n.md, tests/compiler/move_effect.n.md"
---

# ISS-20260428T182913710Z-RESOURCE-IR-LOWERING-MISCLASSIFIES-C-A1508C51: Resource IR lowering misclassifies compiler field reads as raw memory loads

## 概要

After typecheck, `core/field::get` is represented as a compiler-generated `load` from an aggregate pseudo-address. Resource IR lowering treated every `load` intrinsic as `RawMemoryOp::Load`, so normal aggregate field reads such as `get p "left"` became raw linear-memory loads and `RawMemoryLoadCell` reported false D3100 on `p.*`.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/compiler.rs, nepl-core/src/resource/shadow.rs, nepl-core/src/resource/mod.rs, nepl-core/tests/resource_ir.rs, tests/compiler/move_check.n.md, tests/compiler/move_effect.n.md`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- 一時 `RawMemoryLoadCell` gate で `tests/compiler/move_check.n.md::doctest#28` と `tests/compiler/move_effect.n.md::doctest#86` - `#88` が、通常 aggregate field read に対して D3100 / D3101 になっていた。

## 問題

After typecheck, `core/field::get` is represented as a compiler-generated `load` from an aggregate pseudo-address. Resource IR lowering treated every `load` intrinsic as `RawMemoryOp::Load`, so normal aggregate field reads such as `get p "left"` became raw linear-memory loads and `RawMemoryLoadCell` reported false D3100 on `p.*`.

## 影響

RawMemoryLoadCell cannot become authoritative: normal type-safe field access and Copy aggregate field reads are blocked by raw memory diagnostics, and the Resource IR checker keeps depending on old HIR move_check to understand aggregate field movement.

## 修正方針

Give Resource IR lowering access to TypeCtx, recognize compiler-generated field load address patterns whose base is an aggregate value, and lower them to ResourceOp::Read from Place::Field/TupleField instead of RawMemoryOp::Load. Keep true raw aggregate loads from i32/MemPtr addresses as raw memory operations.

## 修正内容

- `lower_hir_module` を追加し、compiler pipeline / shadow report では `TypeCtx` を渡した Resource IR lowering を使うようにした。互換用の `lower_hir_module_skeleton` は残した。
- Resource IR lowering は `load(base)` / `load(add(base, literal_offset))` の base が aggregate value で、offset と result 型が aggregate field layout に一致する場合、`RawMemoryOp::Load` ではなく `ResourceOp::Read` from `PlaceProjection::Field` / `TupleField` に下げるようにした。
- lowering coverage も同じ `TypeCtx` 付き分類を使い、compiler-generated field load を raw memory coverage と数えないようにした。
- `resource_ir_lowering_treats_compiler_field_load_as_field_read` を追加し、offset 0 と offset 4 の field pseudo-load が field projection read になること、RawMemory::Load を出さないことを固定した。

## 検証

Add Resource IR regression for lowered field::get pseudo-load; temporarily enable RawMemoryLoadCell gate and confirm the normal field-access doctests stop producing D3100 while remaining raw pointer cases are still tracked; run cargo test -p nepl-core --test resource_ir, cargo check -p nepl-core --tests, trunk build, focused move_effect/move_check doctests, and node nodesrc/issues.js check

- `cargo test -p nepl-core --test resource_ir resource_ir_lowering_treats_compiler_field_load_as_field_read -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 74 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-field-lowering-move-effect.json -j 1`: total=110, passed=110
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\resource-field-lowering-move-check.json -j 1`: total=52, passed=52
- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `node nodesrc\issues.js check`: pass
- 一時 `RawMemoryLoadCell` gate + `tests/compiler/move_check.n.md`: 51/52 から 52/52 に改善
- 一時 `RawMemoryLoadCell` gate + `tests/compiler/move_effect.n.md`: 101/110 から 104/110 に改善。通常 field access 系 `#86` - `#88` は解消し、残り 6 件は MemPtr / RegionToken / raw stored aggregate cell transfer 系。
