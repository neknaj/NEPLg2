---
id: ISS-20260518T220931012Z-VEC-OWNER-RETURNING-ERROR-ACCESSORS--EEDC747A
title: "Vec owner-returning error accessors accept non-Copy payload before drop traversal"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js"
---

# ISS-20260518T220931012Z-VEC-OWNER-RETURNING-ERROR-ACCESSORS--EEDC747A: Vec owner-returning error accessors accept non-Copy payload before drop traversal

## 概要

VecPushError<T>, VecTransformError<T>, and VecSortMergeError<T> owner-returning accessors can be called with non-Copy T even though the producers are Copy-only until OwnedBuffer initialized-cell drop traversal exists.

## 対象

- `stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl, tests/stdlib/collection_cleanup_contract.n.md, nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`

## 根拠

- `vec_push_error_vec<T>` は `push<T: Copy>` の失敗 payload から `Vec<T>` owner を返す API だが、accessor 自体は unconstrained `T` だった。
- `vec_transform_error_vec<T>` も `map` / `filter` / `partition` / `take_while` / `drop_while` が Copy-only であるにもかかわらず、error payload から `Vec<T>` owner を unconstrained に返せた。
- `vec_sort_merge_error_vec<T>` も `sort_merge_ret<T: Ord&Copy>` の failure payload を unconstrained に分解できた。
- `BinaryHeapPushError` / `DequePushError` / `StackPushError` などの同種 accessor はすでに `.T: Copy` に限定済みであり、Vec だけ境界が揃っていなかった。

## 問題

VecPushError<T>, VecTransformError<T>, and VecSortMergeError<T> owner-returning accessors can be called with non-Copy T even though the producers are Copy-only until OwnedBuffer initialized-cell drop traversal exists.

## 影響

A direct parameter or future producer of these error payloads could expose Vec<NonCopyPayload> through the safe accessor surface before collection drop traversal and non-Copy owner discipline are implemented, weakening the current Copy-only safety boundary.

## 修正方針

Constrain the owner-returning Vec error accessors to T: Copy, add compile-fail regressions for CleanupPayload, and extend the Vec source policy so these accessors cannot regress to unconstrained signatures.

## 検証

Run the collection cleanup contract doctests and Vec source policy regression.

## 2026-05-18 修正

`vec_push_error_vec<T>`、`vec_transform_error_vec<T>`、`vec_sort_merge_error_vec<T>` を `.T: Copy` に限定した。あわせて internal grow failure payload の `vec_realloc_region_error_region<T>` も現行 `push<T: Copy>` contract に揃え、Copy-only `Vec` storage 境界から non-Copy `RegionToken<T>` を public recovery 面へ出さないようにした。

回帰として、`tests/stdlib/collection_cleanup_contract.n.md` に `VecPushError<CleanupPayload>`、`VecTransformError<CleanupPayload>`、`VecSortMergeError<CleanupPayload>` の owner accessor が `type.trait_bound.unsatisfied` で拒否される compile-fail doctest を追加した。`nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` でも各 accessor signature を直接監視する。

これは non-Copy collection の完成ではない。`OwnedBuffer<T>` の initialized prefix、moved slot、drop traversal、compiler-issued owner token が入るまでは、error payload からの owner recovery も Copy-only collection 契約に閉じる。

検証:

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-vec-error-accessor-copy-bound.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i tests/stdlib/neplg2_type_arena.n.md --no-tree -o tmp/agent1-vec-error-accessor-type-arena.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/alloc/diag/error/diags.nepl --no-tree -o tmp/agent1-vec-error-accessor-diags.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/tests.js -i stdlib/neplg2/core/module/import_scan.nepl --no-tree -o tmp/agent1-vec-error-accessor-import-scan.json -j 1 --dist web/dist --assert-io`
- `node nodesrc/run_doctest.js -i tests/stdlib/neplg2_stdlib_map.n.md -n 2 --assert-io --dist web/dist`
