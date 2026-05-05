---
id: ISS-20260430T135134835Z-STR-COPY-VIEW-CONTRACT-CONFLICTS-WIT-0998304C
title: "str Copy view contract conflicts with Resource IR owner obligations"
area: CORE
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-30
updated: 2026-05-05
target: "stdlib/alloc/string.nepl, nepl-core/src/resource"
---

# ISS-20260430T135134835Z-STR-COPY-VIEW-CONTRACT-CONFLICTS-WIT-0998304C: str Copy view contract conflicts with Resource IR owner obligations

## 概要

stdlib/core/traits/copy.nepl documents str as a Copy non-owning view, but Resource IR still uses str places as possible free-obligation carriers for raw-memory-backed strings. Observer APIs such as str_char_at_result take str by value, so reusing one local across fallible observer calls can produce resource.owner.reserved; simply ignoring Copy sources in the owner checker would weaken dynamic string and MemPtr/RegionToken ownership safety.

## 対象

- `stdlib/alloc/string.nepl, nepl-core/src/resource`

## 根拠

- `stdlib/core/traits/copy.nepl` states that `str` is a `Clone` / `Copy` non-owning length-prefixed string view.
- Current Resource IR owner checks still need to preserve storage/free obligations for raw-memory-backed string paths such as `str_from_addr_unchecked`, `concat_result`, and `string_from_mem_unchecked_result`.
- `tests/stdlib/string_char.n.md` exposed the conflict: repeated by-value observer calls over one `str` local hit `resource.owner.reserved`, but blindly skipping Copy sources would also weaken existing unresolved fallible owner checks for dynamic storage aliases.

## 問題

stdlib/core/traits/copy.nepl documents str as a Copy non-owning view, but Resource IR still uses str places as possible free-obligation carriers for raw-memory-backed strings. Observer APIs such as str_char_at_result take str by value, so reusing one local across fallible observer calls can produce resource.owner.reserved; simply ignoring Copy sources in the owner checker would weaken dynamic string and MemPtr/RegionToken ownership safety.

## 影響

The language contract, stdlib API shape, and Resource IR owner model disagree. Tests and examples can be forced to pass with fresh literals, but self-host code will keep encountering unclear ownership requirements until str view, owned string storage, and Resource IR storage owners are separated.

## 修正方針

Follow doc/neplg2/static_check_complexity_reduction_plan.md Stage 3/6: keep str as a non-owning Copy view, represent allocator-backed string storage with an explicit non-Copy owner token/storage in Resource IR or stdlib, and migrate observer APIs to borrow/view-only contracts. Do not relax reserved-owner diagnostics for Copy MemPtr or dynamic-storage aliases until the owner token split is represented.

## 検証

Add focused compiler and stdlib regressions: repeated reads of a literal/non-owning str local must pass, owned dynamic string storage must not leak or double-free, and unresolved fallible dealloc/realloc results must still reserve the storage owner until matched.

## 2026-05-05 Agent 1 修正

`Resource IR` の owner summary leaf 生成から `TypeKind::Str` を外し、`str` parameter / local を型だけで free obligation owner として扱わないようにした。これにより、`str` を返す `Result` や fallible observer の未 match 結果が、入力 `str` view を `Reserved` にして後続の read を拒否する問題を解消した。

今回の修正は `Copy` 全般を owner checker から除外するものではない。`MemPtr` / `RegionToken` は引き続き raw owner provenance の leaf として扱うため、raw storage owner の検査は弱めていない。また、`str_from_addr_unchecked` で raw region owner が returned `str` へ移る回帰を追加し、動的文字列領域の owner transfer が残ることを確認した。

追加した回帰:

- `nepl-core/tests/resource_ir.rs`: `str` view を `Result` payload 経由で複数回返しても、Copy view が予約されないことを確認。
- `nepl-core/tests/resource_ir.rs`: raw region owner が `str_from_addr_unchecked` 経由で returned `str` へ移ることを確認。
- `tests/stdlib/string_char.n.md`: 同じ `str` local を未 match の `str_char_at_result` 結果が残った状態で複数回 observer に渡しても compile/run できることを確認。

残る大規模な `MemPtr = non-owning pointer`、`OwnedRegion/Storage = free obligation owner`、`InitializedCell/Resource IR = initialized/moved/drop state` の完全分離は、`ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` と `ISS-20260425T000000Z-RV-CORE-009-58589A3F` の親 issue で継続する。この issue では `str` の Copy view contract と owner reservation の衝突を解消済みとする。
