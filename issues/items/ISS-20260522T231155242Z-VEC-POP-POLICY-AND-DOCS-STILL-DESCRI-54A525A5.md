---
id: ISS-20260522T231155242Z-VEC-POP-POLICY-AND-DOCS-STILL-DESCRI-54A525A5
title: "Vec pop policy and docs still describe Copy-only surface after Drop move-out support"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/types.nepl, nodesrc/test_stdlib_vec_borrowed_observers.js"
---

# ISS-20260522T231155242Z-VEC-POP-POLICY-AND-DOCS-STILL-DESCRI-54A525A5: Vec pop policy and docs still describe Copy-only surface after Drop move-out support

## 概要

Vec.pop now has Copy and Drop overloads backed by VecStorageInvariant, collection_slot_move_out, and vec_pop_with owner recovery, but the root facade documentation and borrowed observer policy still describe pop as Copy-only. This stale policy can hide regressions by checking only the old Copy overload and giving future agents the wrong boundary.

## 対象

- `stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/vec/types.nepl, nodesrc/test_stdlib_vec_borrowed_observers.js`

## 根拠

- 未記入

## 問題

Vec.pop now has Copy and Drop overloads backed by VecStorageInvariant, collection_slot_move_out, and vec_pop_with owner recovery, but the root facade documentation and borrowed observer policy still describe pop as Copy-only. This stale policy can hide regressions by checking only the old Copy overload and giving future agents the wrong boundary.

## 影響

The source policy and public docs can drift away from the actual static-check contract. That is dangerous for non-Copy collection work because it can either reject the intended Drop-capable API later or encourage reintroducing accessor patterns that discard the popped Option<T> owner.

## 修正方針

Update the Vec facade/type docs and nodesrc borrowed observer policy to classify pop as an owner-consuming move-out API with Copy and Drop overloads, while keeping get, vec_pop_item, vec_pop_vec, transform, sort, and borrowed payload-copy observers Copy-only until their own owner-preserving designs exist.

## 検証

Run node --check and node tests for Vec borrowed observer/source policies, then issues check/index.

## 2026-05-22 Agent 1 修正

`Vec.pop` の Drop payload 対応後も残っていた Copy-only 前提の説明と監視文言を更新した。root facade の注意書きは、`pop` を `push` / `drop_last` / `clear` / `free` と同じ Drop-capable lifecycle API として記述し、`VecPop<T>` と `vec_pop_with<T, R>` により更新後 `Vec<T>` owner と removed `Option<T>` owner を同じ callback で回収することを明示した。

`vec/types.nepl` では、`OwnedBuffer<T>` の説明を `push` / `pop` / `replace_drop_old` が private lifecycle proof を通して `len == initialized_len` を維持する現行設計に合わせた。一方で `get`、borrowed `replace`、transform / sort、`vec_pop_item`、`vec_pop_vec` は payload copy-out または item owner discard を伴うため Copy-only のまま残す境界を明確化した。

`nodesrc/test_stdlib_vec_borrowed_observers.js` は、`pop<T: Copy>` だけではなく `pop<T: Drop>` と `vec_pop_with<T, R>` を監視し、`vec_pop_item<T: Copy>` / `vec_pop_vec<T: Copy>` が Drop payload へ広がらないことも同時に固定するように更新した。

検証:

- `node --check nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node --check nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node --check nodesrc/test_stdlib_collection_cleanup_contract.js`
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/test_stdlib_collection_cleanup_contract.js`
