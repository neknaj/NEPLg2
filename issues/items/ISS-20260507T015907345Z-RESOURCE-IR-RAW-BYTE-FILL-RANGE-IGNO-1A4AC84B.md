---
id: ISS-20260507T015907345Z-RESOURCE-IR-RAW-BYTE-FILL-RANGE-IGNO-1A4AC84B
title: "Resource IR raw byte fill range ignores length guard"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/effects.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/cell_state.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/mem_fill.n.md"
---

# ISS-20260507T015907345Z-RESOURCE-IR-RAW-BYTE-FILL-RANGE-IGNO-1A4AC84B: Resource IR raw byte fill range ignores length guard

## 概要

RawMemoryOp::Fill represents memset_u8/fill_u8/fill_i32 as one operation and initialized checker records fill as an unbounded unknown-offset Copy cell. As a result byte fills cannot be checked against typed i32 relation facts such as 0 <= i && i < len, and unguarded dynamic byte loads after memset can be accepted as initialized.

## 対象

- `nepl-core/src/effects.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/src/resource/cell_state.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `RawMemoryOp::Fill` が `memset_u8` / `fill_u8` / `fill_i32` / `mem_fill` を一括で表していた。
- `initialized_raw_memory.rs` は `Fill` 後に `raw_memory_unknown_offset_cell_place` を initialized として記録していたため、byte buffer の `base + i` load は `i` が `len` 未満である証明なしでも通過できた。
- `ResourceConditionFact::I32Relation` と `I32ValueCondition::NonNegative` は既に initialized path state に保存されるため、byte offset range の証明に使える状態だった。
- `tests/stdlib/mem_fill.n.md` は raw memory helper を検証するにもかかわらず pure `main` のままで、現在の `effect.pure.calls_impure` gate と合っていなかった。

## 問題

RawMemoryOp::Fill represents memset_u8/fill_u8/fill_i32 as one operation and initialized checker records fill as an unbounded unknown-offset Copy cell. As a result byte fills cannot be checked against typed i32 relation facts such as 0 <= i && i < len, and unguarded dynamic byte loads after memset can be accepted as initialized.

## 影響

This weakens Resource IR RawMemoryLoadCell strictness for byte buffers and hides the dependent range model needed by self-host scanners.

## 修正方針

Split byte fill from word fill in RawMemoryOp, record byte fill as a typed initialized byte range keyed by base and count, and accept symbolic byte-offset loads only when Resource IR condition facts prove the offset is non-negative and below the filled count.

## 検証

Add Resource IR regressions for guarded byte load success and unguarded byte load failure after memset_u8; keep existing raw fill and move_effect regressions passing.

## 2026-05-07 修正内容

`RawMemoryOp::FillBytes` を追加し、`memset_u8` / `fill_u8` / `mem_fill` を byte fill、`fill_i32` を既存 word fill として区別した。byte fill は unbounded unknown-offset cell ではなく、`address`、`count`、cell type を持つ initialized byte range として `CellTable` に記録する。

`RawMemoryLoadCell` は、literal offset では `0 <= offset < count` を count の literal fact から確認し、symbolic offset では `0 <= offset` と `offset < count` が Resource IR condition facts から証明できる場合だけ byte range 由来の initialized cell として扱う。guard なしの symbolic load は従来どおり `resource.cell.uninit` になる。

fill 固有の状態遷移は `initialized_raw_fill.rs` へ分離し、`initialized_raw_memory.rs` の責務分割上限を維持した。

`tests/stdlib/mem_fill.n.md` は raw memory helper 自体を検証する fixture なので、entry function を `()*` effect に更新し、unsafe memory operation を pure function から呼ぶ形にしない。

確認:

- `cargo test -p nepl-core --test resource_ir byte_fill -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill_helpers_initialize_copy_cells -- --nocapture`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/mem_fill.n.md --no-tree --dist web/dist -o tmp/mem_fill_agent1_byte_range_guard.json -j 1 --assert-io`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=120000 node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree --dist web/dist -o tmp/move_effect_agent1_byte_range_guard.json -j 1 --assert-io`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
