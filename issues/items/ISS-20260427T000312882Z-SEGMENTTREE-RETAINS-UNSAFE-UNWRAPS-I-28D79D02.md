---
id: ISS-20260427T000312882Z-SEGMENTTREE-RETAINS-UNSAFE-UNWRAPS-I-28D79D02
title: "SegmentTree retains unsafe unwraps in owned tree storage"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/segment_tree.nepl, tests/stdlib/segment_tree_collections.n.md"
---

# ISS-20260427T000312882Z-SEGMENTTREE-RETAINS-UNSAFE-UNWRAPS-I-28D79D02: SegmentTree retains unsafe unwraps in owned tree storage

## 概要

SegmentTree uses uwok for owned array stores and free cleanup.

## 対象

- `stdlib/alloc/collections/segment_tree.nepl, tests/stdlib/segment_tree_collections.n.md`

## 根拠

- 未記入

## 問題

SegmentTree uses uwok for owned array stores and free cleanup.

## 影響

Range-query helpers for self-host analysis can still trap in normal internal storage paths and weaken collection consistency.

## 修正方針

Introduce raw owned-array store helper, replace owned cleanup with dealloc_raw, add update/query/free regression coverage, and add a source guard.

## 検証

Run SegmentTree doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
