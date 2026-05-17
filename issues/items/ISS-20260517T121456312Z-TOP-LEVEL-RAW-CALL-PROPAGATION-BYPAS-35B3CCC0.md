---
id: ISS-20260517T121456312Z-TOP-LEVEL-RAW-CALL-PROPAGATION-BYPAS-35B3CCC0
title: "Top-level raw call propagation bypasses source capability proof event dispatcher"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/top_level_raw_calls.rs, nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T121456312Z-TOP-LEVEL-RAW-CALL-PROPAGATION-BYPAS-35B3CCC0: Top-level raw call propagation bypasses source capability proof event dispatcher

## 概要

Source capability top-level raw call propagation computes cross-function raw operation evidence and inserts SourceCapabilityProofFact directly from top_level_raw_calls.rs. This bypasses SourceCapabilityProofEvent dispatch and leaves one proof-emission path outside the generic rule engine.

## 対象

- `nepl-core/src/source_capability/top_level_raw_calls.rs, nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `source_capability/rule.rs` には `SourceCapabilityProofEvent` と `dispatch_source_capability_proof_event` があり、source proof の event coverage を enum / match で監査できる設計にしている。
- 一方で `source_capability/top_level_raw_calls.rs` は propagation worklist の最後に `SourceCapabilityProofFact::RawMemoryOperationBoundary` を直接 `SourceCapabilityProof::insert_fact` していた。
- これにより、top-level raw helper call propagation だけが generic proof event dispatcher の外側に proof fact emission path を持っていた。

## 問題

Source capability top-level raw call propagation computes cross-function raw operation evidence and inserts SourceCapabilityProofFact directly from top_level_raw_calls.rs. This bypasses SourceCapabilityProofEvent dispatch and leaves one proof-emission path outside the generic rule engine.

## 影響

A proof fact can be emitted by a domain-specific propagation module without going through the typed event dispatcher. Future source capability domains or proof event changes can miss this bypass, weakening Stage 6 checker-auditability and the requirement that static-check code itself is easy to verify through enum/match structure.

## 修正方針

Make top_level_raw_calls.rs return typed propagated raw-operation evidence only. Add a SourceCapabilityProofEvent variant for propagated raw operation evidence and route the final proof insertion through dispatch_source_capability_proof_event. Add policy that top_level_raw_calls.rs does not import SourceCapabilityProof or SourceCapabilityProofFact.

## 関連計画

- [静的検査の不必要な複雑化の解消についての大規模な修正の仕様と実装計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対応内容

- `apply_top_level_raw_call_evidence` を `collect_top_level_raw_call_evidence` に変更し、top-level raw call propagation module は `PropagatedRawOperationEvidence` の計算だけを行うようにした。
- `SourceCapabilityProofEvent::PropagatedRawOperation` を追加し、propagation 後の raw operation proof fact emission も `dispatch_source_capability_proof_event` の exhaustive `match event` に戻した。
- `top_level_raw_calls.rs` から `SourceCapabilityProof` / `SourceCapabilityProofFact` への依存を削除した。
- static-check responsibility policy に、propagation evidence が typed dispatcher を通ることと、top-level module が proof builder mutation API / proof fact enum に依存しないことを追加した。

## 検証

- `cargo fmt -p nepl-core --check`: pass
- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core raw_memory_boundary_accepts_proven_top_level_raw_helper_call_evidence --lib -- --nocapture`: pass
- `cargo test -p nepl-core raw_memory_boundary_rejects_unproven_top_level_raw_helper_call_evidence --lib -- --nocapture`: pass
- `cargo test -p nepl-core raw_memory_boundary_accepts_raw_helper_definition_evidence --lib -- --nocapture`: pass
- `cargo test -p nepl-core raw_memory_boundary_keeps_raw_helper_body_evidence_function_scoped --lib -- --nocapture`: pass
- `node nodesrc/test_static_check_boundary_responsibility.js`: pass
- `node nodesrc/issues.js check --dir issues`: pass
