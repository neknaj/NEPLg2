---
id: ISS-20260505T082325239Z-RESOURCEOFFSET-USES-OPTION-INSTEAD-O-84C0E554
title: "ResourceOffset uses Option instead of exact/dynamic enum"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/model.rs,nepl-core/src/resource/**,nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T082325239Z-RESOURCEOFFSET-USES-OPTION-INSTEAD-O-84C0E554: ResourceOffset uses Option instead of exact/dynamic enum

## 概要

Resource IR represents storage offsets as ResourceOffset { bytes: Option<usize> }, so exact zero, exact nonzero, and dynamic/unknown offsets are not a closed semantic enum. This weakens match exhaustiveness in memory-safety checks that must distinguish exact cell aliases from conservative dynamic aliases.

## 対象

- `nepl-core/src/resource/model.rs,nepl-core/src/resource/**,nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceOffset { bytes: Option<usize> }` では `Some(0)`、`Some(n)`、`None` の意味が call site ごとの慣習に依存し、alias 判定・dump・lowering・external I/O effect が `is_none()` や `Option` 比較を直接扱っていた。
- `ISS-20260505T045316820Z-RESOURCE-IR-VEC-RANGE-CLEANUP-9A95DB6F` の range cleanup では、exact offset と dynamic offset を絶対に混同しないことが前提になる。`Option` 表現のままだと、後続の range state 追加時に unknown offset を exact cell fact と同じ分岐へ混ぜる危険がある。

## 問題

Resource IR represents storage offsets as ResourceOffset { bytes: Option<usize> }, so exact zero, exact nonzero, and dynamic/unknown offsets are not a closed semantic enum. This weakens match exhaustiveness in memory-safety checks that must distinguish exact cell aliases from conservative dynamic aliases.

## 影響

Range cleanup and unknown-offset raw memory checks can accidentally grow more ad-hoc Option handling, making it easier to treat one dynamic access as an exact range fact or miss a new offset case during Resource IR evolution.

## 修正方針

Replace the Option field with a ResourceOffset enum that has explicit Exact and Dynamic variants, then update aliasing, dump, lowering, external I/O, and tests to match over the closed enum.

## 検証

Run focused Resource IR tests covering exact offsets, dynamic offsets, unknown-offset preconditions, and source policy/index checks.

## 対応結果

`ResourceOffset` を `Exact(usize)` / `Dynamic` の enum に変更し、Resource IR 内の storage offset 生成・alias 判定・canonical order・dump・external I/O iov cell 判定・テスト fixture をすべて enum variant へ移行した。

これにより dynamic offset は `None` ではなく明示的な `ResourceOffset::Dynamic` として扱われ、exact offset 同士だけが同一 byte の場合に exact alias になる。dynamic offset は引き続き保守的に exact offset と alias し得るため、静的検査の安全側の挙動は弱めていない。

検証:

- `cargo fmt --check -p nepl-core`
- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_summary_rejects_unproven_unknown_offset_non_copy_raw_load -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_mem_ptr_disjoint_offsets -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_region_ptr_at_unknown_offset_rejects_dealloc_over_live_cell -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_literal_arithmetic_helper_zero_offset -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_unknown_arithmetic_helper_offset_conservative -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_fd_pwrite_initializes_nwritten_not_offset -- --nocapture`
