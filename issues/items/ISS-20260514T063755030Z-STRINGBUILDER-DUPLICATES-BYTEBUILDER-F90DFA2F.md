---
id: ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F
title: "StringBuilder duplicates ByteBuilder raw MemPtr owner state"
area: stdlib
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/string/builder/**, stdlib/alloc/string/builder_ext.nepl, nodesrc/test_stdlib_memptr_owner_field_policy.js"
---

# ISS-20260514T063755030Z-STRINGBUILDER-DUPLICATES-BYTEBUILDER-F90DFA2F: StringBuilder duplicates ByteBuilder raw MemPtr owner state

## 概要

StringBuilder stores Option<MemPtr<u8>>, len, and cap directly even though ByteBuilder already owns the byte buffer boundary. This keeps a duplicate raw owner field in public StringBuilder state and preserves a Stage 6 MemPtr owner-field migration exception.

## 対象

- `stdlib/alloc/string/builder/**, stdlib/alloc/string/builder_ext.nepl, nodesrc/test_stdlib_memptr_owner_field_policy.js`

## 根拠

- `StringBuilder` と `ByteBuilder` がそれぞれ `Option<MemPtr<u8>>` / len / cap 相当の owner state を持つと、text builder と byte builder の両方で raw storage identity / free obligation を追跡する必要がある。
- Stage 6 の方針は `MemPtr` を non-owning pointer とし、free obligation owner は別型へ寄せることである。したがって `StringBuilder` 固有の raw owner field は増やすべき例外ではなく、`ByteBuilder` owner boundary へ集約すべき重複である。
- 旧 `StringBuilder` API は safe public surface として pure で使われていたため、委譲先の `ByteBuilder` / `ByteBuf` safe API も external effect としてではなく、raw-memory-boundary source 内で Resource IR が検査する internal memory effect として扱う必要がある。

## 問題

StringBuilder stores Option<MemPtr<u8>>, len, and cap directly even though ByteBuilder already owns the byte buffer boundary. This keeps a duplicate raw owner field in public StringBuilder state and preserves a Stage 6 MemPtr owner-field migration exception.

## 影響

The static-check complexity reduction cannot finish while text builders keep independent raw owner layouts. Callers and policies must reason about both ByteBuilder and StringBuilder storage identity instead of one owner boundary.

## 修正方針

Represent StringBuilder as a typed wrapper around ByteBuilder, delegate capacity, append, free, and finish to ByteBuilder/ByteBuf APIs, remove StringBuilder.data from the MemPtr owner-field baseline, and update policy/docs without compatibility aliases for the raw layout.

## 対応

- `StringBuilder` を `bytes: ByteBuilder` だけを持つ typed wrapper に変更した。
- `string_builder_with_capacity_result` / `string_builder_reserve_result` / append / free / build を `ByteBuilder` / `ByteBuf` owner boundary へ委譲した。
- `StringBuilder` 固有の `Option<MemPtr<u8>>` / len / cap raw owner layout と、`string_builder_from_owned_ptr` / `string_builder_with_len` の raw-layout helper を削除した。
- `ByteBuilder` / `ByteBuf` の safe buffer API を pure surface に揃え、raw memory effect は各 raw-memory-boundary source 内の Resource IR / source capability gate で検査する形に戻した。
- `nodesrc/test_stdlib_memptr_owner_field_policy.js` の transitional baseline から `StringBuilder.data` を削除し、残件を 4 field に下げた。
- StringBuilder source policy を、raw byte mutation の直接実装ではなく `ByteBuilder` 委譲を検査する内容へ更新した。

## 検証

- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed。
- `node nodesrc/test_stdlib_bytebuf_utf8_boundary.js`: passed。
- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed。
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: passed。`4 transitional field(s)`。
- `node nodesrc/test_stdlib_string_doc_no_boilerplate.js`: passed。
- `node nodesrc/tests.js -i stdlib/alloc/string/builder -i stdlib/alloc/string/builder_ext.nepl -i tests/stdlib/string.n.md -i tests/stdlib/string_char.n.md --no-tree -o tmp/agent1-stringbuilder-bytebuilder-string.json -j 1 --dist web/dist`: 20/20 passed。
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md -i tests/stdlib/bytebuf_result.n.md --no-tree -o tmp/agent1-bytebuilder-bytebuf-surface-noraw.json -j 1 --dist web/dist`: 9/9 passed。
- 広めの `selfhost_req` focused run は 13/15 passed。失敗 2 件は `test_req_file_io` の `len` import と `test_req_byte_manipulation` の `unwrap_ok` / collection helper import 由来の fixture drift であり、今回触った `test_req_string_builder` は passed。別 issue `ISS-20260514T071055890Z-SELFHOST-REQ-DOCTESTS-STILL-RELY-ON--F9CC30E7` として記録した。
