---
id: ISS-20260426T213058421Z-LLVM-TARGET-TYPECHECKS-STDLIB-COLLEC-DD7EF927
title: "LLVM target typechecks stdlib collection modules with signature drift"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/typecheck.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/binary_heap.nepl, stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/deque.nepl, stdlib/alloc/collections/list.nepl"
---

# ISS-20260426T213058421Z-LLVM-TARGET-TYPECHECKS-STDLIB-COLLEC-DD7EF927: LLVM target typechecks stdlib collection modules with signature drift

## 概要

GitHub Actions run 24967172989 llvm-dual-stdlib reports D3003 return type mismatch, D3005 ambiguous overload, and D3016 stack leftovers across binary_heap, btreemap/btreeset, deque, list, queue, ringbuffer, and related collection doctests only in LLVM lowering mode.

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/binary_heap.nepl, stdlib/alloc/collections/btreemap.nepl, stdlib/alloc/collections/deque.nepl, stdlib/alloc/collections/list.nepl`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-stdlib reports D3003 return type mismatch, D3005 ambiguous overload, and D3016 stack leftovers across binary_heap, btreemap/btreeset, deque, list, queue, ringbuffer, and related collection doctests only in LLVM lowering mode.

## 影響

WASM stdlib doctests can pass while the LLVM target sees a different typed surface, so backend parity for self-host collections is not established.

## 修正方針

Identify whether LLVM target cfg exposes different overload sets or whether lowering re-typechecks already checked modules with altered expectations; make target-specific imports and generic constraints deterministic across WASM and LLVM.

## 検証

Run llvm-dual-stdlib focused collection doctests and confirm D3003/D3005/D3016 collection signature drift is gone.
