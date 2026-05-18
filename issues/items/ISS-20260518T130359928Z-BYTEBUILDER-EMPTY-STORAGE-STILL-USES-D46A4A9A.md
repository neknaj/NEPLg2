---
id: ISS-20260518T130359928Z-BYTEBUILDER-EMPTY-STORAGE-STILL-USES-D46A4A9A
title: "ByteBuilder empty storage still uses zero-size RegionToken sentinel"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/io/bytebuilder/types.nepl, stdlib/alloc/io/bytebuilder/storage.nepl, stdlib/alloc/io/bytebuilder/append.nepl, stdlib/alloc/io/bytebuilder/build.nepl, nodesrc/source_policy/stdlib_builder_owner.js"
---

# ISS-20260518T130359928Z-BYTEBUILDER-EMPTY-STORAGE-STILL-USES-D46A4A9A: ByteBuilder empty storage still uses zero-size RegionToken sentinel

## 概要

ByteBuilder no longer exposes byte_builder_empty_region publicly, but its internal empty state is still encoded by region_new(mem_ptr_wrap 0, 0). Empty storage and owned storage therefore share the same RegionToken field, so the absence of a free obligation is a helper convention rather than an enum state.

2026-05-18 に修正済み。`ByteBuilderStorage::Empty | Owned(RegionToken<u8>)` を導入し、空 storage と free obligation owner を enum state として分離した。

関連計画: [NEPLg2 静的検査の複雑化解消計画 Stage 6](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 対象

- `stdlib/alloc/io/bytebuilder/types.nepl, stdlib/alloc/io/bytebuilder/storage.nepl, stdlib/alloc/io/bytebuilder/append.nepl, stdlib/alloc/io/bytebuilder/build.nepl, nodesrc/source_policy/stdlib_builder_owner.js`

## 根拠

- Stage 6 の memory model 方針では、`MemPtr` は non-owning pointer、free obligation owner は `RegionToken` / `Owned*` 側へ分離し、空状態は sentinel ではなく enum state と `match` の網羅性で扱う。
- `ByteBuf` は `ByteBufStorage::Empty | Owned(RegionToken<u8>)` へ移行済みだったが、`ByteBuilder` は内部で `region_new(mem_ptr_wrap 0, 0)` を使い続けていた。
- source policy は public sentinel helper の削除までは監視していたが、private zero-size sentinel の再導入を十分に拒否していなかった。

## 問題

ByteBuilder no longer exposes byte_builder_empty_region publicly, but its internal empty state is still encoded by region_new(mem_ptr_wrap 0, 0). Empty storage and owned storage therefore share the same RegionToken field, so the absence of a free obligation is a helper convention rather than an enum state.

## 影響

Stage 6 wants MemPtr as non-owning pointer and owner/free obligation state as a separate typed value. Keeping an empty RegionToken sentinel in ByteBuilder makes Resource IR and source policy rely on a special zero-size token convention and leaves a different design than ByteBuf and VecStorage.

## 修正方針

Introduce ByteBuilderStorage::Empty | Owned(RegionToken<u8>), store that enum in ByteBuilder, update reserve/append/finish/free to match storage state, and extend source policy so byte_builder_empty_region or region_new ptr 0 cannot return.

## 対応内容

- `ByteBuilderStorage::Empty | Owned(RegionToken<u8>)` を追加し、`ByteBuilder` の owner field を `region` から `storage` に変更した。
- `byte_builder_empty` は `ByteBuilderStorage::Empty` を返し、`byte_builder_from_owned_region` だけが `Owned(region)` を構築するようにした。
- `byte_builder_reserve` / `byte_builder_push_u8` / `byte_builder_push_bytes_ref` / `byte_builder_finish` / `byte_builder_free` は `storage` を `match` し、`Owned` branch だけが `RegionToken<u8>` を borrow / realloc / dealloc / finish する形にした。
- source policy を更新し、`ByteBuilderStorage` enum、`storage` field、structural empty constructor、owned constructor、storage match を要求し、`byte_builder_empty_region`、zero-size `region_new` sentinel、旧 `region` field access の再導入を拒否するようにした。
- memory safety doctest の sentinel helper 回帰名を現状に合わせ、存在しない helper として固定した。

## 検証

- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_io_bytebuf_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`: passed
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: passed
- `node nodesrc/test_stdlib_streamio_writer_boundary.js`: passed
- `node nodesrc/test_stdlib_streamio_no_unsafe_unwraps.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuilder.nepl -i stdlib/alloc/io/bytebuilder/types.nepl -i stdlib/alloc/io/bytebuilder/storage.nepl -i stdlib/alloc/io/bytebuilder/append.nepl -i stdlib/alloc/io/bytebuilder/build.nepl --no-tree -o tmp/agent1-bytebuilder-storage-state.json -j 1 --dist web/dist --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i stdlib/alloc/io/bytebuf.nepl -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-bytebuilder-storage-bytebuf.json -j 1 --dist web/dist --assert-io`: total=8, passed=8
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-bytebuilder-storage-memory-safety.json -j 1 --dist web/dist --assert-io`: total=60, passed=60
