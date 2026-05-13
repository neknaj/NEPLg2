---
id: ISS-20260513T092818532Z-VEC-CLEANUP-FREE-ACCEPT-NON-COPY-PAY-497499BC
title: "Vec cleanup/free accept non-Copy payload without element drop traversal"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/vec/**, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260513T092818532Z-VEC-CLEANUP-FREE-ACCEPT-NON-COPY-PAY-497499BC: Vec cleanup/free accept non-Copy payload without element drop traversal

## 概要

Vec.clear and Vec.free are storage-only cleanup paths, but they currently accept unconstrained T even though they do not traverse initialized elements or invoke Drop. This makes unsupported non-Copy payload vectors look safely discardable before OwnedBuffer and initialized-cell traversal exist.

## 対象

- `stdlib/alloc/collections/vec/**, tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `stdlib/alloc/collections/vec/mutation/cleanup.nepl` の `clear<T>` / `free<T>` は、`len` を 0 にする、または backing storage を解放するだけで、`0..len` の initialized element を走査して `Drop` しない。
- `stdlib/alloc/collections/vec/storage/cleanup.nepl` の `vec_free_storage<T>` は `VecStorageState` と `dealloc_ptr<T>` による storage-only cleanup であり、element payload の所有権を閉じる処理を持たない。
- 親 issue `ISS-20260425T000000Z-RV-STDLIB-004-91534828` は、non-Copy payload collection の完成条件を `OwnedBuffer<T>`、initialized prefix、move-out / replace / drop traversal、fallible update owner contract として整理している。

## 問題

Vec.clear and Vec.free are storage-only cleanup paths, but they currently accept unconstrained T even though they do not traverse initialized elements or invoke Drop. This makes unsupported non-Copy payload vectors look safely discardable before OwnedBuffer and initialized-cell traversal exist.

## 影響

A caller can clear or free a Vec<T> containing owning non-Copy payloads without the compiler seeing a Drop traversal obligation, weakening RV-STDLIB-004 and hiding the remaining Stage 6 collection redesign work.

## 修正方針

Require Copy on Vec.clear/free and the storage-only vec_free_storage helper. Keep non-Copy push/pop/owner-preserving redesign under RV-STDLIB-004 and OwnedBuffer Stage D instead of pretending storage-only cleanup supports it.

## 検証

Add compile-fail coverage for Vec<CleanupPayload> clear/free and source policy assertions that Vec storage-only cleanup remains Copy-only.

## 修正結果

- `Vec.clear<T>` / `Vec.free<T>` を `.T: Copy` に限定し、Drop traversal を持たない API が non-Copy payload を安全に破棄できるように見える入口を閉じた。
- internal `vec_free_storage<T>` も `.T: Copy` に揃え、public `free` と storage-only dealloc helper の契約差をなくした。
- `tests/stdlib/collection_cleanup_contract.n.md` に `Vec<CleanupPayload>` の `clear` / `free` compile-fail を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に Vec cleanup/free の Copy-only source policy を追加した。

## 残件

この修正は `Vec` の storage-only cleanup 契約漏れを閉じる局所修正であり、non-Copy payload collection の完成ではない。`push` の fallible update、`pop` / remove の owner-preserving API、collection-wide Drop traversal、`OwnedBuffer<T>` / initialized prefix model は `RV-STDLIB-004` と Stage 6 `OwnedBuffer` 設計で継続する。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/vec-cleanup-copy-contract.json -j 1 --dist web/dist`
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/vec-cleanup-copy-vec-doctests.json -j 4 --dist web/dist`
- `node nodesrc/issues.js check --dir issues`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `git diff --check`
