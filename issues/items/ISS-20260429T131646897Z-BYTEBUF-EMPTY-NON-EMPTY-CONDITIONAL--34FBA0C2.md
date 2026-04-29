---
id: ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2
title: "ByteBuf empty/non-empty ownership invariant lacks explicit structural representation"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/io.nepl, stdlib/core/mem.nepl, doc/neplg2/static_check_complexity_reduction_plan.md"
---

# ISS-20260429T131646897Z-BYTEBUF-EMPTY-NON-EMPTY-CONDITIONAL--34FBA0C2: ByteBuf empty/non-empty ownership invariant lacks explicit structural representation

## 概要

After fixing raw address view owner moves, io_bytebuf_from_str_result exposes a deeper stdlib/API modeling gap: the empty branch returns a ByteBuf with no free obligation while the non-empty branch returns a ByteBuf whose ptr owns storage. Resource IR now propagates that conditional obligation as `MaybeFreed` across function summaries, but ByteBuf itself still encodes the invariant as a loose pair of `ptr` and `len` instead of a structural representation the compiler can fully validate.

## 対象

- `stdlib/alloc/io.nepl, stdlib/core/mem.nepl, doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view` で、元の `ConstructInput out Moved` は解消した一方で、空/非空 ByteBuf の owner が `MaybeFreed` として合流することが分かった。
- 空 branch は `io_bytebuf_empty` により free obligation を持たず、非空 branch は `alloc_ptr<u8>` の owner を `ByteBuf.ptr` に移すため、現在の `OwnerState` merge では同じ struct field に `NoFreeObligation | Live` が合流する。
- 今回の修正で `MaybeFreed` の return boundary と function summary 伝播は追加した。残る問題は、ByteBuf の「空なら owner なし、非空なら owner あり」という値不変条件が stdlib の規約に留まり、型/IR の構造として表現されていない点である。
- これは [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の stdlib memory API 責務分離に属する。

## 問題

ByteBuf の現在の表現では、`len == 0` と owner 不在、`len > 0` と owner 存在の対応が型や Resource IR op ではなく関数実装の discipline になっている。これにより、owner check は保守的な `MaybeFreed` 伝播に頼る必要があり、ByteBuf constructor/free API の正当性を構造的に検証しづらい。

## 影響

ByteBuf-producing APIs such as io_bytebuf_from_str_result, stdio_finish_read_buffer, fs_read_fd_bytes, and stream input helpers can be made leak-safe by `MaybeFreed` propagation, but the API boundary remains harder to reason about than necessary. Self-host I/O and buffer APIs should not inherit this loose invariant as a long-term design.

## 修正方針

Decide the representation boundary instead of adding local exceptions. Preferred direction is to represent ByteBuf as an enum with an owning non-empty payload and an empty no-owner variant, or introduce an explicit `OwnedBytes`/`Storage` wrapper whose free obligation is structurally separate from non-owning `MemPtr`. Update `io_bytebuf_free` and all ByteBuf constructors to match the chosen model, then simplify any Resource IR special handling that becomes unnecessary.

## 検証

Add stdlib/API-level regressions that exercise empty and non-empty ByteBuf construction, caller-side free, and misuse cases after the representation change. Keep the Resource IR negative regression that proves an actually live conditional owner cannot be silently dropped.
