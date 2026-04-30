---
id: ISS-20260430T024859180Z-COUNTINGBLOOMFILTER-READ-ONLY-APIS-C-20B1FC21
title: "CountingBloomFilter read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/counting_bloom_filter.nepl
---

# ISS-20260430T024859180Z-COUNTINGBLOOMFILTER-READ-ONLY-APIS-C-20B1FC21: CountingBloomFilter read-only APIs consume owners by value instead of borrowing

## 概要

CountingBloomFilter len/contains take CountingBloomFilter by value even though they only inspect counters, nslots and hasher. Read-only membership checks therefore consume the owner and leave no natural path to free the counter storage.

## 対象

- `stdlib/alloc/collections/counting_bloom_filter.nepl`

## 根拠

- `stdlib/alloc/collections/counting_bloom_filter.nepl` に `fn len <.T,.H> <(CountingBloomFilter<.T,.H>)->i32>` と `fn contains <.T: HashKey&Copy,.H: Hasher<.T>&Copy> <(CountingBloomFilter<.T,.H>,.T)->bool>` が残っている。
- BitSet の owner-consuming observer 修正中に sibling collection を確認し、counter storage と hasher を読むだけの CountingBloomFilter observer も値 receiver のままだと判明した。

## 問題

CountingBloomFilter len/contains take CountingBloomFilter by value even though they only inspect counters, nslots and hasher. Read-only membership checks therefore consume the owner and leave no natural path to free the counter storage.

## 影響

CountingBloomFilter cannot be used safely in code that performs membership checks before cleanup. Static owner checking reports real leaks unless callers avoid the public API.

## 修正方針

Redesign CountingBloomFilter observer APIs to take &CountingBloomFilter, update examples/tests to borrow, and remove by-value observer entry points so the static checker enforces the intended ownership contract.

## 検証

Add tests that perform borrowed len/contains checks after insert/remove operations and then explicitly free the same CountingBloomFilter owner.

確認済み:

- `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md --no-tree -o tmp/counting-bloom-stdlib-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/counting-bloom-collections-borrowed-observers.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/alloc/collections/counting_bloom_filter.nepl --no-tree -o tmp/counting-bloom-doctest-borrowed-observers.json -j 1` (`total=6`, `passed=6`, `failed=0`)
- `node nodesrc/test_stdlib_counting_bloom_filter_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`: passed
- `node nodesrc/issues.js check`: passed

## 修正内容

- `CountingBloomFilter.len` と `CountingBloomFilter.contains` を `&CountingBloomFilter<.T,.H>` receiver に変更し、読み取りで owner を移動しない公開 API にした。
- CountingBloomFilter doctest / `.n.md` tests を、borrowed observer の後で同じ owner を `free` する形に直した。
- `nodesrc/test_stdlib_counting_bloom_filter_borrowed_observers.js` を追加し、by-value observer signature と by-value test usage が戻らないよう source policy に登録した。
