---
id: ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376
title: "Compiler memory owner token authority is name-shape based instead of canonical"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/source_capability/**, nepl-core/src/typecheck/**, nepl-core/src/resource_primitives/**"
---

# ISS-20260520T224333208Z-COMPILER-MEMORY-OWNER-TOKEN-AUTHORIT-7FCBF376: Compiler memory owner token authority is name-shape based instead of canonical

## 概要

RegionToken owner-token authority is currently granted from stdlib-root source evidence plus the RegionToken name and exact struct shape. This blocks ordinary user-source forgery, but any non-canonical stdlib-root file that defines the same shape can become CompilerMemoryType::OwnerToken, so the token is not yet truly compiler-issued by definition identity.

## 対象

- `nepl-core/src/source_capability/**, nepl-core/src/typecheck/**, nepl-core/src/resource_primitives/**`

## 根拠

- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) は、`RegionToken<T>` を過渡 owner token とし、最終的には compiler-issued owner token / `OwnedBuffer<T>` / initialized cell state へ移行する方針を示している。
- [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support の前提として forge-proof storage owner を要求している。
- subagent 監査では、ordinary user source からの `RegionToken` constructor / field / collection lifecycle boundary 偽造は現行 gate で拒否される一方、stdlib-root 内の同名同形 definition が compiler-memory authority を得る余地が残ると確認した。

## 問題

RegionToken owner-token authority is currently granted from stdlib-root source evidence plus the RegionToken name and exact struct shape. This blocks ordinary user-source forgery, but any non-canonical stdlib-root file that defines the same shape can become CompilerMemoryType::OwnerToken, so the token is not yet truly compiler-issued by definition identity.

## 影響

Non-Copy collection payload support and Resource IR owner proofs depend on forge-proof storage owners. If owner-token authority remains name-shape based, future stdlib refactors or generated compiler-owned sources can accidentally create owner authority outside the intended canonical definition boundary.

## 修正方針

Bind CompilerMemoryType marking to canonical compiler-memory definition identity instead of name/shape alone. Keep SourceCapabilityUseSite and CompilerMemoryType enum evidence, but require the expected canonical definition site for RegionToken and MemPtr authority.

## 検証

Add tests proving user fake RegionToken, stdlib-root non-canonical fake RegionToken, and fake/real mixed storage_relocate anchors are rejected while the canonical imported RegionToken remains constructor/field restricted and valid at compiler-owned boundaries.
