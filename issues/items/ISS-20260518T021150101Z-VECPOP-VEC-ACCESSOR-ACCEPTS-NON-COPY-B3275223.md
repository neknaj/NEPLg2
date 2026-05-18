---
id: ISS-20260518T021150101Z-VECPOP-VEC-ACCESSOR-ACCEPTS-NON-COPY-B3275223
title: "VecPop vec accessor accepts non-Copy payload and can discard popped owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/vec/types.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260518T021150101Z-VECPOP-VEC-ACCESSOR-ACCEPTS-NON-COPY-B3275223: VecPop vec accessor accepts non-Copy payload and can discard popped owner

## Related

- Parent: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- Stage: `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` Stage D / `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6

## 概要

VecPop<T> contains both the updated Vec owner and the popped Option<T>. vec_pop_vec<T> consumes VecPop<T> and returns only the Vec, but its generic parameter is unconstrained. While pop<T> is Copy-only, a direct parameter or future producer of VecPop<NonCopyPayload> can call vec_pop_vec and drop the Option<T> payload without element drop traversal.

## 対象

- `stdlib/alloc/collections/vec/types.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/vec/types.nepl` の `VecPop<T>` は `vec: Vec<T>` と `item: Option<T>` を保持する。
- 同 file の旧 `vec_pop_vec<T>` は `field::get p "vec"` だけを返し、`item` を返さないにもかかわらず `.T: Copy` bound を持っていなかった。

## 問題

VecPop<T> contains both the updated Vec owner and the popped Option<T>. vec_pop_vec<T> consumes VecPop<T> and returns only the Vec, but its generic parameter is unconstrained. While pop<T> is Copy-only, a direct parameter or future producer of VecPop<NonCopyPayload> can call vec_pop_vec and drop the Option<T> payload without element drop traversal.

## 影響

This weakens the Stage 6 Copy-only collection boundary under RV-STDLIB-004. Non-Copy payload move-out remains unsupported until OwnedBuffer initialized cell state and drop traversal exist, so every public API that can discard the popped item must reject non-Copy payloads.

## 修正方針

Constrain vec_pop_vec to T: Copy, add a compile-fail regression that calls vec_pop_vec<NonCopyPayload> through a VecPop parameter, and extend the Vec source policy so this accessor cannot regress to an unconstrained signature.

## 検証

Run the Vec source policy, the collection cleanup contract doctest, issue validation, and diff whitespace checks.

## 2026-05-18 Agent 1 修正

`vec_pop_vec<T>` を `.T: Copy` に限定した。`VecPop<T>` は更新後 `Vec<T>` と取り出した `Option<T>` を同時に保持するため、item を返さない accessor は現行の Copy-only collection 境界に揃える必要がある。

回帰として、`VecPop<CleanupPayload>` parameter から `vec_pop_vec<CleanupPayload>` を呼ぶ独立 compile-fail doctest を追加し、source policy にも `vec_pop_vec<T: Copy>` signature を固定した。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_doctest.js -i tests\stdlib\collection_cleanup_contract.n.md -n 4 --dist web\dist`: passed
- `node nodesrc/tests.js -i tests\stdlib\collection_cleanup_contract.n.md --no-tree -o tmp\agent1-vecpop-copy-bound-contract.json -j 1 --dist web\dist --assert-io`: total=26, passed=26
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
