---
id: ISS-20260430T053723149Z-MOVE-EFFECT-REALLOC-FIXTURES-IGNORE--B08BEB6E
title: "move_effect realloc fixtures ignore fallible realloc ownership contract"
area: core
status: fixed
resolved: true
priority: P1
type: test
created: 2026-04-30
updated: 2026-04-30
target: "tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs"
---

# ISS-20260430T053723149Z-MOVE-EFFECT-REALLOC-FIXTURES-IGNORE--B08BEB6E: move_effect realloc fixtures ignore fallible realloc ownership contract

## 概要

move_effect.n.md still treats realloc_raw as an unconditional successful move. Current Resource IR correctly marks the realloc result as conditional until the caller checks success, so doctest#8 reports resource.cell.uninit before identity_escape and doctest#54 reports resource.owner.maybe_freed.

## 対象

- `tests/compiler/move_effect.n.md, nepl-core/tests/resource_ir.rs`

## 根拠

- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/current-move-effect-after-trunk.json -j 1` で `total=110`, `passed=108`, `failed=2` だった。
- doctest#8 は `realloc_raw` の戻り値を成功判定せずに `load_i32 grown` しており、Resource IR が `resource.cell.uninit` を出すのは妥当だった。
- doctest#54 は `realloc_raw` の戻り値を成功判定せずに `dealloc_raw q 32` しており、Resource IR が `resource.owner.maybe_freed` を出すのは妥当だった。
- `nepl-core/tests/resource_ir.rs` の `resource_ir_owner_check_refines_realloc_result_branches` は、`lt 0 grown` で success/failure を分ける契約を既に固定している。

## 問題

move_effect.n.md still treats realloc_raw as an unconditional successful move. Current Resource IR correctly marks the realloc result as conditional until the caller checks success, so doctest#8 reports resource.cell.uninit before identity_escape and doctest#54 reports resource.owner.maybe_freed.

## 影響

The focused move/effect regression file fails even though the compiler-side Resource IR tests pass, hiding real memory-safety regressions behind stale fixture expectations.

## 修正方針

Update realloc fixtures to branch on the realloc result before loading or deallocating, preserving the intended raw identity escape and owner-transfer checks without weakening Resource IR.

## 対応結果

- doctest#8 は `realloc_raw slot 4 8` の結果を `lt 0 grown` で分岐し、成功時は `grown`、失敗時は旧 `slot` から raw slot payload を読む形にした。
- doctest#54 は `realloc_raw p size_of<LocalToken> 32` の結果を `lt 0 q` で分岐し、成功時は `q`、失敗時は旧 `p` を dealloc する形にした。
- compiler の Resource IR / owner check を弱めず、fixture 側を fallible realloc contract に合わせた。

## 検証

- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-realloc-fixtures-fixed.json -j 1`: `total=110`, `passed=110`
- `cargo test -p nepl-core --test resource_ir realloc -- --nocapture`: `2 passed`
