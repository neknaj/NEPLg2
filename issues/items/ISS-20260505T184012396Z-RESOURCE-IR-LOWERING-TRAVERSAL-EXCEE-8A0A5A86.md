---
id: ISS-20260505T184012396Z-RESOURCE-IR-LOWERING-TRAVERSAL-EXCEE-8A0A5A86
title: "Resource IR lowering traversal exceeds responsibility split limit"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260505T184012396Z-RESOURCE-IR-LOWERING-TRAVERSAL-EXCEE-8A0A5A86: Resource IR lowering traversal exceeds responsibility split limit

## 概要

After owner variant path splitting, the direct Resource checker responsibility policy now reaches nepl-core/src/resource/lower.rs and reports 1315 lines over the 1300-line limit. This indicates Resource IR lowering traversal has accumulated enough logic again that Stage 4 static-check input construction is no longer safely bounded by the existing split policy.

## 対象

- `nepl-core/src/resource/lower.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は owner variant path builder 分割後、次の未解決責務違反として `lower.rs has 1315 lines; responsibility split limit is 1300` を報告する。
- 既に `lower_raw_address.rs` と `lower_raw_memory.rs` は分離済みだが、`lower.rs` 本体が再び上限を超えているため、raw address 以外の lowering traversal / op construction / condition lowering / aggregate projection lowering が同居している可能性が高い。
- これは `ISS-20260430T135243330Z-RESOURCE-OWNER-VARIANT-PATH-BUILDER--87B356A8` の対応中に発覚した別責務の問題であり、owner variant path builder の修正とは別 issue として扱う。

## 問題

After owner variant path splitting, the direct Resource checker responsibility policy now reaches nepl-core/src/resource/lower.rs and reports 1315 lines over the 1300-line limit. This indicates Resource IR lowering traversal has accumulated enough logic again that Stage 4 static-check input construction is no longer safely bounded by the existing split policy.

## 影響

Resource IR lowering is the trusted input for initialized, owner, borrow, and effect checks. If lower.rs keeps growing, future MemPtr/non-owning pointer, OwnedRegion/storage owner, and Resource IR state fixes can be coupled to general HIR traversal without a focused audit boundary.

## 修正方針

Split the remaining lower.rs responsibilities by semantic role rather than raising the limit: keep traversal orchestration in lower.rs, and extract condition/variant lowering, aggregate projection lowering, and call/effect resource op construction into focused modules with source-policy guards.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, cargo fmt --check -p nepl-core, cargo check -p nepl-core --tests, and focused Resource IR lowering/owner regression tests.
