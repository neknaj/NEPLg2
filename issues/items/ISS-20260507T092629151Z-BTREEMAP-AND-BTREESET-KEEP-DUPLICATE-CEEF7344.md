---
id: ISS-20260507T092629151Z-BTREEMAP-AND-BTREESET-KEEP-DUPLICATE-CEEF7344
title: "BTreeMap and BTreeSet keep duplicate by-value and *_ref observer APIs"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md, tests/stdlib/pipe_collections.n.md, nodesrc/test_stdlib_btree_borrowed_observers.js"
---

# ISS-20260507T092629151Z-BTREEMAP-AND-BTREESET-KEEP-DUPLICATE-CEEF7344: BTreeMap and BTreeSet keep duplicate by-value and *_ref observer APIs

## 概要

BTreeMap and BTreeSet still expose len/contains/get as by-value terminal observers while len_ref/contains_ref/get_ref provide the owner-preserving form. This leaves two names for the same read-only operation and keeps old owner-consuming call style alive.

## 対象

- `stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md, tests/stdlib/pipe_collections.n.md, nodesrc/test_stdlib_btree_borrowed_observers.js`

## 根拠

- `BTreeMap.len` / `contains` / `get` が owner を値で受け取り、内部で `free` していた。
- 同じ読み取り機能を `len_ref` / `contains_ref` / `get_ref` が借用版として重複提供していた。
- `BTreeSet.len` / `contains` も同じ by-value observer と `*_ref` の重複 surface を持っていた。
- focused stdlib tests と pipe collection tests に古い by-value observer / `*_ref` 呼び出しが残っていた。

## 問題

BTreeMap and BTreeSet still expose len/contains/get as by-value terminal observers while len_ref/contains_ref/get_ref provide the owner-preserving form. This leaves two names for the same read-only operation and keeps old owner-consuming call style alive.

## 影響

Selfhost symbol tables and sets can accidentally move map/set owners for lookups, and fixtures must choose between outdated by-value observers and borrowed *_ref variants. Static memory-safety checks are clearer when read-only observers have one borrowed surface.

## 修正方針

Make the primary map/set observer names borrow the owner, remove duplicate *_ref observers, and update focused BTree tests plus source policy.

## 検証

- `node nodesrc/test_stdlib_btree_borrowed_observers.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md -i tests/stdlib/pipe_collections.n.md --no-tree -o tmp/btree-primary-borrowed-observers.json -j 1 --dist web/dist`: total=33, passed=33

## 対応結果

- `BTreeMap.len` / `contains` / `get` を `&BTreeMap<K,V>` receiver の primary borrowed observer に変更した。
- `BTreeSet.len` / `contains` を `&BTreeSet<T>` receiver の primary borrowed observer に変更した。
- `BTreeMap.len_ref` / `contains_ref` / `get_ref` と `BTreeSet.len_ref` / `contains_ref` を削除した。
- BTreeMap/BTreeSet doctest と pipe collection test を明示 borrow/free に更新した。
- `nodesrc/test_stdlib_btree_borrowed_observers.js` を追加し、by-value observer と `*_ref` 再導入を拒否する regression を source policy に組み込んだ。
