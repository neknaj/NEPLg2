---
id: ISS-20260520T165035652Z-RAW-FILL-LIFECYCLE-EVENT-CONFLATES-C-3A87C0FB
title: "Raw fill lifecycle event conflates Copy element proof with destructive discard"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-21
target: "nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs"
source: "doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行"
---

# ISS-20260520T165035652Z-RAW-FILL-LIFECYCLE-EVENT-CONFLATES-C-3A87C0FB: Raw fill lifecycle event conflates Copy element proof with destructive discard

## 概要

RawCellLifecycleEvent::FillCopyElements can be constructed for non-Copy payloads and then silently avoids creating initialized range evidence. This keeps current safety but represents a failed Copy proof as a successful event with no postcondition instead of making Copy evidence part of the event type.

## 対象

- `nepl-core/src/resource/raw_cell_lifecycle.rs, nepl-core/src/resource/initialized_raw_fill.rs, nepl-core/tests/resource_ir.rs`

## 根拠

`RawCellLifecycleEvent::FillCopyElements` が `TypeId` だけを持ち、handler 内部の `types.is_copy(value_ty)` 分岐で postcondition を決めていた。これは non-Copy の `TypeId` でも Copy-element fill event を構築できる設計であり、event の型から「Copy 証明済み」が読み取れなかった。

## 問題

RawCellLifecycleEvent::FillCopyElements can be constructed for non-Copy payloads and then silently avoids creating initialized range evidence. This keeps current safety but represents a failed Copy proof as a successful event with no postcondition instead of making Copy evidence part of the event type.

## 影響

The checker program itself is harder to audit: callers can request a Copy-element fill without carrying a typed Copy proof, and correctness depends on a hidden branch inside the lifecycle handler. That weakens enum/match based static verification and can hide future non-Copy slot lifecycle bugs.

## 修正方針

Split the lifecycle variants so Copy-element fill requires typed Copy evidence before construction, while non-Copy fill/destructive overwrite is represented as a separate discard-only event or a diagnostic path.

## 検証

Add source-policy and Resource IR regressions proving non-Copy fill cannot enter the Copy-element lifecycle variant, while Copy fill still creates initialized range evidence.

## 2026-05-21 修正

Copy-element fill event が `TypeId` を直接受け取る設計をやめ、`CopyRawElementType` を導入した。`CopyRawElementType::new` は `TypeCtx::is_copy` が成立したときだけ `Some` を返すため、`FillCopyElements` variant は Copy 証明済みの element type なしには構築できない。

修正内容:

- `RawCellLifecycleEvent::FillCopyElements` の payload を `value_ty: TypeId` から `element_ty: CopyRawElementType` に変更した。
- lifecycle handler 内の hidden `if types.is_copy(...)` を削除し、`match` で到達した時点で initialized element range を作る設計にした。
- `check_raw_memory_fill_words` で Copy 証明が取れた場合のみ `FillCopyElements` を構築し、non-Copy の場合は `DiscardCellsUnderAddress` に分岐する。
- `CopyRawElementType` には stride を保持し、range unit の byte stride も Copy 証明済み payload から取り出すようにした。

追加した回帰:

- `copy_raw_element_type_requires_copy_evidence`
- `resource_ir_cell_check_word_fill_non_copy_value_does_not_create_range_evidence`

既存回帰:

- `resource_ir_cell_check_word_fill_accepts_scaled_symbolic_load_with_range_guard`

検証:

- `cargo test -p nepl-core raw_cell_lifecycle -- --test-threads=1`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill_non_copy_value_does_not_create_range_evidence -- --test-threads=1 --exact`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_word_fill_accepts_scaled_symbolic_load_with_range_guard -- --test-threads=1 --exact`: passed
