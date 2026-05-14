---
id: ISS-20260514T042244722Z-KP-FOCUSED-DOCTESTS-ARE-STALE-AFTER--DAB8C87D
title: "kp focused doctests are stale after streamio owner and allocator module split"
area: TEST
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "tests/stdlib/kp.n.md, stdlib/kp/kpsearch.nepl"
---

# ISS-20260514T042244722Z-KP-FOCUSED-DOCTESTS-ARE-STALE-AFTER--DAB8C87D: kp focused doctests are stale after streamio owner and allocator module split

## 概要

tests/stdlib/kp.n.md currently fails independently of the StreamWriter close facade fix. Doctest#1 opens a StreamScanner owner and borrows it through read &sc but never consumes it with close, so Resource IR correctly reports resource.owner.leak. Doctest#3 and doctest#7 still import core/mem while using alloc/dealloc, but current stdlib keeps those APIs in core/mem/allocator, so they fail with resolve.identifier.undefined before exercising kp prefix/search behavior.

## 対象

- `tests/stdlib/kp.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-streamwriter-close-kp.json -j 1 --dist web/dist --assert-io`: total=7, passed=4, failed=3。
- `tests/stdlib/kp.n.md::doctest#1` は `let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;` で owner を作り、`read &sc` で借用した後に `close sc` を呼んでいないため、`resource.owner.leak` になる。
- `tests/stdlib/kp.n.md::doctest#3` と `doctest#7` は `alloc` / `dealloc` を使うが、current `stdlib/core/mem.nepl` は allocator module を re-export しておらず、`stdlib/core/mem/allocator.nepl` に API があるため `resolve.identifier.undefined` になる。
- allocator import を直すと、同じ fixture が `fill_i32` / `store_i32` / `load_i32` を ordinary doctest から呼び、`resource.raw.memory_outside_boundary` で正しく拒否された。fixture を raw boundary に逃がすのではなく、safe `Vec<i32>` API で同じ入出力を表す必要がある。

## 問題

tests/stdlib/kp.n.md currently fails independently of the StreamWriter close facade fix. Doctest#1 opens a StreamScanner owner and borrows it through read &sc but never consumes it with close, so Resource IR correctly reports resource.owner.leak. Doctest#3 and doctest#7 still import core/mem while using alloc/dealloc, but current stdlib keeps those APIs in core/mem/allocator, so they fail with resolve.identifier.undefined before exercising kp prefix/search behavior.

## 影響

The KP focused suite remains red for fixture-contract reasons and can obscure real streamio/stdout writer regressions. Weakening Resource IR owner checks or resolver behavior would hide the root cause; the tests need to state the current owner cleanup and allocator import contracts explicitly.

## 修正方針

Update the KP doctest fixtures to close StreamScanner owners on every success path and stop constructing raw buffers in ordinary doctest source. Use safe `Vec<i32>` APIs for prefix and search examples, and add a `unique_sorted_vec_i32` wrapper so callers can use the raw-pointer unique implementation without receiving a raw address. Keep raw-memory and Resource IR checks unchanged.

## 検証

- `tests/stdlib/kp.n.md::doctest#1` now closes the `StreamScanner` owner.
- `tests/stdlib/kp.n.md::doctest#3` uses `Vec<i32>::filled` / `get` / `replace` / `free` instead of raw `alloc` / `fill_i32` / `load_i32` / `store_i32` / `dealloc`.
- `stdlib/kp/kpsearch.nepl` now exposes `UniqueSortedVecI32` and `unique_sorted_vec_i32`, an owner-consuming `Vec<i32>` wrapper around the raw-pointer `unique_sorted_i32` implementation.
- `tests/stdlib/kp.n.md::doctest#7` uses `count_equal_range_vec_i32` and `unique_sorted_vec_i32` instead of constructing raw buffers in ordinary source.
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 3 --assert-io --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i tests/stdlib/kp.n.md -n 7 --assert-io --dist web/dist`: passed
- `node nodesrc/run_doctest.js -i stdlib/kp/kpsearch.nepl -n 4 --assert-io --dist web/dist`: passed
- `node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kp-fixture-contract.json -j 1 --dist web/dist --assert-io`: total=7, passed=7
