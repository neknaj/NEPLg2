---
id: ISS-20260512T111921719Z-RESOURCE-COVERAGE-MODULE-EXCEEDS-RES-2FFC6C86
title: "Resource coverage module exceeds responsibility limit after operation enum migration"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_kind.rs, nepl-core/src/resource/coverage_operation.rs, nepl-core/src/resource/coverage_resource_place.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T111921719Z-RESOURCE-COVERAGE-MODULE-EXCEEDS-RES-2FFC6C86: Resource coverage module exceeds responsibility limit after operation enum migration

## 概要

ResourceCoveragePlaceOperation enum was added to coverage.rs while moving coverage unknown-place classification away from free-form strings. The source policy now reports coverage.rs has 369 lines against a 280 line responsibility limit.

## 対象

- `nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_kind.rs, nepl-core/src/resource/coverage_operation.rs, nepl-core/src/resource/coverage_resource_place.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `coverage.rs has 369 lines; responsibility split limit is 280` を報告した。
- `ResourceCoveragePlaceOperation` の enum 化自体は正しいが、coverage report、coverage kind、unknown-place operation taxonomy、HIR/Resource comparison を同一 file に戻していた。
- operation enum 分離後も `coverage_resource.rs` が 525 lines / limit 520 になり、ResourceOp traversal と place-level unknown-place diagnostic generation が同居していることも確認した。

## 問題

ResourceCoveragePlaceOperation enum was added to coverage.rs while moving coverage unknown-place classification away from free-form strings. The source policy now reports coverage.rs has 369 lines against a 280 line responsibility limit.

## 影響

Resource IR coverage, count comparison, and operation taxonomy are concentrated in one file again. Keeping the oversized module would weaken the static-check large-refactor rule that Resource IR proof modules stay small and reviewable.

## 修正方針

Move ResourceCoveragePlaceOperation and its display spelling boundary into a dedicated resource coverage operation module, export it through resource/mod.rs, and keep coverage.rs focused on coverage report data and HIR/Resource comparison.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, focused resource_ir coverage tests, cargo check -p nepl-core --tests, and issue checks.

## 対応結果

`ResourceCoveragePlaceOperation` を `coverage_operation.rs` に、`ResourceCoverageKind` を `coverage_kind.rs` に分離した。さらに `coverage_resource.rs` の末尾にあった place-level unknown-place diagnostic helper を `coverage_resource_place.rs` へ移し、ResourceOp traversal と Place 検査生成の責務を分けた。

`nodesrc/test_resource_checker_responsibility.js` には新 module の存在、`mod` 登録、役割 marker、line budget を追加し、今回の分割が戻らないように固定した。limit を緩める修正は行っていない。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
