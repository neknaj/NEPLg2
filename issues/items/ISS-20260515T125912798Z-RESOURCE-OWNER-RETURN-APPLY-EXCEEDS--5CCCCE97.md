---
id: ISS-20260515T125912798Z-RESOURCE-OWNER-RETURN-APPLY-EXCEEDS--5CCCCE97
title: "Resource owner_return_apply exceeds responsibility split limit after owner summary growth"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/owner_return_apply.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T125912798Z-RESOURCE-OWNER-RETURN-APPLY-EXCEEDS--5CCCCE97: Resource owner_return_apply exceeds responsibility split limit after owner summary growth

## 概要

After fixing the stale effect_return_summary_filter policy check, nodesrc/test_resource_checker_responsibility.js reaches the next hidden blocker: owner_return_apply.rs has 434 lines while the responsibility split limit is 410. Owner return transfer orchestration, parameter-source owner materialization, raw view propagation, returned extent application, and summary extent requirement checks are concentrated in one module.

## 対象

- `nepl-core/src/resource/owner_return_apply.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_return_apply.rs has 434 lines; responsibility split limit is 410` で失敗した。
- `owner_return_apply.rs` には owner return transfer orchestration と、返却 extent の requirement 照合 / live extent 反映 helper が同居していた。
- extent requirement は owner return application の中でも独立した proof boundary であり、owner transfer loop 本体に残すと summary growth のたびに同 module が再肥大化しやすい。

## 問題

After fixing the stale effect_return_summary_filter policy check, nodesrc/test_resource_checker_responsibility.js reaches the next hidden blocker: owner_return_apply.rs has 434 lines while the responsibility split limit is 410. Owner return transfer orchestration, parameter-source owner materialization, raw view propagation, returned extent application, and summary extent requirement checks are concentrated in one module.

## 影響

The Resource IR owner-return application path is becoming a new monolith. This raises the risk that future owner summary fixes for memory safety will be implemented by appending more conditional logic instead of preserving the MemPtr / OwnedRegion / InitializedCell separation.

## 修正方針

Split owner_return_apply.rs by responsibility without changing semantics. Keep orchestration in owner_return_apply.rs, move extent application/requirement checks or raw/non-owning view propagation into focused modules, update resource responsibility policy, and add/keep focused ResourceIR owner return regressions.

## 検証

Run cargo fmt -p nepl-core --check, focused ResourceIR owner return tests, nodesrc/test_resource_checker_responsibility.js, source policy warn-only, issues check, and diff whitespace check.

## 対応

`owner_return_apply.rs` から返却 extent の照合・反映責務を `owner_return_apply_extent.rs` へ分離した。

- `summary_return_extent_requirement_holds` を `owner_return_apply_extent.rs` の `ResourceOwnerCheckEngine` impl へ移し、owner return transfer loop から extent proof detail を外した。
- `apply_returned_owner_extent` を同 module へ移し、`OwnerParameterReturnExtent` と `OwnerStorageExtent` の適用を集約した。
- `owner_return_apply.rs` は owner return / projection return の orchestration と transfer selection に集中する形へ戻した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_return_apply_extent.rs` を mandatory module / line-limit 監視として追加した。
- 分割後の行数は `owner_return_apply.rs=382`、`owner_return_apply_extent.rs=68`。
- policy はこの blocker を通過し、次の別 blocker `owner_variant.rs has 871 lines; responsibility split limit is 840` を露出した。これは `ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F` に分離した。

## 検証結果

- `cargo fmt -p nepl-core --check`: pass
- `cargo test -p nepl-core --test resource_ir owner_return -- --nocapture`: 9 passed
- `node --check nodesrc/test_resource_checker_responsibility.js`: pass
- `node nodesrc/test_resource_checker_responsibility.js`: owner_return_apply blocker は解消。次 blocker `ISS-20260515T130438617Z-RESOURCE-OWNER-VARIANT-MODULE-EXCEED-3B94063F` に到達。
