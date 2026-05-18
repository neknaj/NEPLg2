---
id: ISS-20260518T115751573Z-BYTEBUILDER-PUBLIC-RAW-BYTE-APPEND-D-D94BB3A0
title: "ByteBuilder public raw byte append drops source extent proof"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/io/bytebuilder/append.nepl, stdlib/alloc/string/builder/append.nepl, stdlib/alloc/string/builder_ext.nepl"
---

# ISS-20260518T115751573Z-BYTEBUILDER-PUBLIC-RAW-BYTE-APPEND-D-D94BB3A0: ByteBuilder public raw byte append drops source extent proof

## 概要

byte_builder_push_bytes_ref is public through alloc/io/bytebuilder and accepts an arbitrary MemPtr<u8> with an arbitrary length. StringBuilder callers derive that pair from str length or UTF-8 checked slice bounds, but the public ByteBuilder API lets ordinary callers bypass those source-derived extent proofs.

## 対象

- `stdlib/alloc/io/bytebuilder/append.nepl, stdlib/alloc/string/builder/append.nepl, stdlib/alloc/string/builder_ext.nepl`

## 根拠

- `alloc/io/bytebuilder` facade は `append.nepl` を re-export するため、`pub fn byte_builder_push_bytes_ref(ByteBuilder, &MemPtr<u8>, i32)` は通常 source から直接呼べた。
- `byte_builder_push_bytes_ref` は `mem_copy<u8> dst *src data_len` に到達するが、`src` と `data_len` が同じ source object から導出されたことを型や API signature で要求していなかった。
- 既存の `StringBuilder` は `str` の `len` と `string_data_ptr`、または UTF-8 境界確認済み slice からこの pair を作っていたが、その規律は caller convention であり public safe API としては迂回できた。

## 問題

byte_builder_push_bytes_ref is public through alloc/io/bytebuilder and accepts an arbitrary MemPtr<u8> with an arbitrary length. StringBuilder callers derive that pair from str length or UTF-8 checked slice bounds, but the public ByteBuilder API lets ordinary callers bypass those source-derived extent proofs.

## 影響

A raw memory backed builder API remains available as a public safe surface, so caller-supplied pointer/length pairs can reach mem_copy without a typed source object proving the readable extent. This weakens the Stage 6 MemPtr=non-owning pointer discipline and makes future Resource IR checks depend on API conventions instead of source/type proof artifacts.

## 修正方針

Make the raw MemPtr copy helper private inside bytebuilder append, add public typed helpers for full str and bounded str slice inputs that derive pointer/length from the source object, update StringBuilder to call those helpers, and add source policy plus compile-fail regressions rejecting the raw public append surface.

## 対応内容

- `byte_builder_push_bytes_ref` を private helper に変更し、public safe surface から raw `MemPtr<u8>` と任意 length の組を受け取る入口を削除した。
- `byte_builder_push_str(builder, s)` を追加し、`len s` と `string_data_ptr s` を同じ `str` から導出してから private copy helper へ渡すようにした。
- `byte_builder_push_str_slice(builder, s, start, end)` を追加し、`0 <= start <= end <= len(s)` を確認してから `mem_ptr_add (string_data_ptr s) start` と `end - start` を導出するようにした。
- `StringBuilder` の full append / slice append は typed ByteBuilder helper へ委譲し、StringBuilder 側で raw pointer/length pair を組み立てない構造にした。
- `nodesrc` の source policy と `tests/stdlib/memory_safety.n.md` に、raw append helper が public に戻らない regression を追加した。

## 検証

- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib\alloc\io\bytebuilder -i stdlib\alloc\string\builder -i stdlib\alloc\string\builder_ext.nepl --no-tree -o tmp\agent1-bytebuilder-typed-source-copy-docs-after.json -j 1 --dist web\dist --assert-io`: total=6, passed=6
- `node nodesrc/tests.js -i tests\stdlib\memory_safety.n.md --no-tree -o tmp\agent1-bytebuilder-typed-source-copy-memory-safety-after.json -j 1 --dist web\dist --assert-io`: total=60, passed=60
