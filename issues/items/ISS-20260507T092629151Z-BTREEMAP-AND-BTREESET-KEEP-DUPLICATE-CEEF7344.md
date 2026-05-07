---
id: ISS-20260507T092629151Z-BTREEMAP-AND-BTREESET-KEEP-DUPLICATE-CEEF7344
title: "BTreeMap and BTreeSet keep duplicate by-value and *_ref observer APIs"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md"
---

# ISS-20260507T092629151Z-BTREEMAP-AND-BTREESET-KEEP-DUPLICATE-CEEF7344: BTreeMap and BTreeSet keep duplicate by-value and *_ref observer APIs

## 概要

BTreeMap and BTreeSet still expose len/contains/get as by-value terminal observers while len_ref/contains_ref/get_ref provide the owner-preserving form. This leaves two names for the same read-only operation and keeps old owner-consuming call style alive.

## 対象

- `stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreeset.nepl, stdlib/tests/btreemap.n.md, stdlib/tests/btreeset.n.md`

## 根拠

- 未記入

## 問題

BTreeMap and BTreeSet still expose len/contains/get as by-value terminal observers while len_ref/contains_ref/get_ref provide the owner-preserving form. This leaves two names for the same read-only operation and keeps old owner-consuming call style alive.

## 影響

Selfhost symbol tables and sets can accidentally move map/set owners for lookups, and fixtures must choose between outdated by-value observers and borrowed *_ref variants. Static memory-safety checks are clearer when read-only observers have one borrowed surface.

## 修正方針

Make the primary map/set observer names borrow the owner, remove duplicate *_ref observers, and update focused BTree tests plus source policy.

## 検証

node nodesrc/tests.js -i stdlib/alloc/collections/btreemap.nepl -i stdlib/alloc/collections/btreeset.nepl -i stdlib/tests/btreemap.n.md -i stdlib/tests/btreeset.n.md --no-tree -o tmp/btree-primary-borrowed-observers.json -j 1 --dist web/dist
