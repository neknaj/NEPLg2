---
id: ISS-20260604T034125199Z-VEC-SORT-VARIANTS-HANDLE-INVALID-MET-1475F2ED
title: "Vec sort variants handle invalid metadata inconsistently and silently no-op"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-05
target: "stdlib/alloc/collections/vec/types.nepl, stdlib/alloc/collections/vec/sort/quick.nepl, stdlib/alloc/collections/vec/sort/heap.nepl, stdlib/alloc/collections/vec/sort/simple/*.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl"
---

# ISS-20260604T034125199Z-VEC-SORT-VARIANTS-HANDLE-INVALID-MET-1475F2ED: Vec sort variants handle invalid metadata inconsistently and silently no-op

## 概要

Subagent audit found quick/heap/selection style sort helpers silently returning unit for invalid views while merge sort had Result-shaped error handling. This conflicted with Zenn guidance to avoid silent no-op except documented best-effort effects and to model unsupported/invalid states explicitly.

## 対象

- `stdlib/alloc/collections/vec/types.nepl`
- `stdlib/alloc/collections/vec/sort/quick.nepl`
- `stdlib/alloc/collections/vec/sort/heap.nepl`
- `stdlib/alloc/collections/vec/sort/simple/insertion.nepl`
- `stdlib/alloc/collections/vec/sort/simple/selection.nepl`
- `stdlib/alloc/collections/vec/sort/simple/exchange.nepl`
- `stdlib/alloc/collections/vec/sort/simple/gap.nepl`
- `stdlib/alloc/collections/vec/sort/merge/api.nepl`

## 根拠

- `sort_quick` / `sort_heap` / simple sort variants returned bare `unit` for `VecCopyInvariant::Invalid` and `VecDataView::Invalid`.
- `sort_quick_ret` / `sort_heap_ret` returned the original `Vec` without typed failure on invalid metadata.
- `sort_merge_ret` had a merge-only owner error payload, so the sort family did not share one failure surface.

## 問題

quick / heap / simple sort helpers silently returned `unit` for invalid views while merge sort used `Result`. Owner-consuming quick / heap helpers also returned the input `Vec` directly, making failure indistinguishable from a valid no-op.

## 影響

Sort callers could not reliably tell whether a sort succeeded, failed due to invalid metadata, or merely did no work; this also hid owner recovery obligations for future non-Copy sort support.

## 修正方針

Resolved by unifying borrowed sort APIs around `Result unit StdErrorKind` and owner-consuming sort APIs around `Result Vec .T VecSortError .T`. `VecSortError` lives in `vec/types.nepl`, carries the consumed `Vec` owner plus `StdErrorKind`, and exposes `vec_sort_error_kind`, `vec_sort_error_with`, and Copy-only `vec_sort_error_vec`. Invalid metadata now returns `StdErrorKind::InvalidOperation`; merge scratch allocation failure still returns `StdErrorKind::OutOfMemory`.

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_vec_sort_module_split.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_neplg21_collection_cleanup_contract_postfix_cleanup.js`
- `node nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js`
- `node nodesrc/test_neplg21_pipe_traits_sort_postfix_cleanup.js`
- `node nodesrc/test_neplg21_prose_type_notation_cleanup.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/sort.nepl -i tests/stdlib/sort.n.md -i tests/stdlib/sort_simple.n.md -i tests/stdlib/traits_order.n.md -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent2-vec-sort-result-tests -j 1 --dist web/dist --assert-io` -> 81/81 passed
