---
id: ISS-20260506T000220058Z-FULLREVIEW-STATIC-CHECK-RESOURCE-REP-BB4876E4
title: "Fullreview static-check resource report still lists resolved owner variant path split as open"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-06
updated: 2026-05-06
target: "doc/fullreview20260430/rust-compiler/static-check-resource.md, issues/items/ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8.md"
---

# ISS-20260506T000220058Z-FULLREVIEW-STATIC-CHECK-RESOURCE-REP-BB4876E4: Fullreview static-check resource report still lists resolved owner variant path split as open

## 概要

doc/fullreview20260430/rust-compiler/static-check-resource.md still says owner_summary_variant_paths.rs is 637 lines and policy-red, but main has already split the owner variant path builder and the file is now within the responsibility policy.

## 対象

- `doc/fullreview20260430/rust-compiler/static-check-resource.md, issues/items/ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8.md`

## 根拠

- `doc/fullreview20260430/rust-compiler/static-check-resource.md` が、`owner_summary_variant_paths.rs` 637 lines / policy red を現在の残件として記載していた。
- 現在の `owner_summary_variant_paths.rs` は 338 lines で、`node nodesrc/test_resource_checker_responsibility.js` は passed。
- `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` は fixed で、分割後の module と line limit が issue に記録済みである。

## 問題

doc/fullreview20260430/rust-compiler/static-check-resource.md still says owner_summary_variant_paths.rs is 637 lines and policy-red, but main has already split the owner variant path builder and the file is now within the responsibility policy.

## 影響

The fullreview report is used for static-check prioritization. Leaving a resolved responsibility split as an active blocker can cause agents to duplicate fixed work and miss the remaining real blockers around old checker authority and memory-model separation.

## 修正方針

Update the fullreview static-check resource report to mark owner variant path responsibility split as completed and keep the remaining problem list focused on old move_check/drop insertion authority, raw-memory-boundary migration, and MemPtr/Storage/InitializedCell separation.

## 検証

node nodesrc/issues.js check; git diff --check

## 解決

`doc/fullreview20260430/rust-compiler/static-check-resource.md` の owner variant path builder 記述を現在の main に合わせて更新した。

- `owner_summary_variant_paths.rs` の 637 lines / policy red 記述を、fixed 済みの責務分割として修正した。
- 2026-05-06 追補の残件リストから owner variant path builder を外し、残る blocker が旧 checker authority、raw-memory-boundary migration、MemPtr/Storage/InitializedCell 分離であることを明確にした。

検証:

- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
