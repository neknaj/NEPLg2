---
id: ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159
title: "ByteBuf and ByteBuilder store owned bytes as Option MemPtr instead of RegionToken owner"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/io/bytebuf.nepl, stdlib/alloc/io/bytebuilder, nepl-core/src/resource/owner_summary_raw_transfer.rs, nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260514T071955576Z-BYTEBUF-STORES-OWNED-BYTES-AS-OPTION-FA165159: ByteBuf and ByteBuilder store owned bytes as Option MemPtr instead of RegionToken owner

## 概要

ByteBuf and ByteBuilder still kept owned byte storage as Option<MemPtr<u8>>, so the Stage 6 MemPtr owner-field baseline had to treat ByteBuf.ptr and ByteBuilder.ptr as transitional exceptions. This preserved public raw pointer owner shapes even though RegionToken already represents the free obligation owner.

## 対象

- `stdlib/alloc/io/bytebuf.nepl`
- `stdlib/alloc/io/bytebuilder/**`
- `nepl-core/src/resource/owner_summary_raw_transfer.rs`
- `nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6。
- [stdlib collection/mem/string と静的検査の安全設計](../../doc/neplg2/stdlib_collection_mem_string_static_safety_design.md) の `MemPtr = non-owning pointer` 方針。

## 問題

ByteBuf and ByteBuilder kept owned byte storage as Option<MemPtr<u8>>, so the Stage 6 MemPtr owner-field baseline had to treat both fields as transitional exceptions. This preserved public raw pointer owner shapes even though RegionToken already represents the free obligation owner.

## 影響

The static-check complexity reduction cannot finish while ByteBuf exposes a MemPtr owner field. Callers and source policies must reason about raw pointer ownership instead of an explicit owner token, increasing the chance that non-owning MemPtr projections are mistaken for storage owners.

## 修正方針

Represent ByteBuf and ByteBuilder storage with RegionToken<u8> owner tokens and derive borrowed MemPtr views only through RegionToken references. Keep empty buffers as zero-length token sentinels that are never deallocated, update ByteBuf free/build/conversion paths, update ByteBuilder reserve/append/finish/free paths, and remove ByteBuf.ptr / ByteBuilder.ptr from the MemPtr owner-field migration baseline.

## 検証

Run focused ByteBuf/ByteBuilder/string tests plus the MemPtr owner-field and ByteBuf owner-boundary source policies.

## 解決内容

- `ByteBuf` は `region <RegionToken<u8>>` と `len` を持つ構造に変更し、`io_bytebuf_data_ptr_ref` で参照から non-owning `MemPtr<u8>` view を得る形にした。
- `ByteBuilder` は `region <RegionToken<u8>>` / `len` / `cap` を持つ構造に変更し、reserve / append / finish / free が owner token を field 全体として移す形にした。
- stdio / fs / streamio / text から ByteBuf / ByteBuilder の旧 `ptr` field を直接見る経路を削除した。
- compiler ResourceIR の function summary raw owner alias 追跡に raw view state を接続し、`region_ptr` 由来の non-owning projection に `mem_ptr_add` を重ねた値を owner alias と誤認しないようにした。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional baseline を `RegionToken.ptr` と `Vec.data` の 2 件に更新した。
