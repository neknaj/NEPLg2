---
id: ISS-20260428T175617166Z-RESOURCE-CELLSTATE-EXPRESSION-MARKER-26479BD3
title: "Resource CellState expression markers clear call and aggregate raw aliases"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: nepl-core/src/resource/initialized.rs
---

# ISS-20260428T175617166Z-RESOURCE-CELLSTATE-EXPRESSION-MARKER-26479BD3: Resource CellState expression markers clear call and aggregate raw aliases

## 概要

Resource IR lowering emits semantic operations such as Call, IndirectCall, Construct, Branch, and Match followed by ResourceExprKind marker ops for the expression result. CellState propagates raw address aliases in the semantic op, then clears the same output when it sees the marker. Real HIR lowering therefore loses helper-returned slot and aggregate-field aliases even though manual ResourceOp tests without markers pass.

## 対象

- `nepl-core/src/resource/initialized.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 親 issue: [ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E](./ISS-20260428T170745661Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-N-1CBE1D0E.md)
- `RawMemoryLoadCell` gate を一時的に有効化した確認で、手動 ResourceOp regression は通るのに実際の HIR lowering では `slot_id slot` の直後に alias が消え、doctest#13/#14 が D3025 ではなく false D3100 になっていた。

## 問題

Resource IR lowering emits semantic operations such as Call, IndirectCall, Construct, Branch, and Match followed by ResourceExprKind marker ops for the expression result. CellState propagates raw address aliases in the semantic op, then clears the same output when it sees the marker. Real HIR lowering therefore loses helper-returned slot and aggregate-field aliases even though manual ResourceOp tests without markers pass.

## 影響

RawMemoryLoadCell remains blocked from authoritative Stage 4 enforcement: correct raw slot programs fail with false D3100 before D3025/effect diagnostics, and the previous raw address summary fix does not apply to actual lowered HIR.

## 修正方針

Treat ResourceExprKind marker ops for semantic ResourceOps as non-clearing markers in CellState and raw alias summary propagation. Keep clearing for literal/intrinsic/function-value/borrow-like outputs that introduce fresh non-address values.

## 修正内容

- `ResourceExprKind::Call` / `IndirectCall` / `Construct` / `Branch` / `Match` は、直前の semantic `ResourceOp` が設定した raw address alias を消さない marker として扱うようにした。
- raw alias return summary の計算側も同じ marker 規則を使い、summary と実チェックで alias の保持条件がずれないようにした。
- 既存の helper-returned raw address / function-value returned raw address / aggregate field raw address regression に、実際の HIR lowering と同じ `Expr` marker を追加した。

## 検証

Add Resource IR regressions with Expr(Call) and Expr(Construct) markers after the semantic ops, run cargo test -p nepl-core --test resource_ir, cargo check -p nepl-core --tests, trunk build, and move_effect/move_check focused doctests.

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_raw_address -- --nocapture`: 3 passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 73 passed
- `cargo check -p nepl-core --tests`: pass
- temporary `RawMemoryLoadCell` gate で `tests/compiler/move_effect.n.md`: 99/110 から 101/110 に改善し、doctest#13/#14 の false D3100 が解消したことを確認
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\resource-marker-production-move-effect.json -j 1`: total=110, passed=110
- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\resource-marker-production-move-check.json -j 1`: total=52, passed=52
