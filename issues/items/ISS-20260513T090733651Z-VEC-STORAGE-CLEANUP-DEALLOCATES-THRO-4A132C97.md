---
id: ISS-20260513T090733651Z-VEC-STORAGE-CLEANUP-DEALLOCATES-THRO-4A132C97
title: "Vec storage cleanup deallocates through raw address and loses owner obligation"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/vec/storage/cleanup.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl"
---

# ISS-20260513T090733651Z-VEC-STORAGE-CLEANUP-DEALLOCATES-THRO-4A132C97: Vec storage cleanup deallocates through raw address and loses owner obligation

## 概要

Vec storage cleanup and scratch-buffer cleanup lower an allocated MemPtr through mem_ptr_addr and call dealloc_raw directly. After MemPtr is fixed as a non-owning pointer view, this raw i32 path loses the compiler-visible free obligation and Vec doctests fail with resource.owner.no_free_obligation or maybe_leak.

## 対象

- `stdlib/alloc/collections/vec/storage/cleanup.nepl, stdlib/alloc/collections/vec/mutation/push.nepl, stdlib/alloc/collections/vec/sort/merge/api.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/vec-owner-dealloc-ptr-all.json -j 4 --dist web/dist`: total=32, passed=30, failed=2。残りは merge sort scratch buffer の `resource.owner.maybe_leak`。
- 直前の Vec/string broad doctest では、undefined import 解消後に Vec 側 31 件が `resource.owner.no_free_obligation` で止まっていた。
- 失敗箇所は `vec_free_storage`、`push` の realloc failure cleanup、merge sort scratch buffer cleanup で、いずれも `mem_ptr_addr` で `MemPtr` を raw `i32` に落として `dealloc_raw` へ渡していた。

## 問題

Vec storage cleanup and scratch-buffer cleanup lower an allocated MemPtr through mem_ptr_addr and call dealloc_raw directly. After MemPtr is fixed as a non-owning pointer view, this raw i32 path loses the compiler-visible free obligation and Vec doctests fail with resource.owner.no_free_obligation or maybe_leak.

## 影響

Vec becomes unusable under Stage 6 Resource IR owner checking, and doctest failures obscure remaining static-check migration work.

## 修正方針

Route owned Vec buffers and merge-sort scratch buffers through typed dealloc_ptr, and make impossible dealloc_ptr failure branches unreachable so owner obligations cannot leak.

## 検証

- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/alloc/collections/vec/mutation/push.nepl -i stdlib/alloc/collections/vec/storage/alloc.nepl --no-tree -o tmp/vec-owner-dealloc-ptr-focused.json -j 1 --dist web/dist`: total=6, passed=6。
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/vec-owner-dealloc-ptr-all-after-unreachable.json -j 4 --dist web/dist`: total=32, passed=32。
- `node nodesrc/tests.js -i stdlib/alloc/string -i stdlib/alloc/collections/vec --no-tree -o tmp/string-vec-after-vec-owner-dealloc-ptr.json -j 4 --dist web/dist`: total=46, passed=46。
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed。

## 解決

- `vec_free_storage` は `dealloc_raw mem_ptr_addr data ...` をやめ、typed owner-consuming `dealloc_ptr<T>` で backing storage を閉じるようにした。
- `push` の `realloc_ptr` failure cleanup も raw address dealloc ではなく `dealloc_ptr<T>` を使うようにした。
- merge sort の scratch buffer は `alloc_ptr<T>` で確保した owner を `dealloc_ptr<T>` で閉じるようにし、確保直後の buffer dealloc 失敗は通常の Vec error ではなく invariant violation として `unreachable` にした。
- これにより `MemPtr` を raw `i32` owner に変換せず、free obligation を Resource IR が追跡できる typed API 境界に残した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_sort_merge_no_unsafe_unwraps.js` は、旧 `dealloc_raw mem_ptr_addr` 固定から、typed `dealloc_ptr` cleanup とその Err 分岐だけの invariant `unreachable` を監視する policy へ更新した。
