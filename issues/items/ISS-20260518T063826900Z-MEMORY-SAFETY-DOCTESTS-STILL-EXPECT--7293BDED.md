---
id: ISS-20260518T063826900Z-MEMORY-SAFETY-DOCTESTS-STILL-EXPECT--7293BDED
title: "memory safety doctests still expect pre-raw-boundary MemPtr forging behavior"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: tests/stdlib/memory_safety.n.md
---

# ISS-20260518T063826900Z-MEMORY-SAFETY-DOCTESTS-STILL-EXPECT--7293BDED: memory safety doctests still expect pre-raw-boundary MemPtr forging behavior

## 概要

After raw address alias/view helpers were gated behind raw-memory-boundary source proof, several memory_safety doctests still treat mem_ptr_wrap/region_new from ordinary source as executable runtime fixtures or expect the older resource.owner.no_free_obligation diagnostic.

## 対象

- `tests/stdlib/memory_safety.n.md`

## 根拠

- `tests/stdlib/memory_safety.n.md` previously expected `mem_ptr_wrap 0` based invalid pointer fixtures to compile and return Option/Result failure values.
- After raw helper boundary proof was introduced, ordinary source is correctly rejected before runtime for mem_ptr_wrap/region_new forging attempts.
- Rewriting the positive fill fixture to public `alloc_region`/`region_ptr` provenance exposed `ISS-20260518T064507398Z-RESOURCEIR-FILLBYTES-RECORDS-THE-FIL-C96703EF`, which is fixed in the same working set because the fixture must remain a real regression for compiler proof, not a weakened test.

## 問題

After raw address alias/view helpers were gated behind raw-memory-boundary source proof, several memory_safety doctests still treat mem_ptr_wrap/region_new from ordinary source as executable runtime fixtures or expect the older resource.owner.no_free_obligation diagnostic.

## 影響

Focused memory safety verification reports 10 failures even though the compiler is correctly rejecting ordinary source raw helper use; this can hide real static-check regressions in the same suite.

## 修正方針

Update stale doctest fixtures so ordinary source raw helper forging is compile_fail with resource.raw.memory_outside_boundary, and rewrite valid fill/load coverage to use public alloc_region/region_ptr provenance instead of internal raw helper construction. Keep negative argument coverage on valid MemPtr provenance so it tests checked API behavior instead of raw helper gate behavior.

## 検証

- `trunk build`
- `node nodesrc/tests.js -i tests\\stdlib\\memory_safety.n.md --no-tree -o tmp\\agent1-memory-safety-raw-boundary-after.json -j 1 --dist web\\dist`
