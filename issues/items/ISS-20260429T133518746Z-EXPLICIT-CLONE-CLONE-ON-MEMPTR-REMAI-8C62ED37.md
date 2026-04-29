---
id: ISS-20260429T133518746Z-EXPLICIT-CLONE-CLONE-ON-MEMPTR-REMAI-8C62ED37
title: "Explicit Clone::clone on MemPtr remains unresolved after monomorphize"
area: compiler
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core, stdlib/core/traits/copy.nepl, stdlib/core/mem.nepl"
---

# ISS-20260429T133518746Z-EXPLICIT-CLONE-CLONE-ON-MEMPTR-REMAI-8C62ED37: Explicit Clone::clone on MemPtr remains unresolved after monomorphize

## 概要

While refactoring std/streamio StreamWriter, using let w_init <MemPtr<u8>> Clone::clone &w compiled through type checking but failed backend codegen with ackend.codegen.trait_call_unresolved: Clone<>::clone [self=MemPtr_T_u8]. MemPtr<.T> has Clone/Copy impls in core/traits/copy, so explicit clone should resolve or fail earlier with a precise diagnostic.

## 対象

- `nepl-core, stdlib/core/traits/copy.nepl, stdlib/core/mem.nepl`

## 根拠

- 未記入

## 問題

While refactoring std/streamio StreamWriter, using let w_init <MemPtr<u8>> Clone::clone &w compiled through type checking but failed backend codegen with ackend.codegen.trait_call_unresolved: Clone<>::clone [self=MemPtr_T_u8]. MemPtr<.T> has Clone/Copy impls in core/traits/copy, so explicit clone should resolve or fail earlier with a precise diagnostic.

## 影響

Stdlib code that tries to avoid moving a MemPtr owner by explicit cloning can hit a backend-only unresolved trait call. This encourages awkward raw-address workarounds and weakens confidence in Copy/Clone capability checking for self-host code.

## 修正方針

Trace trait resolution and monomorphization for explicit associated trait calls on generic type constructors such as MemPtr<.T>. Ensure Clone::clone for MemPtr<u8> resolves to the stdlib impl before backend, or produce an earlier typed diagnostic if the call form is unsupported.

## 検証

Add a focused .n.md or Rust integration test that imports core/traits/copy and calls Clone::clone on a MemPtr<u8>, then run the backend/codegen path that previously reported backend.codegen.trait_call_unresolved.
