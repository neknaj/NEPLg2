---
id: ISS-20260427T000313062Z-STACK-RETAINS-UNSAFE-UNWRAPS-IN-OWNE-62A953CC
title: "Stack retains unsafe unwraps in owned buffer cleanup"
area: stdlib
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-27
updated: 2026-04-27
target: "stdlib/alloc/collections/stack.nepl, tests/stdlib/stack_collections.n.md"
---

# ISS-20260427T000313062Z-STACK-RETAINS-UNSAFE-UNWRAPS-IN-OWNE-62A953CC: Stack retains unsafe unwraps in owned buffer cleanup

## 概要

Stack.free uses uwok on dealloc_ptr for owned data/header storage, and related allocation cleanup still relies on checked cleanup paths.

## 対象

- `stdlib/alloc/collections/stack.nepl, tests/stdlib/stack_collections.n.md`

## 根拠

- 未記入

## 問題

Stack.free uses uwok on dealloc_ptr for owned data/header storage, and related allocation cleanup still relies on checked cleanup paths.

## 影響

Parser/evaluator stacks for self-host can trap during cleanup and remain inconsistent with the safer Queue/Deque owner-invariant pattern.

## 修正方針

Replace owned data/header cleanup with dealloc_raw, audit allocation-failure cleanup, add free/grow regressions, and guard implementation code against unsafe unwrap helpers.

## 検証

Run Stack doctests, focused collection tests, source guard, stdlib suite, and nodesrc/issues.js check.
