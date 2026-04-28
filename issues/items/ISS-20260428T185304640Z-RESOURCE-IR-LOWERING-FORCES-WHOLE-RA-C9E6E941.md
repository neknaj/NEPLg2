---
id: ISS-20260428T185304640Z-RESOURCE-IR-LOWERING-FORCES-WHOLE-RA-C9E6E941
title: "Resource IR lowering forces whole raw aggregate loads before field projection"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/typecheck/field_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/passes/move_check/provenance.rs, nepl-core/src/passes/move_check/visitor.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/move_check.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260428T185304640Z-RESOURCE-IR-LOWERING-FORCES-WHOLE-RA-C9E6E941: Resource IR lowering forces whole raw aggregate loads before field projection

## 概要

Raw aggregate storage field access such as `get load<Holder> p "ptr"` was lowered through a whole `load<Holder> p` before the field projection. This erased the aggregate field projection from Resource IR, caused `RawMemoryLoadCell` to see a whole non-Copy raw load, and made Copy field reads look like raw aggregate moves.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/typecheck/field_apply.rs, nepl-core/src/typecheck/prefix_check.rs, nepl-core/src/passes/move_check/provenance.rs, nepl-core/src/passes/move_check/visitor.rs, nepl-core/tests/resource_ir.rs, nepl-core/tests/move_check.rs, tests/compiler/move_effect.n.md`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- 一時 `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md::doctest#76` - `#79` が raw aggregate の Copy field read 後に whole aggregate load D3100 になっていた。

## 問題

Typecheck lowered raw aggregate field access to `load(add(raw_addr, field_offset))`, so Resource IR could no longer distinguish "read Copy field from raw aggregate cell" from "move a whole raw memory value". Resource IR lowering and coverage also lacked a source-level `get` / `get_field` path for raw aggregate field projection, and old `move_check` still visited the preserved `get_field(load<Aggregate> ...)` base as a whole raw load.

## 影響

RawMemoryLoadCell cannot become authoritative for raw aggregate storage: Copy field reads from raw aggregate cells fail before the intended field-level move/ownership diagnostics, so old HIR move_check remains necessary for raw aggregate projection semantics.

## 修正方針

Lower compiler field projection over raw aggregate load directly to the raw cell field place, e.g. p.*.field, without emitting a whole-aggregate RawMemoryOp::Load for the base expression. Keep true whole aggregate loads unchanged.

## 修正内容

- typecheck の field get lowering で、raw aggregate load に対する field access は `load(add(...))` へ潰さず、`get_field(load<Aggregate> addr, selector)` として aggregate context を保持するようにした。
- Resource IR lowering は source-level `get` / `get_field` を raw aggregate field projection として解釈し、`raw_addr.Deref.Field` の `ResourceOp::Read` へ下げるようにした。
- Resource IR coverage も同じ分類を使い、raw aggregate field read の deref projection を HIR coverage として数えるようにした。
- CellState の raw alias canonical は temporary より local / return / storage を優先し、store 側と field read 側が別の代表 place へ割れる問題を避けた。
- old `move_check` は preserved `get_field(load<Aggregate> ...)` と source-level `get` call を raw aggregate field projection として扱い、Copy field では whole raw place を move せず、non-Copy field では offset 付き raw place だけを move するようにした。
- `resource_ir` と `move_check` に回帰テストを追加し、Resource IR lowering、coverage、old move_check compatibility を固定した。

## 検証

Add Resource IR lowering regression for field projection over raw aggregate load; temporarily enable RawMemoryLoadCell gate and confirm move_effect raw aggregate Copy field doctests no longer fail while focused resource_ir tests pass.

- `rustfmt --check nepl-core/src/passes/move_check/provenance.rs nepl-core/src/passes/move_check/visitor.rs nepl-core/src/resource/coverage.rs nepl-core/src/resource/lower.rs nepl-core/src/resource/initialized_alias.rs nepl-core/src/typecheck/field_apply.rs nepl-core/src/typecheck/prefix_check.rs nepl-core/tests/move_check.rs nepl-core/tests/resource_ir.rs`: pass
- `cargo test -p nepl-core --test move_check move_raw_aggregate -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 76 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-raw-aggregate-field-final-move-effect.json -j 1`: total=110, passed=110
- 一時 `RawMemoryLoadCell` gate + `tests/compiler/move_effect.n.md`: 101/110 から 105/110 に改善し、`#76` - `#79` の raw aggregate field projection false D3100 は解消。残り 5 件は親 issue の raw pointer summary / load-cell gate 残件。
