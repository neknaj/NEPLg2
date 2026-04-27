---
id: ISS-20260427T000311423Z-BITSET-RETAINS-UNSAFE-UNWRAP-IN-OWNE-60B0934D
title: "BitSet retains unsafe unwrap in owned bit storage cleanup"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/bitset.nepl, tests/stdlib/bitset_collections.n.md"
---

# ISS-20260427T000311423Z-BITSET-RETAINS-UNSAFE-UNWRAP-IN-OWNE-60B0934D: BitSet retains unsafe unwrap in owned bit storage cleanup

## 概要

BitSet.free still calls uwok on dealloc_ptr for storage owned by the BitSet value, so the normal cleanup path depends on an unsafe Result helper instead of the owner invariant.

## 対象

- `stdlib/alloc/collections/bitset.nepl, tests/stdlib/bitset_collections.n.md`

## 根拠

- 未記入

## 問題

BitSet.free still calls uwok on dealloc_ptr for storage owned by the BitSet value, so the normal cleanup path depends on an unsafe Result helper instead of the owner invariant.

## 影響

Self-host set membership helpers can turn cleanup invariant regressions into unreachable traps, and RV-STDLIB-010 cannot be closed while collection internals keep unsafe helpers.

## 修正方針

Replace owned bit storage cleanup with dealloc_raw, document the owner invariant, add a focused free regression, and add a source guard that prevents unsafe unwrap helpers from returning to BitSet implementation.

## 検証

Run BitSet doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
