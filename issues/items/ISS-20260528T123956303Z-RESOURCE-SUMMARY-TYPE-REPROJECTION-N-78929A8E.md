---
id: ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E
title: "Resource summary type reprojection needs instantiated generic nominal mapping"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-28
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/types.rs"
---

# ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E: Resource summary type reprojection needs instantiated generic nominal mapping

## 概要

RPN same-session code edit reports raw_init_param_facts_reprojection_bypasses=10. Subagent review identified that ResourceSummaryTypeReprojection may register generic nominal Apply trees using definition-side type variables instead of instantiated signature mappings.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/types.rs`

## 根拠

- `ISS-20260528T110220373Z-RESOURCE-SUMMARY-CACHE-NEEDS-QUALIFI-08D1AA04` の verified 測定で、`raw_init_param_facts_reprojection_bypasses=10` が残った。
- subagent review では、`ResourceSummaryTypeReprojection` が `Apply` の base definition tree と args を別々に登録するため、generic nominal definition 側の `.T` と signature instantiation 側の `.T` が別 `TypeId` として衝突し得ると指摘された。
- `TypeCtx::nominal_stable_identity` には bare-name fallback が残っており、soundness は stable key の shape check で守っているが、長期的には substitute / monomorphize 側で identity origin を明示伝播する方が安全である。
- 2026-05-31 の raw-init stable entry checkpoint 後、`raw_init_param_facts_unstable_entry_bypasses` は 0 になった。一方で RPN same-session code edit の初回 compile では `raw_init_param_facts_reprojection_bypasses=165` が残り、store 可能な stable entry を現在 compile の `TypeCtx` / function signature へ戻す境界が次の blocker になった。

## 問題

RPN same-session code edit reports raw_init_param_facts_reprojection_bypasses=10. Subagent review identified that ResourceSummaryTypeReprojection may register generic nominal Apply trees using definition-side type variables instead of instantiated signature mappings.

## 影響

Nominal-heavy stdlib functions can build stable keys and entries but still fail fail-closed replay, limiting Resource summary value cache reuse.

## 修正方針

Make reprojection register instantiated nominal trees with explicit type-parameter substitution, and replace bare-name nominal identity fallback with deliberate identity propagation through substitute/monomorphize paths.

## 検証

Focused generic nominal raw-init and drop-traversal replay tests pass, and RPN reprojection_bypasses decreases without accepting ambiguous type keys.
