---
id: ISS-20260430T024858788Z-BLOOMFILTER-READ-ONLY-APIS-CONSUME-O-B8202860
title: "BloomFilter read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/bloom_filter.nepl
---

# ISS-20260430T024858788Z-BLOOMFILTER-READ-ONLY-APIS-CONSUME-O-B8202860: BloomFilter read-only APIs consume owners by value instead of borrowing

## 概要

BloomFilter len/contains take BloomFilter by value while only reading nbits, bit storage and hasher. This consumes the filter owner during observation and prevents the caller from freeing the bit storage afterwards.

## 対象

- `stdlib/alloc/collections/bloom_filter.nepl`

## 根拠

- `stdlib/alloc/collections/bloom_filter.nepl` に `fn len <.T,.H> <(BloomFilter<.T,.H>)->i32>` と `fn contains <.T: HashKey&Copy,.H: Hasher<.T>&Copy> <(BloomFilter<.T,.H>,.T)->bool>` が残っている。
- BitSet の owner-consuming observer 修正中に sibling collection を確認し、bit storage と hasher を読むだけの BloomFilter observer も値 receiver のままだと判明した。

## 問題

BloomFilter len/contains take BloomFilter by value while only reading nbits, bit storage and hasher. This consumes the filter owner during observation and prevents the caller from freeing the bit storage afterwards.

## 影響

BloomFilter users must choose between leaking the filter owner or bypassing the public observer API. This is incompatible with the memory-safety policy and blocks self-host use of BloomFilter as a normal collection.

## 修正方針

Change BloomFilter observer APIs to take &BloomFilter, copy only Copy fields from borrowed references, update doctests/tests, and keep mutating APIs as owner-consuming update functions.

## 検証

Add tests that insert values, query len/contains through &BloomFilter, query more than once, then free the same owner cleanly.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/bloom_filter.n.md --no-tree -o tmp/bloom-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/bloom_filter_collections.n.md --no-tree -o tmp/bloom-collections-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/bloom_filter.nepl --no-tree -o tmp/bloom-doctest-borrowed-observers.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/test_stdlib_bloom_filter_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `BloomFilter.len` と `BloomFilter.contains` を `&BloomFilter<.T,.H>` receiver に変更し、読み取りで owner を移動しない公開 API にした。
- BloomFilter doctest / `.n.md` tests を、borrowed observer の後で同じ owner を `free` する形に直した。
- `nodesrc/test_stdlib_bloom_filter_borrowed_observers.js` を追加し、by-value observer signature と by-value test usage が戻らないよう source policy に登録した。
