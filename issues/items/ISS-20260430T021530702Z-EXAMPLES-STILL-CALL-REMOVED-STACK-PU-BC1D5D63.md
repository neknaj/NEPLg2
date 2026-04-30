---
id: ISS-20260430T021530702Z-EXAMPLES-STILL-CALL-REMOVED-STACK-PU-BC1D5D63
title: "examples still call removed Stack push_ref and pop_ref APIs"
area: examples
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "examples/bf.nepl, examples/rpn.nepl, examples/rpn_legacy.nepl"
---

# ISS-20260430T021530702Z-EXAMPLES-STILL-CALL-REMOVED-STACK-PU-BC1D5D63: examples still call removed Stack push_ref and pop_ref APIs

## 概要

Stack owner-safety redesign removed the borrow-mutating push_ref/pop_ref API, but examples still call those names and now fail to compile.

## 対象

- `examples/bf.nepl, examples/rpn.nepl, examples/rpn_legacy.nepl`

## 根拠

- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 4` で `examples/bf.nepl`, `examples/rpn.nepl`, `examples/rpn_legacy.nepl` が `resolve.identifier.undefined` (`stk::push_ref` / `stk::pop_ref`) により compile fail した。
- `stdlib/alloc/collections/stack.nepl` は `push_ref` / `pop_ref` を廃止し、owner を返す `push` と `pop_top` を正規 API としている。
- `ISS-20260429T155343006Z-COLLECTION-STORAGE-STATES-USE-NUMERI-E4B3A749` で Stack owner-safe redesign が入った後、examples 側の追従が漏れていた。

## 問題

Stack owner-safety redesign removed the borrow-mutating push_ref/pop_ref API, but examples still call those names and now fail to compile.

## 影響

The examples doctest suite fails for Brainfuck and RPN samples, so tutorial/example verification and deploy confidence are incomplete.

## 修正方針

Rewrite the examples to use current Stack owner flow: push returns a new Stack owner and pop_top returns StackPop containing the updated owner plus optional item.

## 検証

Run nodesrc/tests.js on examples/bf.nepl, examples/rpn.nepl, examples/rpn_legacy.nepl, and examples.

確認済み:

- `node nodesrc/tests.js -i examples/rpn_legacy.nepl --no-tree -o tmp/rpn-legacy-stack-owner-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i examples/rpn.nepl --no-tree -o tmp/rpn-stack-owner-tests.json -j 2` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i examples/bf.nepl --no-tree -o tmp/bf-stack-owner-tests.json -j 2` (`total=2`, `passed=2`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-stack-owner-tests.json -j 4` (`total=12`, `passed=12`, `failed=0`)
