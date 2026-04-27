---
id: ISS-20260427T000311744Z-BLOOMFILTER-RETAINS-UNSAFE-UNWRAP-IN-8D46F230
title: "BloomFilter retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/bloom_filter.nepl, tests/stdlib/bloom_filter_collections.n.md"
---

# ISS-20260427T000311744Z-BLOOMFILTER-RETAINS-UNSAFE-UNWRAP-IN-8D46F230: BloomFilter retains unsafe unwrap in owned bit storage cleanup

## 概要

BloomFilter.free still calls uwok on dealloc_ptr for its owned bit array while public allocation APIs expose Result.

## 対象

- `stdlib/alloc/collections/bloom_filter.nepl, tests/stdlib/bloom_filter_collections.n.md`

## 根拠

- 未記入

## 問題

BloomFilter.free still calls uwok on dealloc_ptr for its owned bit array while public allocation APIs expose Result.

## 影響

Probabilistic membership filters for self-host caches keep a trap-prone cleanup path and weaken the collection-wide unsafe-helper policy.

## 修正方針

Replace the owned bit-array cleanup with dealloc_raw, document the invariant, add a free smoke regression, and register a source guard.

## 検証

Run BloomFilter doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
