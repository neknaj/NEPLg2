---
id: ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB
title: "ByteBuilder and StringBuilder Result owner paths leak under Resource IR"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "stdlib/alloc/io.nepl, stdlib/alloc/string.nepl, tests/stdlib/byte_builder.n.md"
---

# ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB: ByteBuilder and StringBuilder Result owner paths leak under Resource IR

## 概要

While fixing ByteBuf owner transfer, tests/stdlib/byte_builder.n.md reports byte_builder_with_capacity Result::Ok(ByteBuilder) owner obligation violations, and broader text/fs runs still surface sb_build_result/string_builder_with_capacity_result owner leaks. The builder APIs still mix empty non-owning state and owning buffer state in a way Resource IR cannot prove at Result boundaries.

## 対象

- `stdlib/alloc/io.nepl, stdlib/alloc/string.nepl, tests/stdlib/byte_builder.n.md`

## 根拠

- `ISS-20260429T125126519Z` の修正後に `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/byte-builder-after-alloc-io-bytebuf.json -j 1 --dist web/dist` を実行すると、`byte_builder_with_capacity` の `Result::Ok(ByteBuilder)` return path で Resource IR owner obligation violation が残る。
- `ByteBuilder` は空 builder を `ptr=mem_ptr_wrap 0, len=0, cap=0` として表し、非空 builder は `ptr` に所有領域を入れる。同じ `ptr` field が null sentinel と owner を兼ねるため、`Result<ByteBuilder, StdErrorKind>` の `Ok` arm で所有領域が戻り値に一度だけ移ることを Resource IR が証明しにくい。
- `tests/stdlib/fs.n.md` と `tests/stdlib/text_utf8.n.md` の広い実行でも `sb_build_result` / `string_builder_with_capacity_result` の owner leak が出る。`StringBuilder` も `ByteBuilder` と同型の「空 sentinel と owner を同じ field に混ぜる」設計を持つため、個別の call site 修正ではなく builder contract の再設計が必要。

## 問題

While fixing ByteBuf owner transfer, tests/stdlib/byte_builder.n.md reports byte_builder_with_capacity Result::Ok(ByteBuilder) owner obligation violations, and broader text/fs runs still surface sb_build_result/string_builder_with_capacity_result owner leaks. The builder APIs still mix empty non-owning state and owning buffer state in a way Resource IR cannot prove at Result boundaries.

## 影響

ByteBuilder and StringBuilder users in self-host binary/text emitters can be hidden behind D3100 owner obligation failures even when caller code is correct. This blocks broader stdlib/self-host validation and encourages unsafe raw pointer workarounds.

## 修正方針

Review builder ownership contracts as a dedicated issue. Model empty builder state and owning buffer state with a ResourceIR-clean type boundary, then update ByteBuilder/StringBuilder constructors, reserve, finish, free, and regression tests without weakening Resource IR.

## 検証

- 修正後に `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/byte-builder-after-builder-redesign.json -j 1 --dist web/dist` が ByteBuilder owner violation を出さないこと。
- 修正後に `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/string-builder-after-builder-redesign.json -j 1 --dist web/dist` が StringBuilder owner violation を出さないこと。
- 修正後に `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text-utf8-after-builder-redesign.json -j 1 --dist web/dist` を実行し、少なくとも builder owner leak が top issue から消えていること。
- source policy regression を追加し、`ByteBuilder` / `StringBuilder` が空 sentinel と owning pointer を同じ裸 `MemPtr` field に混在させる設計へ戻らないことを固定する。
