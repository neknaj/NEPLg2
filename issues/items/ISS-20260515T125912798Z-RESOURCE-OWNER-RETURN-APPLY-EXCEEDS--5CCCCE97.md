---
id: ISS-20260515T125912798Z-RESOURCE-OWNER-RETURN-APPLY-EXCEEDS--5CCCCE97
title: "Resource owner_return_apply exceeds responsibility split limit after owner summary growth"
area: core
status: open
resolved: false
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

- 未記入

## 問題

After fixing the stale effect_return_summary_filter policy check, nodesrc/test_resource_checker_responsibility.js reaches the next hidden blocker: owner_return_apply.rs has 434 lines while the responsibility split limit is 410. Owner return transfer orchestration, parameter-source owner materialization, raw view propagation, returned extent application, and summary extent requirement checks are concentrated in one module.

## 影響

The Resource IR owner-return application path is becoming a new monolith. This raises the risk that future owner summary fixes for memory safety will be implemented by appending more conditional logic instead of preserving the MemPtr / OwnedRegion / InitializedCell separation.

## 修正方針

Split owner_return_apply.rs by responsibility without changing semantics. Keep orchestration in owner_return_apply.rs, move extent application/requirement checks or raw/non-owning view propagation into focused modules, update resource responsibility policy, and add/keep focused ResourceIR owner return regressions.

## 検証

Run cargo fmt -p nepl-core --check, focused ResourceIR owner return tests, nodesrc/test_resource_checker_responsibility.js, source policy warn-only, issues check, and diff whitespace check.
