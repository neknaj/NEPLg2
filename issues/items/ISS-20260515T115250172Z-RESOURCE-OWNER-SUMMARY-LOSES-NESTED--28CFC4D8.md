---
id: ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8
title: "Resource owner summary loses nested owner payload through result unwrapping helper"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/**, stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**"
---

# ISS-20260515T115250172Z-RESOURCE-OWNER-SUMMARY-LOSES-NESTED--28CFC4D8: Resource owner summary loses nested owner payload through result unwrapping helper

## 概要

A helper that matches Result<BTreeMap, BTreeMapInsertError> or Result<BTreeSet, BTreeSetInsertError> and returns the collection owner from either Ok or Err is rejected with resource.owner.leak/maybe_leak, even though the same match in the caller is accepted. This indicates ResourceIR owner summaries do not fully preserve nested owner projections through helper returns from owner-bearing enum payloads.

## 対象

- `nepl-core/src/resource/**, stdlib/alloc/collections/btreemap/**, stdlib/alloc/collections/btreeset/**`

## 根拠

- 2026-05-15 の `BTreeMapInsertError` / `BTreeSetInsertError` 導入中に、呼び出し側で直接 `match insert ...` して `Ok` / `Err.owner` のどちらかを返す最小ケースは ResourceIR を通過した。
- 同じ分岐を `fn must_map(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>) -> BTreeMap<i32,i32>` / `fn must_set(...) -> BTreeSet<i32>` に閉じると、caller 側で `resource.owner.leak` または `resource.owner.maybe_leak` が出た。
- `VecPushError<T>` の単純 owner payload helper は通過するため、問題は owner-bearing Result 一般ではなく、`BTreeMapStorage -> Vec<Option<T>> -> RegionToken<T>` のような入れ子 owner projection を helper summary が保持しきれていない可能性が高い。

## 問題

A helper that matches Result<BTreeMap, BTreeMapInsertError> or Result<BTreeSet, BTreeSetInsertError> and returns the collection owner from either Ok or Err is rejected with resource.owner.leak/maybe_leak, even though the same match in the caller is accepted. This indicates ResourceIR owner summaries do not fully preserve nested owner projections through helper returns from owner-bearing enum payloads.

## 影響

Safe reusable helpers for owner-preserving fallible updates cannot be written reliably for nested owner aggregates. Users are forced to inline matches or use unwrap-style success helpers, which weakens maintainability and hides a ResourceIR composition gap.

## 修正方針

Extend Resource owner return/variant summaries so nested owner projections returned from matched enum payloads are propagated through helper function summaries, not only direct caller matches. Add regression tests using BTreeMapInsertError/BTreeSetInsertError-style nested owners.

## 検証

Add nepl-core ResourceIR regression for a helper that unwraps Result<Collection, InsertError> by returning the owner from both Ok and Err branches, plus stdlib doctest/source-policy coverage when the core fix lands.
