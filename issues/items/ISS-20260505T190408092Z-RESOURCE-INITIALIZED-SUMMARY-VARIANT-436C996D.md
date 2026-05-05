---
id: ISS-20260505T190408092Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-436C996D
title: "Resource initialized summary variant builder exceeds responsibility split limit again"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T190408092Z-RESOURCE-INITIALIZED-SUMMARY-VARIANT-436C996D: Resource initialized summary variant builder exceeds responsibility split limit again

## 概要

After initialized_summary.rs was reduced to the data contract, the direct Resource checker responsibility policy reaches initialized_summary_variant_build.rs and reports 337 lines over the 260-line limit. Variant-gated initialized summary construction has accumulated condition collection, requirement collection, payload path traversal, and uniqueness helpers again.

## 対象

- `nepl-core/src/resource/initialized_summary_variant_build.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `initialized_summary.rs` 分割後、次の未解決責務違反として `initialized_summary_variant_build.rs has 337 lines; responsibility split limit is 260` を報告する。
- `ISS-20260430T062125921Z-RESOURCE-INITIALIZED-SUMMARY-BUILDER-2875F09C` で variant builder は一度分離済みだが、その後の Result/Option branch gating、condition collection、raw load requirement collection が増え、variant builder 自体が再び上限を超えた。
- Stage 4 Resource check では variant-gated initialized summary が branch-specific memory-safety fact を担うため、path traversal、condition extraction、requirement collection、deduplication の境界を再確認する必要がある。

## 問題

After initialized_summary.rs was reduced to the data contract, the direct Resource checker responsibility policy reaches initialized_summary_variant_build.rs and reports 337 lines over the 260-line limit. Variant-gated initialized summary construction has accumulated condition collection, requirement collection, payload path traversal, and uniqueness helpers again.

## 影響

Variant-gated initialized summaries determine which Result/Option branches make raw cells initialized or required. If this builder remains concentrated, memory-safety checks can become difficult to audit and regressions in branch-specific initialization facts can hide behind a monolithic helper.

## 修正方針

Split initialized_summary_variant_build.rs by responsibility, such as variant path traversal, condition extraction, requirement collection, and uniqueness helpers, while preserving exact Result/Option gated summary semantics.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused initialized summary Resource IR tests.
