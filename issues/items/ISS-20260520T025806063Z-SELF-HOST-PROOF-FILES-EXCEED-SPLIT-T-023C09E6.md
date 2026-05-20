---
id: ISS-20260520T025806063Z-SELF-HOST-PROOF-FILES-EXCEED-SPLIT-T-023C09E6
title: "self-host proof files exceed split threshold after Stage 6 expansion"
area: selfhost
status: open
resolved: false
priority: P2
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/query.nepl, stdlib/neplg2/core/proof/fact.nepl, stdlib/neplg2/core/proof/api.nepl, nodesrc/test_selfhost_proof_entry_contract.js"
---

# ISS-20260520T025806063Z-SELF-HOST-PROOF-FILES-EXCEED-SPLIT-T-023C09E6: self-host proof files exceed split threshold after Stage 6 expansion

## 概要

Stage 6 proof work concentrated many domains in core/proof/solver.nepl, query.nepl, fact.nepl, and api.nepl. solver.nepl now exceeds the source-tree split threshold, and continuing to add proof domains there will recreate a flat proof engine.

## 対象

- `stdlib/neplg2/core/proof/solver.nepl, stdlib/neplg2/core/proof/query.nepl, stdlib/neplg2/core/proof/fact.nepl, stdlib/neplg2/core/proof/api.nepl, nodesrc/test_selfhost_proof_entry_contract.js`

## 根拠

- 2026-05-20 時点の `stdlib/neplg2/core/proof/solver.nepl` は 959 行で、`doc/neplg2/self_host_source_tree_layout_review_20260518.md` の pass implementation 目安である 900 行を超えている。
- `query.nepl` は 490 行、`fact.nepl` は 469 行、`api.nepl` は 418 行で、query / evidence / refutation / projection helper / domain payload が同じ file に集まり始めている。
- Stage 6 proof architecture は generic proof engine として妥当だが、このまま新しい Resource IR / typecheck / abstraction proof を追加すると、Rust 側の flat `parser.rs` / `types.rs` / `compiler.rs` と同じ監査困難な構造を self-host に再導入する。
- 分割しても、checker や resource module が個別証明器を持つ設計にはしない。domain-specific rule は private solver rule として generic `SelfhostProofQuery -> SelfhostProofResult` 境界に残す。

## 問題

Stage 6 proof work concentrated many domains in core/proof/solver.nepl, query.nepl, fact.nepl, and api.nepl. solver.nepl now exceeds the source-tree split threshold, and continuing to add proof domains there will recreate a flat proof engine.

## 影響

Future type/resource/effect/abstraction proofs become harder to audit, and match exhaustiveness remains technically present but spread across files that are too large to review safely.

## 修正方針

Split proof model and solver code by responsibility: domain model, fact/obligation/evidence/refutation payloads, public API projection, solver entry/dispatch, and domain-specific proof rules. Keep facade imports stable only as implementation-free boundaries.

## 検証

Run selfhost proof entry contract and focused proof doctests after the split.
