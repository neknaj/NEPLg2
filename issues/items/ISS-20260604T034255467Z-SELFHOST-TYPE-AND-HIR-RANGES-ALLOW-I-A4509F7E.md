---
id: ISS-20260604T034255467Z-SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-I-A4509F7E
title: "selfhost Type and HIR ranges allow invalid raw i32 count invariants"
area: selfhost
status: open
resolved: false
priority: P1
type: bug
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/hir/hir/range.nepl, stdlib/neplg2/core/ty/ty/record.nepl, stdlib/neplg2/core/ty/ty/eq.nepl"
---

# ISS-20260604T034255467Z-SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-I-A4509F7E: selfhost Type and HIR ranges allow invalid raw i32 count invariants

## 概要

Subagent audit found HIR and type range constructors storing first/count as raw i32 without checked construction, and equality logic can treat negative counts as trivially equal. This violates the Zenn policy that invalid states should be excluded with typed constructors and Result.

## 対象

- `stdlib/neplg2/core/hir/hir/range.nepl, stdlib/neplg2/core/ty/ty/record.nepl, stdlib/neplg2/core/ty/ty/eq.nepl`

## 根拠

- 未記入

## 問題

Subagent audit found HIR and type range constructors storing first/count as raw i32 without checked construction, and equality logic can treat negative counts as trivially equal. This violates the Zenn policy that invalid states should be excluded with typed constructors and Result.

## 影響

Negative or overflowing ranges can flow into arena/type comparisons and silently produce valid-looking equality results, undermining later static checks and diagnostics.

## 修正方針

Introduce checked range constructors returning Result or a validated range type, reject negative count/overflow/out-of-arena inputs, and update callers to match on typed errors.

## 検証

Add regular tests for negative count, zero count, overflow, out-of-arena count, and equality of invalid ranges.
