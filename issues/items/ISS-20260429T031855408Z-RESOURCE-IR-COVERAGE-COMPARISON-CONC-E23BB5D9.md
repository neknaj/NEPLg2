---
id: ISS-20260429T031855408Z-RESOURCE-IR-COVERAGE-COMPARISON-CONC-E23BB5D9
title: "Resource IR coverage comparison concentrates HIR and Resource IR responsibilities"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage_resource.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T031855408Z-RESOURCE-IR-COVERAGE-COMPARISON-CONC-E23BB5D9: Resource IR coverage comparison concentrates HIR and Resource IR responsibilities

## 概要

Resource IR lowering coverage has grown past 1000 lines and mixes public coverage types, HIR coverage collection, Resource IR coverage collection, unknown-place diagnostics, and count comparison in one file. This recreates the static-check responsibility concentration that the Resource IR migration is supposed to avoid.

## 対象

- `nepl-core/src/resource/coverage.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/src/resource/coverage_resource.rs, nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `nepl-core/src/resource/coverage.rs` は 1047 行まで増え、public coverage data shape、HIR coverage walker、Resource IR coverage walker、unknown-place diagnostic、count mismatch diagnostic を同じ file に持っていた。
- Stage 4/5 の compiler gate は Resource IR lowering coverage に依存するため、この file が巨大化すると、lowering 入力欠落を検出する guard 自体のレビュー性が落ちる。
- 既存の `nodesrc/test_resource_checker_responsibility.js` は checker 本体の分割は検査していたが、coverage comparison の責務集中は検出していなかった。

## 問題

Resource IR lowering coverage has grown past 1000 lines and mixes public coverage types, HIR coverage collection, Resource IR coverage collection, unknown-place diagnostics, and count comparison in one file. This recreates the static-check responsibility concentration that the Resource IR migration is supposed to avoid.

## 影響

Stage 4/5 gates rely on lowering coverage before enforcing cell, owner, borrow, and effect diagnostics. If coverage comparison keeps accumulating logic in one file, future static-check regressions become harder to audit and the responsibility guard misses a new monolithic checker surface.

## 修正方針

Split HIR-side coverage walking and Resource-IR-side coverage walking into dedicated modules while keeping coverage.rs as the public comparison/data-shape boundary. Extend the resource responsibility policy to require the split modules and line limits.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, cargo test -p nepl-core --test resource_ir resource_ir_lowering_coverage_detects_lost_call -- --nocapture, cargo check -p nepl-core --tests, node nodesrc/issues.js check, and git diff --check.

- `rustfmt --check nepl-core\src\resource\coverage.rs nepl-core\src\resource\coverage_hir.rs nepl-core\src\resource\coverage_resource.rs nepl-core\src\resource\mod.rs`: pass
- `node nodesrc\test_resource_checker_responsibility.js`: pass
- `cargo test -p nepl-core --test resource_ir coverage -- --nocapture`: 1 passed
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-resource-coverage-split-move-effect.json -j 1`: total=110, passed=110, failed=0
- `node nodesrc\issues.js check`: pass
- `git diff --check`: pass

## 対応結果

`coverage.rs` を public data shape と compare orchestration に限定し、HIR 側の coverage walker を `coverage_hir.rs`、Resource IR 側の coverage walker と unknown-place diagnostic を `coverage_resource.rs` へ分離した。これにより `coverage.rs` は 250 行、`coverage_hir.rs` は 345 行、`coverage_resource.rs` は 441 行になり、Stage 4/5 の lowering completeness gate をレビューしやすい境界に戻した。

`nodesrc/test_resource_checker_responsibility.js` には coverage module の存在確認と line limit を追加した。これにより、Resource IR checker 本体だけでなく、authoritative gate の前提になる lowering coverage comparison も単一巨大 pass に戻る退行を CI の Source policy で検出できる。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 4](../../doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-%E3%81%B8%E3%81%AE%E7%A7%BB%E8%A1%8C)
