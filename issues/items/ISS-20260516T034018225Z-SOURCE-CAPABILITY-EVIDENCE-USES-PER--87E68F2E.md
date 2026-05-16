---
id: ISS-20260516T034018225Z-SOURCE-CAPABILITY-EVIDENCE-USES-PER--87E68F2E
title: "Source capability evidence uses per-domain collectors instead of a unified proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/source_capability/**"
---

# ISS-20260516T034018225Z-SOURCE-CAPABILITY-EVIDENCE-USES-PER--87E68F2E: Source capability evidence uses per-domain collectors instead of a unified proof

## 概要

Source capability proof traversal was shared, but raw memory and owner aggregate capability checks still instantiate separate collectors and re-walk the same AST. This keeps the proof model split by capability domain and makes it easier to add another local proof engine later.

## 対象

- `nepl-core/src/source_capability/**`

## 根拠

- `nepl-core/src/loader.rs` が raw memory / owner aggregate / compiler memory type それぞれの collector API を個別に呼び、SourceCapabilities を組み立てていた。
- `source_capability/walk.rs` に共通 traversal は入っていたが、raw memory と owner aggregate は別 collector と別 API を保持していたため、domain ごとに proof lifecycle が分裂していた。
- [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 は、stdlib allowlist や module 固有証明ではなく、compiler の汎用 source proof で capability を導く方針である。

## 問題

Source capability proof traversal was shared, but raw memory and owner aggregate capability checks still instantiate separate collectors and re-walk the same AST. This keeps the proof model split by capability domain and makes it easier to add another local proof engine later.

## 影響

Static-check authority becomes harder to audit: each capability domain can drift in traversal lifecycle, source scope treatment, or proof event interpretation. The design goal is a generic typed prover that reads source once and exposes evidence through enum-backed queries.

## 修正方針

Introduce a unified source capability proof value with typed evidence buckets, collect it through the shared SourceCapabilityObserver traversal once, and make raw memory / owner aggregate capability APIs query that proof instead of owning independent collectors.

## 解決内容

- `nepl-core/src/source_capability/proof.rs` を追加し、`SourceCapabilityProof` を raw memory evidence、owner aggregate evidence、compiler memory type evidence の単一 typed proof value として導入した。
- `SourceCapabilityProofCollector` が `SourceCapabilityObserver` を実装し、`walk_module_capability_evidence` を一度だけ消費して raw helper call-head、raw body、intrinsic、owner aggregate constructor、owner aggregate field accessor evidence を同じ traversal lifecycle で集めるようにした。
- `loader.rs` は `module_source_capabilities(module)` だけを呼ぶ形に変更し、loader 側から per-domain collector の組み立て責務を削除した。
- `raw_memory.rs` と `owner_aggregate.rs` は traversal owner ではなく、typed evidence classifier / context を unified proof へ渡す責務に縮小した。
- `nodesrc/test_static_check_boundary_responsibility.js` に、raw memory / owner aggregate module が `SourceCapabilityObserver` を直接実装しないこと、loader が旧 per-domain collector を呼ばないこと、proof module が単一 collector を持つことの regression を追加した。

## 検証

- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo fmt -p nepl-core -- --check`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
- `cargo test -p nepl-core owner_aggregate_boundary -- --nocapture`
