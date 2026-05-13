---
id: ISS-20260513T125403348Z-RESOURCE-OWNER-EXTENT-PROOF-DOES-NOT-510A15F8
title: "Resource owner extent proof does not record repeated size_of constants"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/resource/lower.rs, nepl-core/src/resource/owner_expr.rs, tests/compiler/move_effect.n.md"
---

# ISS-20260513T125403348Z-RESOURCE-OWNER-EXTENT-PROOF-DOES-NOT-510A15F8: Resource owner extent proof does not record repeated size_of constants

## 概要

After rebuilding web/dist, tests/compiler/move_effect.n.md doctest#57 fails with resource.owner.unavailable ReallocExtent even though alloc_raw and realloc_raw both use size_of<LocalToken>. Resource owner extent comparison sees two size_of expressions as unrelated scalar places instead of the same compile-time constant.

## 対象

- `nepl-core/src/resource/lower.rs, nepl-core/src/resource/owner_expr.rs, tests/compiler/move_effect.n.md`

## 根拠

- `trunk build` 後の `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree ...` で doctest#57 が `resource.owner.unavailable` / `ReallocExtent` により失敗した。
- 同じ doctest は `alloc_raw size_of<LocalToken>` と `realloc_raw p size_of<LocalToken> 32` を使っており、source 上は同じ layout constant である。
- 早期 return で `size_of` call 自体を `LiteralI32` に置き換える試作では `resource.lower.incomplete` が発生し、Resource IR coverage gate が DirectCall 欠落を検出した。したがって Call / Intrinsic coverage は残したまま scalar fact だけを記録する必要がある。

## 問題

After rebuilding web/dist, tests/compiler/move_effect.n.md doctest#57 fails with resource.owner.unavailable ReallocExtent even though alloc_raw and realloc_raw both use size_of<LocalToken>. Resource owner extent comparison sees two size_of expressions as unrelated scalar places instead of the same compile-time constant.

## 影響

Safe raw realloc after moving a non-Copy cell out can be rejected unless source code binds the size to a local first. This makes owner extent proof depend on incidental expression sharing instead of compiler-proved layout constants, weakening the correctness and predictability of Resource IR static checks.

## 修正方針

Record compile-time layout intrinsics such as size_of<T> and align_of<T> as scalar i32 facts in Resource IR so repeated constants compare by value during owner extent checks.

## 検証

Run trunk build, node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 57 --dist web/dist, node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/... -j 1 --dist web/dist, cargo check -p nepl-core --tests, node nodesrc/issues.js check --dir issues, and git diff --check.

## 2026-05-13 修正

Resource IR lowering で `size_of<T>` / `align_of<T>` を compile-time i32 scalar fact として記録するようにした。

- `Call` / `Intrinsic` の Resource IR coverage は維持し、最終 `Expr` の `ResourceExprKind` だけを `LiteralI32(value)` にして owner / initialized checker が同じ定数値を参照できるようにした。
- `size_of<T>` の repeated expression が別 temporary になっても `OwnerStorageExtent::PayloadBytes` の比較で値一致を証明できる。
- 早期 return で `DirectCall` coverage を落とす設計は破棄し、Resource IR coverage gate が効く形を維持した。
- layout intrinsic constant extraction は `lower_layout_intrinsic.rs` に分離し、`lower.rs` の責務行数を source policy limit 内に戻した。

検証:

- `cargo check -p nepl-core --tests`: passed
- `trunk build`: passed
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 57 --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/agent1-sizeof-extent-move-effect-after.json -j 1 --dist web/dist`: 113 passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
