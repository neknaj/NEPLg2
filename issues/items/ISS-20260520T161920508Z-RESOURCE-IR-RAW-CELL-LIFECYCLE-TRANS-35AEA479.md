---
id: ISS-20260520T161920508Z-RESOURCE-IR-RAW-CELL-LIFECYCLE-TRANS-35AEA479
title: "Resource IR raw cell lifecycle transitions are scattered across checker operations"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/resource/cell_state*.rs, nepl-core/src/resource/initialized_raw_memory*.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260520T161920508Z-RESOURCE-IR-RAW-CELL-LIFECYCLE-TRANS-35AEA479: Resource IR raw cell lifecycle transitions are scattered across checker operations

## 概要

Raw cell initialized/moved/released transitions are currently assembled by direct CellTable mutation at each raw load/store/fill/bulk/realloc/dealloc site. This weakens the Stage 6 goal: non-Copy collection payload support needs a generic initialized/moved/drop cell proof boundary, not duplicated operation-local mutation sequences.

## 対象

- `nepl-core/src/resource/cell_state*.rs, nepl-core/src/resource/initialized_raw_memory*.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) の Stage 6 は、raw memory provenance、initialized cell、moved/drop state を Resource IR 上の共有状態として扱うことを完了条件にしている。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection slot support を stdlib 個別証明ではなく compiler core の generic cell lifecycle proof に接続することを要求している。

## 問題

Raw cell initialized/moved/released transitions are currently assembled by direct CellTable mutation at each raw load/store/fill/bulk/realloc/dealloc site. This weakens the Stage 6 goal: non-Copy collection payload support needs a generic initialized/moved/drop cell proof boundary, not duplicated operation-local mutation sequences.

## 影響

Future non-Copy Vec/List/Map payload support can accidentally update one raw operation path but miss another, reintroducing shallow copy, stale initialized ranges, or storage-only dealloc of live owner cells. The checker implementation itself is also harder to audit with enum/match exhaustiveness.

## 修正方針

Introduce a typed raw-cell lifecycle transition boundary in compiler core. Raw memory checker operations should request lifecycle events such as move-out, store-initialize, storage release, copy initialized Copy cells, and realloc transfer through an enum/struct API, keeping module-specific proof engines out of stdlib.

## 検証

Add focused Resource IR regressions and source policy checks showing raw load/store/fill/bulk/realloc/dealloc use the lifecycle boundary and still reject double move, live non-Copy overwrite, non-Copy fill range initialization, and live-cell storage destruction.

## 対応結果

2026-05-20 に修正済み。

- `RawCellLifecycleEvent` を追加し、raw load/store/fill/bulk/realloc/dealloc の cell transition を `match` による enum 境界へ集約した。
- 非Copy raw load が cell を `Moved` にするとき、同じ raw address 範囲に残る initialized cell entry と initialized raw byte range を同時に破棄するようにした。これにより、古い initialized evidence が後続 load を再び初期化済みに見せる経路を閉じた。
- store 後の再初期化は lifecycle event を通して `Moved` state を上書きし、正当な replace / re-store を許可する。
- `nodesrc/test_resource_raw_cell_lifecycle_policy.js` で raw memory operation が lifecycle 境界を迂回しないことを監視する。

## 回帰テスト

- `cargo test -p nepl-core cell_state::tests --lib`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_moves_non_copy_raw_load_cell -- --test-threads=1 --exact`
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_store_reinitializes_moved_raw_cell -- --test-threads=1 --exact`
- `node nodesrc/test_resource_raw_cell_lifecycle_policy.js`
