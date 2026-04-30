---
id: ISS-20260430T143236266Z-RESOURCE-IR-TYPED-RAW-STORE-LOAD-LOS-711B058B
title: "Resource IR typed raw store/load loses initialized enum cell after generic store"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs, tests/compiler/intrinsic.n.md"
---

# ISS-20260430T143236266Z-RESOURCE-IR-TYPED-RAW-STORE-LOAD-LOS-711B058B: Resource IR typed raw store/load loses initialized enum cell after generic store

## 概要

tests/compiler/intrinsic.n.md doctest#5 stores Result<(),i64> into raw storage and immediately loads Result<(),i64>, but Resource IR reports resource.cell.uninit at RawMemoryLoadCell. The initialized raw cell is keyed by exact Place/TypeId identity, so structurally identical generic instantiations can fail to match.

## 対象

- `nepl-core/src/resource/cell_state.rs, nepl-core/src/resource/initialized_raw_memory.rs, nepl-core/tests/resource_ir.rs, tests/compiler/intrinsic.n.md`

## 根拠

- `tests/compiler/intrinsic.n.md` doctest#5 で `store<Result<(),i64>> p r` の直後に同じ型引数の `load<Result<(),i64>> p` を行うと、Resource IR が `RawMemoryLoadCell` の対象 cell を `Uninit` と判定していた。
- `CellTable::availability_state` は raw cell の projection 一致を見ていたが、`Place` / `CellState::Initialized(TypeId)` の照合で exact `TypeId` を要求していたため、構造的に同じ generic instantiation が別 `TypeId` として現れた時に初期化済み状態が流れなかった。
- true load-before-store は引き続き `resource_ir_cell_check_reports_raw_load_before_store` で拒否されるため、今回の修正は gate の緩和ではなく型同値な initialized cell の照合精度改善である。

## 問題

tests/compiler/intrinsic.n.md doctest#5 stores Result<(),i64> into raw storage and immediately loads Result<(),i64>, but Resource IR reports resource.cell.uninit at RawMemoryLoadCell. The initialized raw cell is keyed by exact Place/TypeId identity, so structurally identical generic instantiations can fail to match.

## 影響

Valid typed raw memory code is rejected by the strict RawMemoryLoadCell gate. Weakening the gate would hide real load-before-store bugs, so the checker must preserve initialized-state precision for type-equivalent raw cells.

## 修正方針

Keep RawMemoryLoadCell strict. Make raw-cell availability lookup compare raw cell entries under the same canonical address with TypeCtx::same_type, so only type-equivalent store/load pairs reuse initialized state. Add a focused Resource IR regression and keep true load-before-store diagnostics intact.

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_type_equivalent_generic_raw_store_load -- --nocapture; cargo test -p nepl-core --test resource_ir resource_ir_cell_check_reports_raw_load_before_store -- --nocapture; cargo check -p nepl-core; trunk build; node nodesrc/tests.js -i tests/compiler/intrinsic.n.md --no-tree -o tmp/intrinsic-typed-raw-store-load-agent1.json -j 1 --dist web/dist; node nodesrc/issues.js check

## 対応

- `CellTable::availability_state` と branch / loop / match merge に `TypeCtx::same_type` を渡し、raw cell の initialized state だけを型同値で照合するようにした。
- non-initialized state の descendant flow は維持しているため、moved / dropped / maybe moved の拒否は引き続き raw cell projection に対して保守的に働く。
- `resource_ir_cell_check_preserves_type_equivalent_generic_raw_store_load` を追加し、generic enum payload の typed raw store/load が false `resource.cell.uninit` にならないことを固定した。
