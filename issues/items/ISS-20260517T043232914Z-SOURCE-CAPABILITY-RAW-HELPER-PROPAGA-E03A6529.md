---
id: ISS-20260517T043232914Z-SOURCE-CAPABILITY-RAW-HELPER-PROPAGA-E03A6529
title: "source capability raw helper propagation still exposes operation authority from helper name"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/top_level_raw_calls.rs; nepl-core/src/source_capability/proof.rs; nepl-core/src/loader.rs"
---

# ISS-20260517T043232914Z-SOURCE-CAPABILITY-RAW-HELPER-PROPAGA-E03A6529: source capability raw helper propagation still exposes operation authority from helper name

## 概要

SourceCapability top-level raw call propagation still derives the externally exposed RawMemoryOp from raw_memory_op_from_name(frame.name). That keeps one proof edge tied to helper spelling instead of an explicit source-derived proof artifact, making the static-check proof harder to audit and easier to extend with per-helper exceptions.

## 対象

- `nepl-core/src/source_capability/top_level_raw_calls.rs; nepl-core/src/source_capability/proof.rs; nepl-core/src/loader.rs`

## 根拠

- `source_capability/top_level_raw_calls.rs` の worklist が `raw_memory_op_from_name(&frame.name)` を直接呼び、source proof frame の内部に「どの raw evidence が直接観測されたか」と「外部へ露出できる raw operation contract」が型として残っていなかった。
- `SourceCapabilityProofCollector` は exact use-site proof を集める単一 traversal になっていたが、top-level raw helper propagation だけが `has_direct_raw_evidence: bool` と helper spelling の再分類で proof を消費していた。

## 問題

SourceCapability top-level raw call propagation still derives the externally exposed RawMemoryOp from raw_memory_op_from_name(frame.name). That keeps one proof edge tied to helper spelling instead of an explicit source-derived proof artifact, making the static-check proof harder to audit and easier to extend with per-helper exceptions.

## 影響

Raw memory boundary authority can drift back toward stdlib/helper-name conventions. The exact-use-site proof migration remains less robust because the verifier cannot inspect a typed function proof object that says which raw operations were actually proven inside the function.

## 修正方針

Represent raw helper source proof as a typed function proof containing direct raw operations and an explicit classified boundary contract. Propagate call-site authority from this proof object, not by re-querying raw helper names inside the worklist. Add regressions covering source-derived operations and non-raw helper wrappers.

## 検証

Run targeted loader source capability tests, cargo fmt/check for nepl-core, static boundary policy, issues check, and git diff --check.

## 解決内容

2026-05-17 に修正した。

- `source_capability/raw_operation_proof.rs` を追加し、raw helper propagation の入力を `RawOperationBoundaryContract` と `RawOperationFunctionEvidence` に分離した。
- `RawOperationFunctionEvidence` は direct raw operation evidence と raw body operation evidence を typed set として保持する。worklist は `has_direct_raw_evidence()` を見るだけで、helper 名を再分類しない。
- `RawOperationBoundaryContract` は source proof collection 側で一度だけ分類し、`top_level_raw_calls.rs` は contract と evidence の proof object だけを消費する。
- `nodesrc/test_static_check_boundary_responsibility.js` に、top-level raw call propagation が `raw_memory_op_from_name` を再導入しないこと、typed contract/evidence module を使うことを固定した。

## 検証結果

- `cargo test -p nepl-core raw_memory_boundary_ --lib`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo check -p nepl-core`: passed
