---
id: ISS-20260604T034125561Z-BTREEMAP-AND-BTREESET-NAMES-DESCRIBE-4608BFC5
title: "BTreeMap and BTreeSet names describe a tree abstraction but implementation is sorted array"
area: stdlib
status: open
resolved: false
priority: P2
type: architecture
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreemap/types.nepl, stdlib/alloc/collections/btreeset/types.nepl"
---

# ISS-20260604T034125561Z-BTREEMAP-AND-BTREESET-NAMES-DESCRIBE-4608BFC5: BTreeMap and BTreeSet names describe a tree abstraction but implementation is sorted array

## 概要

Subagent audit found BTreeMap/BTreeSet public names while the implementation and docs describe sorted array storage. Zenn guidance treats names, directory hierarchy, and abstraction cost as part of the contract; a B-tree name implies different complexity and structure than sorted array.

## 対象

- `stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/btreemap/types.nepl, stdlib/alloc/collections/btreeset/types.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found BTreeMap/BTreeSet public names while the implementation and docs describe sorted array storage. Zenn guidance treats names, directory hierarchy, and abstraction cost as part of the contract; a B-tree name implies different complexity and structure than sorted array.

## 影響

Users and future maintainers may rely on B-tree complexity or mutation behavior that the implementation does not provide, and tests may miss performance regressions hidden by the mismatched name.

## 修正方針

Either rename the primary public abstraction to SortedArrayMap/SortedArraySet with compatibility aliases, or implement an actual B-tree behind the existing names. Document complexity and migration policy.

## 検証

Add regular tests/bench-style checks for large insert/remove/order behavior and source policy that contract names match documented storage strategy.
