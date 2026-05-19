---
id: ISS-20260519T012026321Z-RAW-OPERATION-SOURCE-PROOF-ACCEPTS-M-D53D8575
title: "raw operation source proof accepts mismatched function body evidence"
area: compiler/resource-ir
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-19
updated: 2026-05-19
target: nepl-core/src/source_capability/top_level_raw_calls.rs
---

# ISS-20260519T012026321Z-RAW-OPERATION-SOURCE-PROOF-ACCEPTS-M-D53D8575: raw operation source proof accepts mismatched function body evidence

## 概要

Raw operation propagation records a function-level boundary from the function name, but only checks that the body has some direct raw evidence. A stdlib-owned helper named for one raw operation can therefore propagate that operation even if the body evidence is for a different raw operation.

## 対象

- `nepl-core/src/source_capability/top_level_raw_calls.rs`

## 根拠

- `collect_top_level_raw_call_evidence` は `RawOperationBoundaryContract` を関数名から導き、`RawOperationFunctionEvidence::has_direct_raw_evidence()` が true ならその operation を伝搬していた。
- `has_direct_raw_evidence()` は operation の種類を見ていなかったため、`load_i32` 名の helper が `store_i32` だけを呼ぶ場合でも `Load` boundary を得られた。
- raw body evidence でも同じ問題があり、`#wasm: i32.store` だけを持つ `load_i32` が `Load` boundary として扱われ得た。

## 問題

Raw operation propagation records a function-level boundary from the function name, but only checks that the body has some direct raw evidence. A stdlib-owned helper named for one raw operation can therefore propagate that operation even if the body evidence is for a different raw operation.

## 影響

This weakens Stage 6 static-check authority: raw-memory operation permission is no longer a proof that the source body performs the same operation, and future helper refactors can accidentally grant an unrelated raw operation boundary.

## 修正方針

Make raw operation propagation consume typed evidence for the same RawMemoryOp as the function boundary contract. Raw body memory instructions must be mapped to compatible RawMemoryOp variants instead of satisfying all contracts.

## 検証

Add Rust regression tests that a load-named helper with only store evidence does not grant load, while matching direct/raw-body evidence still grants the expected operation.

## 対応結果

- `RawOperationFunctionEvidence::supports_operation` を追加し、function boundary の operation と直接 evidence / raw-body evidence の typed operation が一致または明示的に互換な場合だけ function-level proof を成立させるようにした。
- byte fill / i32 fill / allocator growth / deallocation metadata write のような既存 stdlib raw helper の実装形は、operation-level compatibility として enum `match` に閉じ込めた。
- `Realloc` は単一の `Alloc` call だけでは証明せず、top-level propagation で `Alloc` と `Dealloc` の双方が証明済みの場合だけ成立させた。
- regression として direct mismatch、raw-body mismatch、compatible fill、matching raw-body、composite realloc を追加した。

## 2026-05-19 修正完了

- `cargo test -p nepl-core raw_memory_function_boundary -- --nocapture`: pass
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/run_source_policy_regressions.js --warn-only`: exit 0
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
