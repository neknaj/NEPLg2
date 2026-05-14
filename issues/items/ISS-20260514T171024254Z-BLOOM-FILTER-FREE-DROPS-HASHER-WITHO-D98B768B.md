---
id: ISS-20260514T171024254Z-BLOOM-FILTER-FREE-DROPS-HASHER-WITHO-D98B768B
title: "Bloom filter free drops hasher without Copy or Drop contract"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-15
target: "stdlib/alloc/collections/bloom_filter/api.nepl, stdlib/alloc/collections/counting_bloom_filter/api.nepl"
---

# ISS-20260514T171024254Z-BLOOM-FILTER-FREE-DROPS-HASHER-WITHO-D98B768B: Bloom filter free drops hasher without Copy or Drop contract

## 概要

BloomFilter<T,H>.free and CountingBloomFilter<T,H>.free free their internal Vec<u8> storage but accept unconstrained H. The consumed hasher field is then discarded without either Copy-only proof or Drop traversal, so the public cleanup boundary looks safe for non-Copy owner-bearing hashers that the current collection model cannot drop.

## 対象

- `stdlib/alloc/collections/bloom_filter/api.nepl, stdlib/alloc/collections/counting_bloom_filter/api.nepl`

## 根拠

- `BloomFilter<T,H>` / `CountingBloomFilter<T,H>` は `hasher <.H>` を field に持つ。
- `new` / `insert` / `contains` / `remove` は `.H: Hasher<.T>&Copy` を要求し、hasher を複数回 by-value に読む前提を型で表している。
- 一方、修正前の `free <.T,.H>` は内部 `Vec<u8>` だけを解放し、消費した owner 内の hasher field について `Copy` でも `Drop` でもない型を拒否していなかった。

## 問題

BloomFilter<T,H>.free and CountingBloomFilter<T,H>.free free their internal Vec<u8> storage but accept unconstrained H. The consumed hasher field is then discarded without either Copy-only proof or Drop traversal, so the public cleanup boundary looks safe for non-Copy owner-bearing hashers that the current collection model cannot drop.

## 影響

A direct or inferred BloomFilter with non-Copy H can cross a cleanup API that does not discharge H ownership. This keeps the collection cleanup contract inconsistent with the current Copy-only collection model and can hide leaks until Resource IR / OwnedBuffer based drop traversal is complete.

## 修正方針

Restrict both BloomFilter.free and CountingBloomFilter.free to .T: HashKey&Copy and .H: Hasher<.T>&Copy, mirroring constructor/query/update APIs. Add compile-fail regressions for non-Copy hashers and source policy checks so this boundary cannot regress to an unconstrained generic free.

## 検証

Run focused stdlib collection cleanup doctests, BloomFilter/CountingBloomFilter source policy checks, issue index validation, and diff whitespace checks.

## 解決内容

`BloomFilter.free` と `CountingBloomFilter.free` を `.T: HashKey&Copy,.H: Hasher<.T>&Copy` に制限した。これにより、現行 collection cleanup が Drop traversal を持たない間は、free 境界で non-Copy hasher を受け入れない。

`tests/stdlib/collection_cleanup_contract.n.md` に、`Hasher<i32>` は実装しているが `Copy` を実装しない `StatefulHasher` で両 free が `type.trait_bound.unsatisfied` になる compile-fail regression を追加した。source policy も、unconstrained `free <.T,.H>` への退行を拒否するように更新した。

## 関連

- Parent: `ISS-20260425T000000Z-RV-STDLIB-004-91534828`
- Doc: `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 6
