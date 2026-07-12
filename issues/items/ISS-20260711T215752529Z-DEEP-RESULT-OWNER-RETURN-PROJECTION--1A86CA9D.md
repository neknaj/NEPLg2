---
id: ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D
title: "Deep Result owner return projection reuses moved payload"
area: RESOURCE
status: open
resolved: false
priority: P1
type: bug
created: 2026-07-11
updated: 2026-07-12
target: nepl-core/src/resource/owner_return_apply_projection.rs
---

# ISS-20260711T215752529Z-DEEP-RESULT-OWNER-RETURN-PROJECTION--1A86CA9D: Deep Result owner return projection reuses moved payload

## 概要

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 対象

- `nepl-core/src/resource/owner_return_apply_projection.rs`

## 根拠

- GUI production chainでは`writer_step_budget(owner, 1)`の`Result<BudgetStep, StepError>`適用時に、複数の独立deep owner leafが順番に`ReturnValue` transferされ、2件目以降がchecker内部で`Moved`として拒否される。
- outer/inner enum pathをpayload bindへ遅延する試作は単純な`Result<Step, E>`では元の誤報を消したが、GUIのstruct fieldと複数内部owner enumを含む大きなprojection graphでは誤報が残った。
- moved sourceを単にskipする方法は、別variantが選択された場合にownerを返せなくなる。outer variant名だけの平坦化や最初のleaf採用では正当なowner transferを証明できない。
- 同一`OwnerProjectionSource`だけを排他的deep targetの共通prefixへaggregate化する試作はunit対照を通過したがproduction誤報は残った。その後のsource追跡では、異なるleaf同士を同一owning authorityへ収束させる明示的raw alias contractは存在せず、sourceとwriterおよび内部Vecは独立allocationだった。raw viewはnon-owningでありauthority group根拠にできない。
- callee `StorageId`ごとのabstract group schemaも試作したが、production parameter seedはdeep owner leafごとに新しいStorageIdを割り当てるため、StorageId partitionはaggregate authority relationを表さなかった。人工的に同じStorageIdを与えるunitだけではproduction契約を証明できない。
- `resource_ir_owner_check_returns_deep_multi_owner_aggregate_through_result`はdistinct-authority topologyの通過基準を固定した。`resource_ir_owner_check_routes_distinct_deep_owners_across_result_variants`は2個の独立deep ownerをOkと2種類のowner-bearing Errへ返す排他的path対照をowner checkとnormal compileで固定した。この限定的な深度・3 return variantだけでは再現せず、次候補は実owner chain固有のprojection形状またはsummary適用規模である。
- 同じreturn topologyでowner-token直上に内部enumを置いても通過した。`resource_ir_owner_check_scales_distinct_deep_result_projection_leaves`は独立deep owner leafを1、2、4、8、16、32個へ増やして全件通過し、このtopologyでは32 leaf以下の単純な数閾値を除外した。次はproductionのVec/OwnedBuffer storage enum、相関する複数metadata field、追加のsummary extent mappingを段階的に加える。

## 問題

A caller that consumes a deeply nested move-only registered owner through Result<BudgetStep, StepError> is rejected at the call expression with resource.owner.use_after_move on ReturnValue projections even though every branch consumes or frees the owner exactly once. F5nxj registered path command sink runtime reproduction fails first at writer_step_budget(recovered_writer, 0).

## 影響

Valid production owner chains cannot be exercised by runtime fixtures; F5nxj integration is blocked despite normal compile and source-policy gates passing.

## 修正方針

GUI owner chainから型を段階的に削り、どのowner leafまたはprojection expansionで最初に誤報するかを二分する。独立allocationをflat group化せず、owner summary内のsource suffixとreturn projectionの対応を直接検査する。適用側変更は、最小再現、distinct-authority control、owner-bearing Err path control、genuine use-after-move診断を同時に満たす場合だけ採用する。

## 検証

Run the minimized Resource IR regression and tests/stdlib/gui_font_registered_face.n.md with the F5nxj controlled 8-command runtime contract: read retry, zero/negative budget, partial seal, eight writes, terminal completion, checked seal, and cleanup-only push failure.
