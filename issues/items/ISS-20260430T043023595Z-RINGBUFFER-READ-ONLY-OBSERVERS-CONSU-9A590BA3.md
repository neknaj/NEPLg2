---
id: ISS-20260430T043023595Z-RINGBUFFER-READ-ONLY-OBSERVERS-CONSU-9A590BA3
title: "RingBuffer read-only observers consume owners instead of borrowing"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "stdlib/alloc/collections/ringbuffer.nepl, stdlib/tests/ringbuffer.n.md, tests/stdlib/ringbuffer_collections.n.md, nodesrc/test_stdlib_ringbuffer_borrowed_observers.js"
---

# ISS-20260430T043023595Z-RINGBUFFER-READ-ONLY-OBSERVERS-CONSU-9A590BA3: RingBuffer read-only observers consume owners instead of borrowing

## 概要

RingBuffer.len, cap, is_empty, and peek are read-only observers but take RingBuffer by value and free the owner. The borrowed variants remain as duplicate *_ref surfaces, so observation and ownership destruction are mixed in the public API.

## 対象

- `stdlib/alloc/collections/ringbuffer.nepl, stdlib/tests/ringbuffer.n.md, tests/stdlib/ringbuffer_collections.n.md, nodesrc/test_stdlib_ringbuffer_borrowed_observers.js`

## 根拠

- `stdlib/alloc/collections/ringbuffer.nepl` documented `len` / `cap` / `is_empty` / `peek` as value-consuming observers that close the buffer owner.
- The same module also kept borrowed `len_ref` / `cap_ref` / `is_empty_ref` / `peek_ref` variants, leaving duplicate public observer surfaces.
- Other Stage 6 collection work has moved read-only observation to borrowed receivers so Resource IR can distinguish observation from owner transfer or terminal cleanup.
- Existing tests used `len rb` / `peek rb` as terminal observations, which made accidental buffer destruction look like normal usage.

## 問題

RingBuffer.len, cap, is_empty, and peek are read-only observers but take RingBuffer by value and free the owner. The borrowed variants remain as duplicate *_ref surfaces, so observation and ownership destruction are mixed in the public API.

## 影響

Callers can accidentally destroy the buffer when only observing length, capacity, emptiness, or the front item. This is inconsistent with the Stage 6 owner model and with other collection observers that now borrow owners.

## 修正方針

Change len/cap/is_empty/peek to borrow &RingBuffer, remove duplicate *_ref observers, update tests/docs to explicitly free observed owners, and add source-policy regression coverage.

## 検証

Run RingBuffer doctests, stdlib/tests/ringbuffer.n.md, tests/stdlib/ringbuffer_collections.n.md, source-policy regressions, and issue checks.

確認済み:

- `node nodesrc/test_stdlib_ringbuffer_borrowed_observers.js`: passed
- `node nodesrc/test_stdlib_ringbuffer_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/ringbuffer.nepl --no-tree -o tmp/ringbuffer-borrowed-observers-doctests.json -j 1` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i stdlib/tests/ringbuffer.n.md --no-tree -o tmp/ringbuffer-borrowed-observers-stdlib-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i tests/stdlib/ringbuffer_collections.n.md --no-tree -o tmp/ringbuffer-borrowed-observers-collections-tests.json -j 1` (`total=2`, `passed=2`, `failed=0`)

## 修正内容

- `RingBuffer.len` / `cap` / `is_empty` / `peek` を `&RingBuffer` receiver に変更した。
- 重複していた `len_ref` / `cap_ref` / `is_empty_ref` / `peek_ref` を削除した。
- `pop` は先頭値だけを返して buffer を閉じる terminal API として残し、owner-preserving には既存 `pop_front` を使う設計を明記した。
- `ringbuffer.nepl` に borrowed observer 後に `free` する doctest を追加した。
- stdlib / collection tests を、borrowed observer 後に同じ owner を `free` する形へ更新した。
- `nodesrc/test_stdlib_ringbuffer_borrowed_observers.js` を source policy に登録し、by-value observer と `*_ref` surface の再発を検出するようにした。
