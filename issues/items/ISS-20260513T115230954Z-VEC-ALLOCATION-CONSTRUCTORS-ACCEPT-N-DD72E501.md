---
id: ISS-20260513T115230954Z-VEC-ALLOCATION-CONSTRUCTORS-ACCEPT-N-DD72E501
title: "Vec allocation constructors accept non-Copy payloads without cleanup contract"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/alloc/collections/vec/storage/alloc.nepl
---

# ISS-20260513T115230954Z-VEC-ALLOCATION-CONSTRUCTORS-ACCEPT-N-DD72E501: Vec allocation constructors accept non-Copy payloads without cleanup contract

## 概要

Vec.new<T>, Vec.with_capacity<T>, and vec_alloc_empty<T> can allocate backing storage for non-Copy payloads even though current Vec cleanup, mutation, raw element access, and drop traversal are Copy-only. This creates Vec<T> owners whose storage cannot be freed through the supported API when T is non-Copy.

## 対象

- `stdlib/alloc/collections/vec/storage/alloc.nepl`

## 根拠

- 現行 `Vec.clear` / `Vec.free` / `vec_free_storage` は `.T: Copy` に限定されており、non-Copy payload の element drop traversal を持たない。
- `Vec.push` / `Vec.pop` / raw element helper / sort も Stage D の境界として `.T: Copy` に閉じている。
- 変更前の `vec_alloc_empty<T>` / `new<T>` / `with_capacity<T>` は制約なしで `alloc_ptr<T>` を呼べたため、non-Copy payload の backing storage owner だけを作成できた。
- `vec_empty<T>` は allocation を作らない typed empty state helper であり、free obligation owner を発生させないため今回の対象から外した。

## 問題

Vec.new<T>, Vec.with_capacity<T>, and vec_alloc_empty<T> can allocate backing storage for non-Copy payloads even though current Vec cleanup, mutation, raw element access, and drop traversal are Copy-only. This creates Vec<T> owners whose storage cannot be freed through the supported API when T is non-Copy.

## 影響

Unsupported non-Copy Vec payloads can still create allocation/free obligations and leak storage or force callers toward raw-memory escape paths. This contradicts the Stage D boundary that non-Copy collection support requires OwnedBuffer<T> plus initialized cell/drop state.

## 修正方針

Restrict Vec allocation constructors that can allocate backing storage to T: Copy until OwnedBuffer<T> exists. Keep zero-allocation vec_empty as the typed empty state helper. Add compile-fail doctests for non-Copy new/with_capacity and source policy coverage.

## 検証

Run Vec source policy checks, focused storage allocation doctests, Vec doctests, issue check, and diff check.

## 修正内容

- `vec_alloc_empty` / `new` / `with_capacity` を `.T: Copy` に限定した。
- `storage/alloc.nepl` の doc comment に、non-Copy payload の allocation constructor を `OwnedBuffer<T>` / drop traversal 導入まで許可しない理由を明記した。
- `Vec<NonCopyPayload>` の `new` / `with_capacity` が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` の source policy を、allocation constructor が Copy-only であり non-Copy regression を持つことを監視する形へ更新した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/storage/alloc.nepl --no-tree -o tmp/agent1-vec-allocation-copy-bound-alloc.json -j 1 --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-allocation-copy-bound-vec.json -j 4 --dist web/dist`: total=39, passed=39
- `git diff --check`: passed

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の Stage D 残件のうち、non-Copy payload の backing storage owner だけを作れる入口を閉じた。
- full non-Copy collection support はこの issue では完了扱いにしない。non-Copy allocation は `OwnedBuffer<T>`、initialized prefix、partial initialization state、drop traversal、failure cleanup を設計してから再導入する。
