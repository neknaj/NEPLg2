---
id: ISS-20260513T133223124Z-MEMORY-SAFETY-CONSTRUCTOR-BOUNDARY-R-A6590141
title: "memory_safety constructor boundary regressions no longer prove typecheck rejection"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/typecheck/constructor_apply.rs; tests/stdlib/memory_safety.n.md"
---

# ISS-20260513T133223124Z-MEMORY-SAFETY-CONSTRUCTOR-BOUNDARY-R-A6590141: memory_safety constructor boundary regressions no longer prove typecheck rejection

## 概要

tests/stdlib/memory_safety.n.md doctest#16 currently compiles despite expecting type.raw_pointer.constructor_restricted, and doctest#22 fails with resource.owner.no_free_obligation instead of type.owner_token.constructor_restricted. The fixtures are intended to prove constructor restriction at typecheck, but current behavior either leaves the constructor in an unused function or lets owner checking become the first failing gate.

## 対象

- `nepl-core/src/typecheck/constructor_apply.rs; tests/stdlib/memory_safety.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-initialized-raw-access-memory-safety.json -j 1 --dist web/dist` で total=29, passed=27, failed=2。
- doctest#16 `MemPtr の直 constructor は memory boundary 外で使えない` は `type.raw_pointer.constructor_restricted` を期待しているが、compile_fail にならず compiled successfully だった。
- doctest#22 `RegionToken の直 constructor は memory boundary 外で使えない` は `type.owner_token.constructor_restricted` を期待しているが、実際には `resource.owner.no_free_obligation` が最初に出ている。
- direct constructor restriction は Resource IR owner gate より前の typecheck gate で拒否されるべき safety boundary なので、fixture 更新だけでなく compiler 側のgate到達条件も監査する必要がある。

## 問題

tests/stdlib/memory_safety.n.md doctest#16 currently compiles despite expecting type.raw_pointer.constructor_restricted, and doctest#22 fails with resource.owner.no_free_obligation instead of type.owner_token.constructor_restricted. The fixtures are intended to prove constructor restriction at typecheck, but current behavior either leaves the constructor in an unused function or lets owner checking become the first failing gate.

## 影響

MemPtr and RegionToken direct constructor capability can regress without a stable typecheck-level signal, weakening the proof that compiler-issued pointer/owner tokens cannot be forged from user source before Resource IR owner checking.

## 修正方針

Audit raw pointer and owner token constructor restriction against reachable and unused functions, make the typecheck gate authoritative for direct constructors, and update memory_safety fixtures so they exercise reachable constructor calls with stable diagnostic codes.

## 検証

Run cargo test -p nepl-core constructor_restricted, trunk build, node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree --dist web/dist -j 1, and node nodesrc/issues.js check --dir issues.
