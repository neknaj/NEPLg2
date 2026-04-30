---
id: ISS-20260430T022009877Z-BITSET-READ-ONLY-APIS-CONSUME-OWNERS-103D989F
title: "BitSet read-only APIs consume owners by value instead of borrowing"
area: stdlib
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: stdlib/alloc/collections/bitset.nepl
---

# ISS-20260430T022009877Z-BITSET-READ-ONLY-APIS-CONSUME-OWNERS-103D989F: BitSet read-only APIs consume owners by value instead of borrowing

## 概要

BitSet len/contains style read-only APIs take BitSet by value, so callers that only inspect the collection either move the owner and cannot free it afterwards, or leave an owner leak when tests end after the read. This conflicts with the memory-safety policy and makes static checks report real leaks in otherwise read-only code.

## 対象

- `stdlib/alloc/collections/bitset.nepl`

## 根拠

- 未記入

## 問題

BitSet len/contains style read-only APIs take BitSet by value, so callers that only inspect the collection either move the owner and cannot free it afterwards, or leave an owner leak when tests end after the read. This conflicts with the memory-safety policy and makes static checks report real leaks in otherwise read-only code.

## 影響

Collection users must choose between by-value observer calls that consume ownership and ad hoc field reads that bypass the public API. Self-host stdlib code will either leak owned buffers or encode unnatural workarounds instead of using checked borrowed observers.

## 修正方針

Redesign BitSet observer APIs around borrowed receivers such as &BitSet, update examples/tests to call borrowed observers, and review sibling collection observer APIs for the same by-value owner pattern.

## 検証

Add compile/run tests that read BitSet length and membership through borrowed APIs, then explicitly free the same BitSet without move-check or owner-check diagnostics.
