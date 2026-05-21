---
id: ISS-20260521T055400560Z-SOURCE-LEVEL-COLLECTION-SLOT-PROOF-L-B5FB8CDA
title: "Source-level collection slot proof lacks symbolic offset regression"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-21
updated: 2026-05-21
target: nepl-core/src/resource, nepl-core/tests/resource_ir.rs
---

# ISS-20260521T055400560Z-SOURCE-LEVEL-COLLECTION-SLOT-PROOF-L-B5FB8CDA: Source-level collection slot proof lacks symbolic offset regression

## 概要

Compiler-owned stdlib source regressions currently cover zero-offset collection slot raw value-flow proof, but real collection operations use symbolic element offsets derived from indices. Without a source-level symbolic-offset regression, future changes could keep known-offset tests green while breaking generic non-Copy collection payload proof for indexed slots.

## 対象

- `nepl-core/src/resource`
- `nepl-core/tests/resource_ir.rs`

## 根拠

- [ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF](./ISS-20260521T050436552Z-SOURCE-LEVEL-NON-COPY-COLLECTION-SLO-D0BB9BAF.md) で、source-level compiler-owned stdlib lowering の raw store/load fact と collection slot lifecycle target は raw address alias と explicit zero offset を跨いで照合されるようになった。
- [ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9](./ISS-20260521T054050076Z-SOURCE-LEVEL-DROPPABLE-COLLECTION-SL-7041CED9.md) で、droppable slot の actual loaded-value drop proof も source-level lowering 経由で固定した。
- ただし real collection slot は index 由来の symbolic offset を使うため、同じ `off` を複数回 read した時に別 temporary root へ分かれても、scalar origin に基づいて同一 slot として証明できる必要があった。

## 問題

Compiler-owned stdlib source regressions currently cover zero-offset collection slot raw value-flow proof, but real collection operations use symbolic element offsets derived from indices. Without a source-level symbolic-offset regression, future changes could keep known-offset tests green while breaking generic non-Copy collection payload proof for indexed slots.

## 影響

Non-Copy collection push/pop/replace/drop traversal could fail or be incorrectly special-cased when implemented with dynamic slot offsets, blocking self-host collection payload support.

## 修正方針

Add compiler-owned stdlib source Resource IR regressions for non-Copy InitializeEmpty/MoveOut and droppable DropInitialized/ReplaceDropOld using the same symbolic offset for raw store/load and collection slot lifecycle events. Fix alias/proof matching if the regression exposes a mismatch.

## 対応

- `RawCellValueFlowFacts` の store / loaded-value origin proof を記録する時点で、symbolic storage offset を `RawCellAddressAliases` の scalar origin へ正規化するようにした。
- `RawCellValueFlow` の alias-aware proof matching でも symbolic offset を同じ正規化に通し、同じ source local から複数回 read された offset を同一 proof として扱うようにした。
- `CollectionSlotStateTable` へ渡る lifecycle target も同じ canonical symbolic offset へ揃え、`InitializeEmpty` 後の `MoveOut` / `DropInitialized` / `ReplaceDropOld` が temporary 名の違いで `Uninitialized` に戻らないようにした。
- `resource_ir_collection_slot_source_symbolic_offset_move_out_accepts_value_flow_proof` と `resource_ir_collection_slot_source_symbolic_offset_drop_and_replace_accept_drop_proofs` を追加し、indexed slot の positive proof を固定した。
- `resource_ir_collection_slot_source_symbolic_offset_rejects_mismatched_value_flow` を追加し、`off + size_of<T>` のように別 scalar local へ移った offset は同じ slot proof として扱わないことを固定した。
- stdlib module 名や helper 名の allowlist は追加していない。source lowering された scalar alias / raw address alias / typed lifecycle event の generic proof boundary だけで証明する。

## 検証

- `cargo check -p nepl-core`: passed
- `cargo fmt --check`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_collection_slot_source_symbolic -- --test-threads=1`: passed
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`: passed
- `cargo test -p nepl-core raw_cell_value_flow -- --test-threads=1`: passed
