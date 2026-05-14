---
id: ISS-20260514T174112370Z-BLOOMFILTER-CLEAR-ACCEPTS-UNCONSTRAI-91A43DBC
title: "BloomFilter clear accepts unconstrained key and hasher payloads"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/bloom_filter/api.nepl, stdlib/alloc/collections/counting_bloom_filter/api.nepl, tests/stdlib/collection_cleanup_contract.n.md"
---

# ISS-20260514T174112370Z-BLOOMFILTER-CLEAR-ACCEPTS-UNCONSTRAI-91A43DBC: BloomFilter clear accepts unconstrained key and hasher payloads

## 概要

BloomFilter.clear and CountingBloomFilter.clear still use unconstrained <.T,.H> even though constructor, insert, contains, remove, and free are Copy-only around HashKey/Hasher. A forged aggregate can carry a non-Copy hasher through the mutating clear API while the current collection cleanup model has no field-level Drop traversal.

## 対象

- `stdlib/alloc/collections/bloom_filter/api.nepl, stdlib/alloc/collections/counting_bloom_filter/api.nepl, tests/stdlib/collection_cleanup_contract.n.md`

## 根拠

- `stdlib/alloc/collections/bloom_filter/api.nepl` の `clear` が `<.T,.H>` のまま `BloomFilter<.T,.H>` を消費して返していた。
- `stdlib/alloc/collections/counting_bloom_filter/api.nepl` の `clear` も同じく `<.T,.H>` のままで、`new` / `insert` / `contains` / `remove` / `free` の Copy-only contract とずれていた。
- `ISS-20260514T171024254Z-BLOOM-FILTER-FREE-DROPS-HASHER-WITHO-D98B768B` で `free` を Copy-only にしても、`clear` だけが forged non-Copy hasher aggregate を受け入れる入口として残っていた。

## 問題

BloomFilter.clear and CountingBloomFilter.clear still use unconstrained <.T,.H> even though constructor, insert, contains, remove, and free are Copy-only around HashKey/Hasher. A forged aggregate can carry a non-Copy hasher through the mutating clear API while the current collection cleanup model has no field-level Drop traversal.

## 影響

This leaves a remaining Stage 6 collection cleanup gap after the free bounds fix: unsupported non-Copy hasher payloads can still be accepted by a consuming collection API, weakening the Copy-only boundary that protects the current raw-memory-backed implementation.

## 修正方針

Require the same <.T: HashKey&Copy,.H: Hasher<.T>&Copy> bounds on both clear APIs, add compile-fail regressions with a non-Copy StatefulHasher, and update source policy so clear cannot regress to unconstrained generics.

## 検証

Run bloom/counting source policies, collection cleanup contract doctests, and focused bloom filter runtime doctests.

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 解決内容

2026-05-15 Agent 1:

- `BloomFilter.clear` を `<.T: HashKey&Copy,.H: Hasher<.T>&Copy>` に変更し、constructor / insert / contains / free と同じ key/hasher contract へ揃えた。
- `CountingBloomFilter.clear` も `<.T: HashKey&Copy,.H: Hasher<.T>&Copy>` に変更し、remove を含む mutating API 群と同じ contract へ揃えた。
- `tests/stdlib/collection_cleanup_contract.n.md` に、`Hasher<i32>` は実装するが `Copy` を実装しない `StatefulHasher` で両 clear が `type.trait_bound.unsatisfied` になる regression を追加した。
- BloomFilter / CountingBloomFilter の source policy に、`clear` が unconstrained `<.T,.H>` へ戻らないことを追加した。

## 解決後の検証

- `node nodesrc/test_stdlib_bloom_filter_no_unsafe_unwraps.js`: pass
- `node nodesrc/test_stdlib_counting_bloom_filter_no_unsafe_unwraps.js`: pass
- `node nodesrc/tests.js -i tests/stdlib/collection_cleanup_contract.n.md --no-tree -o tmp/agent1-bloom-clear-copy-contract.json -j 1 --dist web/dist --assert-io`: 16/16 pass
- `node nodesrc/tests.js -i stdlib/tests/bloom_filter.n.md -i stdlib/tests/counting_bloom_filter.n.md -i tests/stdlib/bloom_filter_collections.n.md -i tests/stdlib/counting_bloom_filter_collections.n.md --no-tree -o tmp/agent1-bloom-clear-runtime-contract.json -j 1 --dist web/dist --assert-io`: 9/9 pass
