---
id: ISS-20260427T000312311Z-RINGBUFFER-RETAINS-UNSAFE-UNWRAPS-IN-8E14D45A
title: "RingBuffer retains unsafe unwraps in circular-buffer internals"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/ringbuffer.nepl, tests/stdlib/ringbuffer_collections.n.md"
---

# ISS-20260427T000312311Z-RINGBUFFER-RETAINS-UNSAFE-UNWRAPS-IN-8E14D45A: RingBuffer retains unsafe unwraps in circular-buffer internals

## 概要

RingBuffer still uses uwok for header stores, grow cleanup, and free despite owning the header and data buffer.

## 対象

- `stdlib/alloc/collections/ringbuffer.nepl, tests/stdlib/ringbuffer_collections.n.md`

## 根拠

- 未記入

## 問題

RingBuffer still uses uwok for header stores, grow cleanup, and free despite owning the header and data buffer.

## 影響

The lower-level circular buffer used by queues can still regress to unsafe helper traps even after Queue/Deque were fixed.

## 修正方針

Move header writes to raw owner-invariant helpers, use dealloc_raw for owned buffers, add grow/pop/clear/free regressions, and add a source guard.

## 検証

Run RingBuffer doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
