---
id: ISS-20260513T113909985Z-VEC-POP-ACCEPTS-NON-COPY-PAYLOAD-BEF-6A1C568A
title: "Vec.pop accepts non-Copy payload before initialized slot move-state exists"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: stdlib/alloc/collections/vec/mutation/pop.nepl
---

# ISS-20260513T113909985Z-VEC-POP-ACCEPTS-NON-COPY-PAYLOAD-BEF-6A1C568A: Vec.pop accepts non-Copy payload before initialized slot move-state exists

## 概要

Vec.pop<T> reads the tail element with raw typed load and returns VecPop<T>. For non-Copy payload this is a move-out from an initialized storage slot, but current Vec storage only has VecStorageState::Empty/Owned and a MemPtr<T>; it cannot mark the removed cell as moved/uninitialized or prove the remaining storage free/drop obligation for non-Copy elements.

## 対象

- `stdlib/alloc/collections/vec/mutation/pop.nepl`

## 根拠

- `Vec.pop` は `vec_raw::vec_read_at<T>` を通じて raw typed `load<T>` で末尾要素を一時値へ読み出す。
- 現行 `Vec<T>` は `VecStorageState::Empty/Owned` と `MemPtr<T>` だけを持ち、slot ごとの initialized / moved / uninitialized 状態を型にも Resource IR にも渡せない。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の Stage D は、non-Copy payload の move-out を `OwnedBuffer<T>` と initialized prefix/cell state 上で扱う方針にしている。
- `VecPop<T>` は owner-bearing result として正しい方向だが、取り出した non-Copy element owner と残り storage owner の drop/free obligation を分離して証明するにはまだ state model が不足している。

## 問題

Vec.pop<T> reads the tail element with raw typed load and returns VecPop<T>. For non-Copy payload this is a move-out from an initialized storage slot, but current Vec storage only has VecStorageState::Empty/Owned and a MemPtr<T>; it cannot mark the removed cell as moved/uninitialized or prove the remaining storage free/drop obligation for non-Copy elements.

## 影響

A non-Copy element can be moved out while the backing storage still looks like an unchanged Owned Vec region to Resource IR and stdlib contracts. This weakens memory safety by making initialized-cell and drop/free obligations depend on caller discipline instead of a compiler-proven state model.

## 修正方針

Until OwnedBuffer<T> plus initialized prefix/cell move state is implemented, require T: Copy on Vec.pop. Keep VecPop as the owner-bearing result, document the temporary Copy-only boundary, add a compile-fail regression for a non-Copy payload, and link the remaining full non-Copy design to RV-STDLIB-004/OwnedBuffer Stage D.

## 検証

Run the Vec source policy checks, focused Vec.pop doctests, Vec doctests, and issues check.

## 修正内容

- `Vec.pop` を `.T: Copy` に限定した。
- `Vec.pop` の doc comment に、`OwnedBuffer<T>` / initialized cell の moved/uninitialized 状態が入るまで non-Copy move-out を許可しない理由を明記した。
- `Vec<NonCopyPayload>` の `pop` が `type.trait_bound.unsatisfied` で compile-fail になる doctest を追加した。
- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_vec_borrowed_observers.js` の source policy を、`Vec.pop` が owner-bearing `VecPop<T>` を返しつつ Copy-only 境界であることを監視する形へ更新した。
- `doc/neplg2/stdlib_collection_mem_string_static_safety_design.md` の `Vec` 進捗を、`push` / `pop` / `sort` / cleanup が Copy-only 境界になった現状へ更新した。

## 検証結果

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_vec_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/mutation/pop.nepl --no-tree -o tmp/agent1-vec-pop-copy-bound-pop.json -j 1 --dist web/dist`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec --no-tree -o tmp/agent1-vec-pop-copy-bound-vec.json -j 4 --dist web/dist`: total=35, passed=35

## 親issueとの関係

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828` の Stage D 残件のうち、raw typed load による non-Copy move-out の入口を閉じた。
- full non-Copy collection support はこの issue では完了扱いにしない。`OwnedBuffer<T>`、initialized prefix、slot state transition、drop traversal、owner-preserving remove/pop API は親 issue で継続する。
