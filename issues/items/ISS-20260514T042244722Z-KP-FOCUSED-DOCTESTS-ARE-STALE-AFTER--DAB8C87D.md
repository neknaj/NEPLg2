---
id: ISS-20260514T042244722Z-KP-FOCUSED-DOCTESTS-ARE-STALE-AFTER--DAB8C87D
title: "kp focused doctests are stale after streamio owner and allocator module split"
area: TEST
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: tests/stdlib/kp.n.md
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

## 問題

tests/stdlib/kp.n.md currently fails independently of the StreamWriter close facade fix. Doctest#1 opens a StreamScanner owner and borrows it through read &sc but never consumes it with close, so Resource IR correctly reports resource.owner.leak. Doctest#3 and doctest#7 still import core/mem while using alloc/dealloc, but current stdlib keeps those APIs in core/mem/allocator, so they fail with resolve.identifier.undefined before exercising kp prefix/search behavior.

## 影響

The KP focused suite remains red for fixture-contract reasons and can obscure real streamio/stdout writer regressions. Weakening Resource IR owner checks or resolver behavior would hide the root cause; the tests need to state the current owner cleanup and allocator import contracts explicitly.

## 修正方針

Update the KP doctest fixtures to close StreamScanner owners on every success path and import the allocator APIs from core/mem/allocator where alloc/dealloc are used. Keep raw-memory and Resource IR checks unchanged.

## 検証

Run node nodesrc/tests.js -i tests/stdlib/kp.n.md --no-tree -o tmp/agent1-kp-fixture-contract.json -j 1 --dist web/dist --assert-io and require total=7 passed=7.
