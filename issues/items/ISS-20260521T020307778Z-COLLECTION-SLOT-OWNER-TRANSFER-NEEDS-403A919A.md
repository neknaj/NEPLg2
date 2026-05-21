---
id: ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A
title: "Collection slot owner transfer needs local raw value-flow proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/**"
---

# ISS-20260521T020307778Z-COLLECTION-SLOT-OWNER-TRANSFER-NEEDS-403A919A: Collection slot owner transfer needs local raw value-flow proof

## 概要

CollectionSlotLifecycle currently rejects every non-Copy owner-transfer event because the event has no way to prove that a raw store or raw load actually consumed or materialized the payload. This is safe but blocks the generic Resource IR path needed for non-Copy collection payloads.

## 対象

- `nepl-core/src/resource/**`

## 根拠

- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を stdlib 個別 allowlist ではなく Resource IR の generic proof boundary に載せることを要求している。
- 直前の [ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2](./ISS-20260521T010410090Z-COLLECTION-SLOT-OWNER-TRANSFER-LIFEC-3C1056B2.md) では、payload value-flow evidence が存在しないため non-Copy owner-transfer lifecycle event を拒否する安全側 gate を追加した。
- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 は、Resource IR の fact / obligation / evidence / refutation を enum / match で扱い、stdlib module 名に依存しないことを完了条件にしている。

## 問題

CollectionSlotLifecycle currently rejects every non-Copy owner-transfer event because the event has no way to prove that a raw store or raw load actually consumed or materialized the payload. This is safe but blocks the generic Resource IR path needed for non-Copy collection payloads.

## 影響

Self-host AST/HIR/diagnostic collections cannot move owning payloads through collection slots without either staying Copy-only or reintroducing shallow raw-memory owner transfer.

## 修正方針

Record typed raw StoreValue and MoveOutLoadedCell facts in the Resource IR cell state, merge them path-sensitively, and require collection slot owner-transfer events to consume the matching local raw value-flow proof instead of using stdlib allowlists.

## 検証

Add focused Resource IR regressions for non-Copy slot initialization and move-out with and without matching raw value-flow proof, then run the resource_ir collection slot tests and issue validation.

## 2026-05-21 修正内容

- `RawCellValueFlowFacts` を追加し、raw memory `StoreValue` / `MoveOutLoadedCell` が non-Copy raw cell の local value-flow proof を typed fact として記録するようにした。
- fact は `CellTable` に保持し、branch / loop / match merge では全 path に共通する fact だけを残す。片側 branch だけで store/load された fact は合流後の collection slot proof には使えない。
- stale store proof を raw load で消すようにし、`store -> load -> initialize` のような古い store fact による誤証明を防いだ。一方で `load old -> store new -> replace_return_old` のように old move-out と new store が両方必要な event は、2 種類の fact を同時に要求できる。
- collection slot owner-transfer obligation は `CollectionSlotOwnerTransferObligation` enum として分離し、`InitializeEmpty` は store proof、`MoveOut` は load proof、`ReplaceReturnOld` は old load / new store の必要十分な組み合わせ、`ReplaceDropOld` は new store proof を要求する。
- Copy payload は従来どおり state-only marker として扱う。non-Copy payload は local raw value-flow fact を消費できる場合だけ state transition を許可する。
- collection slot summary replay の branch / loop / indirect call は、`CollectionSlotStateTable` だけでなく `CellTable` も path ごとに clone / merge する。これにより、callee summary replay 中に片方の path でだけ得た raw value-flow fact が別 path の owner-transfer proof として漏れることを防ぐ。

## 残件

- callee 内で証明済みの non-Copy slot lifecycle を caller へ伝える certified summary proof はまだ未実装。親 issue の残件として、`CollectionSlotLifecycleFunctionSummary` に proof evidence を保持する必要がある。
- droppable payload の `DropInitialized` / `ReplaceDropOld` は引き続き compiler-owned slot-drop lowering が必要で、state-only drop では通さない。

## 2026-05-21 検証

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core resource_ir_collection_slot --test resource_ir -- --test-threads=1`: pass
- `cargo test -p nepl-core raw_value_flow --lib -- --test-threads=1`: pass
- `cargo test -p nepl-core stale_store --lib -- --test-threads=1`: pass
- `cargo test -p nepl-core collection_slot --lib -- --test-threads=1`: pass
- `cargo fmt --check -p nepl-core`: pass
- `node nodesrc/issues.js check --dir issues`: pass
- `git diff --check`: pass
