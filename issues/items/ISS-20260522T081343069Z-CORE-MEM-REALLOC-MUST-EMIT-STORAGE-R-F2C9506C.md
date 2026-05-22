---
id: ISS-20260522T081343069Z-CORE-MEM-REALLOC-MUST-EMIT-STORAGE-R-F2C9506C
title: "core/mem realloc must emit storage relocate proof for collection-managed slots"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-22
updated: 2026-05-22
target: "stdlib/core/mem/pointer/region.nepl, nepl-core/src/resource/**"
---

# ISS-20260522T081343069Z-CORE-MEM-REALLOC-MUST-EMIT-STORAGE-R-F2C9506C: core/mem realloc must emit storage relocate proof for collection-managed slots

## 概要

Vec grow cannot soundly emit collection_slot_storage_relocate after realloc_region_bytes_keep because the old RegionToken owner is moved into realloc before the Ok branch receives the new RegionToken. The source-level proof therefore cannot borrow old and new storage tokens simultaneously at the Vec wrapper layer.

## 対象

- `stdlib/core/mem/pointer/region.nepl, nepl-core/src/resource/**`

## 根拠

- `collection_slot_storage_relocate` は old/new `&RegionToken<T>` を要求し、by-value owner token anchor は拒否される。
- `Vec` grow wrapper は `realloc_region_bytes_keep<T> region new_bytes` へ old `RegionToken<T>` owner を渡した時点で old token を move 済みにする。`Result::Ok grown_region` 分岐では new token だけが残るため、source-level に old/new token refs を同時に渡せない。
- `core/mem/pointer/region.nepl` の `realloc_region_bytes_keep<T>` だけが、raw realloc success、old owner token consumption、new owner token construction の境界を同時に扱える。
- public `realloc_region_bytes_keep<T>` に lifecycle marker authority を直接露出すると、public memory API が collection slot state を任意に rekey できる surface になる。marker emission は private memory boundary に閉じる必要がある。

## 問題

Vec grow cannot soundly emit collection_slot_storage_relocate after realloc_region_bytes_keep because the old RegionToken owner is moved into realloc before the Ok branch receives the new RegionToken. The source-level proof therefore cannot borrow old and new storage tokens simultaneously at the Vec wrapper layer.

## 影響

Non-Copy Vec push/grow cannot preserve initialized slot state across reallocation without either reintroducing raw realloc in Vec, adding module-specific allowlists, or weakening collection slot relocation proof.

## 修正方針

Design a private relocation-aware core/mem realloc boundary that owns the raw realloc success evidence and emits collection_slot_storage_relocate while old and new RegionToken anchors are both available inside the trusted memory boundary. Public APIs must remain owner-preserving and must not expose lifecycle marker authority.

## 検証

Add Resource IR regressions showing collection-managed non-Copy slot state rekeys from old storage to new storage through source-level core/mem realloc, and that Vec wrapper cannot bypass this with raw pointers or public marker helpers.

## 2026-05-22 Agent 1 調査メモ

`Vec` 側で `collection_slot_storage_relocate` を発行する設計は不適切である。`realloc_region_bytes_keep<T>` は owner-preserving API として正しいが、caller から見ると old owner は call に消費され、success branch では new owner だけが返る。したがって `Vec` wrapper で old/new storage anchor を同時に借用することはできない。

根本設計は `core/mem` 側の private helper である。private helper は raw realloc success proof と new `RegionToken<T>` construction を同一境界に持てるため、そこで `collection_slot_storage_relocate` を発行し、public API は owner-preserving `Result<RegionToken<T>, RegionReallocError<T>>` contract を維持する。

避けるべき設計:

- `Vec` 側で `allocator::realloc_raw` を直接呼ぶ。
- old/new raw address の `i32` を marker に渡す。
- by-value token を relocate marker に許す。
- public helper へ lifecycle marker authority を露出する。
- `Vec` module allowlist で grow だけ特別扱いする。
