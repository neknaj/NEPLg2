---
id: ISS-20260513T144754414Z-BLOOM-FILTER-HASH-MODULES-RELY-ON-TR-52B8FBB4
title: "Bloom filter hash modules rely on transitive hash32 mix import"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "stdlib/alloc/collections/bloom_filter/hash.nepl, stdlib/alloc/collections/counting_bloom_filter/hash.nepl, nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js, nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js"
---

# ISS-20260513T144754414Z-BLOOM-FILTER-HASH-MODULES-RELY-ON-TR-52B8FBB4: Bloom filter hash modules rely on transitive hash32 mix import

## 概要

BloomFilter and CountingBloomFilter hash submodules call mix without importing the hash32 module that defines it. The call only works when another import happens to leak mix transitively, so adding std/test to focused doctests can make the same module fail with resolve.identifier.undefined.

## 対象

- `stdlib/alloc/collections/bloom_filter/hash.nepl, stdlib/alloc/collections/counting_bloom_filter/hash.nepl, nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js, nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`

## 根拠

- `stdlib/alloc/collections/bloom_filter/hash.nepl` と `stdlib/alloc/collections/counting_bloom_filter/hash.nepl` は `mix xor h0 hk` を呼んでいたが、`mix` を定義する `alloc/hash/hash32` を直接 import していなかった。
- `std/test` を追加した BloomFilter focused doctest で、`/stdlib/alloc/collections/bloom_filter/hash.nepl:20:21` の `mix` が `resolve.identifier.undefined` になった。
- counting variant も同じ構造だったため、同時に明示依存へ揃える必要がある。

## 問題

BloomFilter and CountingBloomFilter hash submodules call mix without importing the hash32 module that defines it. The call only works when another import happens to leak mix transitively, so adding std/test to focused doctests can make the same module fail with resolve.identifier.undefined.

## 影響

Submodule correctness depends on caller import shape instead of explicit module dependencies. This weakens facade split boundaries and makes doctest/report migration expose unrelated compile failures.

## 修正方針

Import alloc/hash/hash32 explicitly in each hash submodule and call hash32::mix through a qualified name. Extend source policy tests so the explicit dependency cannot regress.

## 検証

Run bloom/counting bloom source policies and focused doctests.

## 修正結果

- `bloom_filter/hash.nepl` と `counting_bloom_filter/hash.nepl` が `alloc/hash/hash32` を `hash32` alias で明示 import し、secondary hash mixing を `hash32::mix` で呼ぶようにした。
- `nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js` と `nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js` に、hash submodule が `hash32` を明示 import し qualified call することを固定する regression を追加した。

検証:

- `node nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/tests/bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md --no-tree -o tmp/agent1-bloom-filter-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4
- `node nodesrc/tests.js -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/agent1-counting-bloom-filter-hash-import-tests.json -j 1 --dist web/dist`: total=5, passed=5
