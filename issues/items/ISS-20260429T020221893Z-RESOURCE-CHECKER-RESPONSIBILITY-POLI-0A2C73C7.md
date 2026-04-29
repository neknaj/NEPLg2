---
id: ISS-20260429T020221893Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-0A2C73C7
title: "resource checker responsibility policy rejects grouped owner imports"
area: nodesrc
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/summary.rs"
---

# ISS-20260429T020221893Z-RESOURCE-CHECKER-RESPONSIBILITY-POLI-0A2C73C7: resource checker responsibility policy rejects grouped owner imports

## 概要

GitHub Actions Source policy regressions fail because nodesrc/test_resource_checker_responsibility.js searches for the exact string super::owner_check::ResourceOwnerCheckEngine. summary.rs now uses the idiomatic grouped import super::owner_check::{resolve_owner_alias_place, ResourceOwnerCheckEngine}, so the policy reports a missing owner checker dependency even though the dependency exists and compiles.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/summary.rs`

## 根拠

- `nepl-core/src/resource/summary.rs` は `use super::owner_check::{resolve_owner_alias_place, ResourceOwnerCheckEngine};` で owner checker engine を import している。
- `nodesrc/test_resource_checker_responsibility.js` は direct path 文字列だけを `text.includes` で探していたため、compiler 上の依存が正しく存在しても Source policy が失敗した。

## 問題

GitHub Actions Source policy regressions fail because nodesrc/test_resource_checker_responsibility.js searches for the exact string super::owner_check::ResourceOwnerCheckEngine. summary.rs now uses the idiomatic grouped import super::owner_check::{resolve_owner_alias_place, ResourceOwnerCheckEngine}, so the policy reports a missing owner checker dependency even though the dependency exists and compiles.

## 影響

CI remains red after the move_check provenance split, and the source policy can pressure developers to perform import-style-only edits instead of checking the actual Resource IR checker boundary.

## 修正方針

Make the resource checker responsibility policy detect direct module imports and grouped imports for ResourceBorrowCheckEngine, ResourceOwnerCheckEngine, and ResourceEffectBoundaryEngine. Keep the Resource checker module boundaries and line limits unchanged.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/issues.js check, git diff --check, and focused checks if source files are touched.

- `node nodesrc\test_resource_checker_responsibility.js`: import 文字列不一致は解消。次の別件として `owner_check.rs has 930 lines; responsibility split limit is 800` を検出。
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

Resource checker responsibility policy に `assertUsesResourceModuleSymbol` を追加し、`super::module::Symbol` の direct import と `super::module::{..., Symbol}` の grouped import の両方を検出するようにした。対象は `summary.rs` の borrow / owner checker engine と、`effect_summary.rs` の effect boundary engine である。

この修正により import style だけの変更を要求する誤検出は消えた。一方で policy は次の実質的な責務分割違反として `owner_check.rs` の行数超過を検出したため、`ISS-20260429T020330179Z-RESOURCE-OWNER-CHECKER-EXCEEDS-RESPO-AB6E0E0E` に分離した。
