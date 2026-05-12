---
id: ISS-20260512T061533036Z-RESOURCE-OWNER-CHECK-EXCEEDS-RESPONS-687898EE
title: "Resource owner_check exceeds responsibility split limit after i32 fact changes"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_check.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T061533036Z-RESOURCE-OWNER-CHECK-EXCEEDS-RESPONS-687898EE: Resource owner_check exceeds responsibility split limit after i32 fact changes

## 概要

After remote main commit 3487e386 and the source policy rename sync, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_check.rs has 813 lines while the responsibility split limit remains 800. The file has regrown past the boundary after Resource IR owner summary and i32 fact changes.

## 対象

- `nepl-core/src/resource/owner_check.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- 未記入

## 問題

After remote main commit 3487e386 and the source policy rename sync, nodesrc/test_resource_checker_responsibility.js reaches the next blocker: owner_check.rs has 813 lines while the responsibility split limit remains 800. The file has regrown past the boundary after Resource IR owner summary and i32 fact changes.

## 影響

Resource owner checking starts accumulating helper predicates and deferred-state plumbing in the traversal module again. This weakens the responsibility split policy used to keep memory-safety checks maintainable.

## 修正方針

Do not raise the owner_check.rs limit. Move small owner-check utility predicates/deferred merge helpers out of owner_check.rs into a dedicated module, keep owner_check.rs focused on traversal and dispatch, and update the source policy to cover the new module.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.
