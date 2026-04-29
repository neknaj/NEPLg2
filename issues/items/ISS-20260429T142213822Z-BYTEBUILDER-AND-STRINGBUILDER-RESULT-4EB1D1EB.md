---
id: ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB
title: "ByteBuilder and StringBuilder Result owner paths leak under Resource IR"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
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

## 2026-04-30 対応結果

- `ByteBuilder.ptr` と `StringBuilder.data` を `Option<MemPtr<u8>>` に変更し、空 storage は `None`、所有 storage は `Some(ptr)` として表すようにした。
- `byte_builder_from_owned_ptr` / `string_builder_from_owned_ptr` を追加し、allocation / reallocation 後の owner construction を一箇所へ集約した。
- append 成功後に payload pointer を取り出して新しい builder へ包み直す形をやめ、`byte_builder_with_len` / `string_builder_with_len` で `Option<MemPtr<u8>>` field 全体を移して length だけ更新する形にした。
- append 中の書き込み pointer は `get_ref` で借用し、所有権移動と buffer 書き込みを分離した。
- `finish` / `free` / `reserve` は `Option` を `match` し、`None` と `Some` の invariant が静的検査に現れる形へ更新した。
- `nodesrc/test_stdlib_builder_owner_boundary.js` を追加し、null owning pointer 表現、direct `Result::Ok ByteBuilder/StringBuilder`、append 中の owner payload unwrap/rewrap へ戻らないことを CI で固定した。

## 2026-04-30 検証

- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `git diff --check`: passed
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/byte-builder-owner-boundary-after2.json -j 1 --dist web/dist`: total=3 failed=3。失敗は `std/test` の `check_eq_i32` `TestAssertion` owner obligation で、`ByteBuilder` owner leak は top issue から消えた。
- `node nodesrc/tests.js -i stdlib/alloc/string.nepl --no-tree -o tmp/string-builder-owner-boundary-after2.json -j 1 --dist web/dist`: total=8 passed=5 failed=3。失敗は `std/test` / fixture mismatch 側で、`sb_append_result` / `string_builder_with_capacity_result` の builder owner leak は top issue から消えた。
- `node nodesrc/tests.js -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/text-utf8-builder-owner-boundary-after2.json -j 1 --dist web/dist`: total=9 failed=9。失敗は `std/test` の `check_*` helper owner obligation で、`byte_builder_push_u8` / `sb_append_result` の builder owner leak は top issue から消えた。

残った `std/test` 互換 `check_*` helper の `TestAssertion` owner obligation は、builder contract とは別根のテスト基盤問題として次 issue で扱う。
