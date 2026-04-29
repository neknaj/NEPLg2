---
id: ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4
title: "Resource IR lacks storage origin for unmanaged raw addresses"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/resource/model.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/lower.rs, stdlib/core/mem.nepl"
---

# ISS-20260429T012328323Z-RESOURCE-IR-LACKS-STORAGE-ORIGIN-FOR-549F82A4: Resource IR lacks storage origin for unmanaged raw addresses

## 概要

Resource owner gate can distinguish Live/Moved/Freed owner obligations, but raw i32 constants and legacy unmanaged addresses have no storage origin. Treating every dealloc without a free obligation as D3100 breaks migration fixtures, while leaving NoFreeObligation shadow-only would hide real owned-storage bugs.

## 対象

- `nepl-core/src/resource/model.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/lower.rs, stdlib/core/mem.nepl`

## 根拠

- `ISS-20260429T004144320Z-RESOURCE-OWNER-GATE-TREATS-RAW-POINT-216A5E25` の修正中、owner gate を D3100 に接続すると `tests/compiler/move_effect.n.md::doctest#35` の positive case が `Dealloc ... found NoFreeObligation` で失敗した。
- 該当 case は `let p <i32> 16` という固定 raw address に `LocalToken` を store/load した後で `dealloc_raw p` する移行用 fixture であり、`alloc_raw` が発行した free obligation owner ではない。
- 一方で、owned storage に対する NoFreeObligation を常に shadow-only にすると、Resource IR owner gate が本来検出すべき「owner obligation を失った dealloc/realloc」を見逃す。
- `doc/neplg2/static_check_complexity_reduction_plan.md` は `MemPtr = non-owning pointer` と `Storage/OwnedRegion = free obligation owner` の分離を完了条件にしているため、raw address が owned storage 由来か unmanaged/external 由来かを Resource IR が保持する必要がある。

## 問題

Resource owner gate can distinguish Live/Moved/Freed owner obligations, but raw i32 constants and legacy unmanaged addresses have no storage origin. Treating every dealloc without a free obligation as D3100 breaks migration fixtures, while leaving NoFreeObligation shadow-only would hide real owned-storage bugs.

## 影響

Stage 4 owner gate cannot be fully authoritative until Resource IR separates compiler-owned storage from external/unmanaged raw addresses. Stage 5/6 core/mem migration also needs this distinction to close public raw address escape without rejecting internal or legacy unmanaged storage cases incorrectly.

## 修正方針

Add a storage origin/provenance classification to Resource IR, for example OwnedStorage versus ExternalUnmanagedStorage/InternalRawBoundary. OwnerState checks should report NoFreeObligation as D3100 only for places that are expected to carry an owned free obligation; unmanaged or internal raw storage must be controlled by explicit capability/effect boundary instead of silent shadow behavior.

## 検証

Add Resource IR owner tests for owned alloc double-free/no-obligation, unmanaged fixed-address dealloc, and internal raw boundary behavior. Then enable NoFreeObligation owner diagnostics for owned storage while keeping tests/compiler/move_effect.n.md D3025 and D3100 expectations stable.
