---
id: ISS-20260426T213057843Z-LLVM-MONOMORPHIZE-LEAVES-GENERIC-HAS-8FDB0749
title: "LLVM monomorphize leaves generic Hasher trait calls unresolved"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/monomorphize.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/hashmap.nepl, tests/stdlib/traits_hash.n.md"
---

# ISS-20260426T213057843Z-LLVM-MONOMORPHIZE-LEAVES-GENERIC-HAS-8FDB0749: LLVM monomorphize leaves generic Hasher trait calls unresolved

## 概要

GitHub Actions run 24967172989 llvm-dual-tests and llvm-dual-stdlib panic in monomorphize.rs with unresolved trait call remained after monomorphize: Hasher<str>::hash32 for Option_T_i32 or Option_T_str self types.

## 対象

- `nepl-core/src/monomorphize.rs, nepl-core/src/codegen_llvm.rs, stdlib/alloc/collections/hashmap.nepl, tests/stdlib/traits_hash.n.md`

## 根拠

- 未記入

## 問題

GitHub Actions run 24967172989 llvm-dual-tests and llvm-dual-stdlib panic in monomorphize.rs with unresolved trait call remained after monomorphize: Hasher<str>::hash32 for Option_T_i32 or Option_T_str self types.

## 影響

HashMap/HashSet and self-host symbol-table style code cannot be trusted under LLVM, and the compiler worker panics instead of emitting a diagnostic.

## 修正方針

Fix trait impl resolution/substitution for generic Hasher calls in LLVM-all monomorphization, and replace the residual panic path with a structured diagnostic if an unresolved trait call remains.

## 検証

Run llvm-all focused tests for stdlib/alloc/collections/hashmap.nepl and tests/stdlib/traits_hash.n.md without monomorphize panics.
