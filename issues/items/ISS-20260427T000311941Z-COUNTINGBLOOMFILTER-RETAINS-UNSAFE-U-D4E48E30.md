---
id: ISS-20260427T000311941Z-COUNTINGBLOOMFILTER-RETAINS-UNSAFE-U-D4E48E30
title: "CountingBloomFilter retains unsafe unwrap in owned counter cleanup"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/counting_bloom_filter.nepl, tests/stdlib/counting_bloom_filter_collections.n.md"
---

# ISS-20260427T000311941Z-COUNTINGBLOOMFILTER-RETAINS-UNSAFE-U-D4E48E30: CountingBloomFilter retains unsafe unwrap in owned counter cleanup

## 概要

CountingBloomFilter.free still calls uwok on dealloc_ptr for the owned counter array.

## 対象

- `stdlib/alloc/collections/counting_bloom_filter.nepl, tests/stdlib/counting_bloom_filter_collections.n.md`

## 根拠

- 未記入

## 問題

CountingBloomFilter.free still calls uwok on dealloc_ptr for the owned counter array.

## 影響

Counting filter cleanup remains inconsistent with Result-returning constructors and can hide ownership invariant bugs behind unreachable traps.

## 修正方針

Use dealloc_raw for owned counter storage, document the invariant, add a cleanup regression, and prevent unsafe helpers from returning to the implementation.

## 検証

Run CountingBloomFilter doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
