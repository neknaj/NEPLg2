---
id: ISS-20260517T094754743Z-SOURCE-CAPABILITY-PROOF-BUILDER-EXPO-C06F3A22
title: "Source capability proof builder exposes per-domain insert APIs"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T094754743Z-SOURCE-CAPABILITY-PROOF-BUILDER-EXPO-C06F3A22: Source capability proof builder exposes per-domain insert APIs

## 概要

SourceCapabilityProof is collected through a unified traversal, but the builder still exposes insert_raw_memory_*, insert_owner_aggregate_evidence, and insert_compiler_memory_* methods. This keeps proof emission coupled to individual capability domains and weakens the enum/match exhaustiveness pressure requested for the static-check redesign.

## 対象

- `nepl-core/src/source_capability/proof_builder.rs, nepl-core/src/source_capability/rule.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 静的検査大規模修正 Stage 6 では、source から導出した証明を file-level / module-level allowlist ではなく exact use-site proof artifact として扱う。
- `SourceCapabilityProofEvent` により traversal と event dispatch は統合されたが、proof builder に domain 別 insert method が残ると、raw memory / owner aggregate / compiler memory field の write path が再び分岐し、追加 domain の写像漏れを Rust の `match` で検出しにくい。
- 証明器自体の誤りを発見しやすくするには、source evidence classifier が `SourceCapabilityProofFact` を発行し、builder が fact enum を exhaustive match で `SourceCapabilityUseSite` へ写像する形にする必要がある。

## 問題

SourceCapabilityProof is collected through a unified traversal, but the builder still exposes insert_raw_memory_*, insert_owner_aggregate_evidence, and insert_compiler_memory_* methods. This keeps proof emission coupled to individual capability domains and weakens the enum/match exhaustiveness pressure requested for the static-check redesign.

## 影響

Future source capability domains can add bespoke insertion paths or drift from SourceCapabilityUseSite mapping without one typed proof fact match catching the design error. That works against the Stage 6 goal of a generic proof artifact pipeline and makes the checker implementation itself harder to audit statically.

## 修正方針

Introduce a SourceCapabilityProofFact enum for all source-derived proof facts, make SourceCapabilityProof consume facts through one exhaustive match, and route existing raw memory, raw body, owner aggregate, compiler memory field, and compiler memory type evidence through that typed fact API. Add source policy checks that reject reintroduced per-domain insert APIs.

## 検証

cargo fmt/check for nepl-core, source capability policy, and focused source_map/source capability tests.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 2026-05-17 解決内容

- `SourceCapabilityProofFact` enum を追加し、raw memory structural boundary、raw address view boundary、raw memory operation、raw body operation、owner aggregate field / constructor、compiler memory field、compiler memory type definition を 1 つの proof fact domain に集約した。
- `SourceCapabilityProof::insert_fact` が `SourceCapabilityProofFact` を exhaustive match し、`SourceCapabilityUseSite` へ写像するようにした。
- `insert_raw_memory_*`、`insert_owner_aggregate_evidence`、`insert_compiler_memory_*` などの domain 別 proof builder API を削除した。
- owner aggregate / compiler memory field の evidence-to-fact 変換を `source_capability/fact.rs` に分離し、`rule.rs` は source event から fact を発行する責務に寄せた。
- top-level raw helper call propagation は `SourceCapabilities` を直接更新せず、`SourceCapabilityProofFact::RawMemoryOperationBoundary` を同じ `insert_fact` path に流すようにした。
- `nodesrc/test_static_check_boundary_responsibility.js` に、`SourceCapabilityProofFact` / `insert_fact` / fact conversion module / 旧 domain 別 insert API の再導入禁止を追加した。

検証:

- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core source_map::tests --lib -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary --lib -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundary --lib -- --nocapture`
- `cargo test -p nepl-core compiler_memory_field_boundary --lib -- --nocapture`
