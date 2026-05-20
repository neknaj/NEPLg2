---
id: ISS-20260520T180237730Z-INITIALIZED-RAW-RANGE-COUNT-ALIASES--A1FBF011
title: "Initialized raw range count aliases are not preserved through loaded count values"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/cell_state_raw_range_value_alias.rs, nepl-core/src/resource/cell_state_raw_range_cover_tests.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260520T180237730Z-INITIALIZED-RAW-RANGE-COUNT-ALIASES--A1FBF011: Initialized raw range count aliases are not preserved through loaded count values

## 概要

The focused unit test byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local currently fails. A raw byte range whose count is represented by a raw memory cell should remain usable after the count is loaded through a temporary into a local, but raw range coverage cannot prove the later i < used guard against the copied count.

## 対象

- `nepl-core/src/resource/cell_state_raw_range_value_alias.rs, nepl-core/src/resource/cell_state_raw_range_cover_tests.rs`

## 根拠

- `cargo test -p nepl-core cell_state -- --test-threads=1` で `resource::cell_state_raw_range_cover_tests::byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local` が失敗した。
- `CellTable::copy_initialized_raw_byte_ranges_through_value_aliases` は range count の value copy を扱うが、raw memory cell から temporary/local に読み出した count と guarded symbolic offset の証明が結合できていない。

## 問題

The focused unit test byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local currently fails. A raw byte range whose count is represented by a raw memory cell should remain usable after the count is loaded through a temporary into a local, but raw range coverage cannot prove the later i < used guard against the copied count.

## 影響

The checker can reject valid raw-memory access patterns that load a length or used count from storage before guarding an indexed access. This is a false negative, but it can block selfhost-style buffer code and hides an incompleteness in the generic i32/raw range proof path.

## 修正方針

Review raw range count value alias propagation and scalar alias integration so initialized range count evidence follows value copies and raw loads through the same generic proof machinery. Keep the check source-derived; do not whitelist specific stdlib modules or functions.

## 検証

The unit test byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local must pass, together with related raw range cover tests and Resource IR raw memory regressions.
