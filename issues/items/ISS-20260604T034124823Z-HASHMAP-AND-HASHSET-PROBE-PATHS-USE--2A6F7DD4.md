---
id: ISS-20260604T034124823Z-HASHMAP-AND-HASHSET-PROBE-PATHS-USE--2A6F7DD4
title: "HashMap and HashSet probe paths use -1 sentinel instead of typed absence"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/alloc/collections/hashmap/probe.nepl, stdlib/alloc/collections/hashmap/api.nepl, stdlib/alloc/collections/hashset/api.nepl"
---

# ISS-20260604T034124823Z-HASHMAP-AND-HASHSET-PROBE-PATHS-USE--2A6F7DD4: HashMap and HashSet probe paths use -1 sentinel instead of typed absence

## 概要

Subagent audit found HashMap/HashSet probing returning and checking -1 as an absence/sentinel value. Zenn guidance prefers Option/Result/enum plus match, because sentinel integers cannot distinguish not-found, tombstone, full table, and corrupted storage states statically.

## 対象

- `stdlib/alloc/collections/hashmap/probe.nepl, stdlib/alloc/collections/hashmap/api.nepl, stdlib/alloc/collections/hashset/api.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found HashMap/HashSet probing returning and checking -1 as an absence/sentinel value. Zenn guidance prefers Option/Result/enum plus match, because sentinel integers cannot distinguish not-found, tombstone, full table, and corrupted storage states statically.

## 影響

Collection invariants are harder to verify, collision/tombstone logic can collapse distinct states, and future non-Copy/drop-capable map storage cannot reliably recover owner state on failure.

## 修正方針

Introduce Option i32 or a ProbeResult enum such as Found/Empty/Tombstone/Full, and return Result where storage invariants are violated. Migrate API callers to match on the typed result.

## 検証

Add collision, tombstone remove, reinsertion, full-table, and not-found regular tests, plus source policy rejecting public -1 probe sentinel checks.
