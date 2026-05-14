---
id: ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E
title: "ByteBuilder grow cleanup still uses unreachable on dealloc failure"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-14
updated: 2026-05-14
target: "stdlib/alloc/io/bytebuilder/storage.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js"
---

# ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E: ByteBuilder grow cleanup still uses unreachable on dealloc failure

## 概要

byte_builder_realloc_region_or_free still traps with #intrinsic unreachable when old buffer cleanup fails after realloc_ptr failure. This keeps unsafe-helper debt in a Stage 6 owner boundary implementation and makes nodesrc/test_stdlib_no_unsafe_helpers.js fail.

## 対象

- `stdlib/alloc/io/bytebuilder/storage.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js`

## 根拠

- `stdlib/alloc/io/bytebuilder/storage.nepl` の `byte_builder_realloc_region_or_free` は `RegionToken<u8>` を受け取るにもかかわらず、失敗時 cleanup で `old_ptr` / `old_size` へ分解して `dealloc_ptr<u8>` を呼んでいた。
- `dealloc_ptr<u8>` の Err branch が `#intrinsic "unreachable"` へ落ち、`nodesrc/test_stdlib_no_unsafe_helpers.js` の通常 stdlib 実装 policy に違反していた。
- Stage 6 の方針では free obligation owner を `RegionToken` / `OwnedRegion` 系に集約し、`MemPtr` / ptr+size cleanup を owner discipline として再導入しない。

## 問題

byte_builder_realloc_region_or_free still traps with #intrinsic unreachable when old buffer cleanup fails after realloc_ptr failure. This keeps unsafe-helper debt in a Stage 6 owner boundary implementation and makes nodesrc/test_stdlib_no_unsafe_helpers.js fail.

## 影響

Static-check Stage 6 cannot claim stdlib owner-boundary code avoids unsafe helpers while ByteBuilder grow cleanup can turn an allocator invariant drift into an unreachable trap instead of a typed error path.

## 修正方針

Remove the unreachable branch from ByteBuilder grow cleanup, keep RegionToken owner transfer explicit, and return a typed StdErrorKind result after the old token has been passed to centralized cleanup. Re-run bytebuilder focused tests and no-unsafe-helper source policy.

## 検証

Run node nodesrc/test_stdlib_no_unsafe_helpers.js, node nodesrc/test_stdlib_builder_owner_boundary.js, node nodesrc/test_stdlib_memptr_owner_field_policy.js, and focused ByteBuilder/ByteBuf doctests.

## 解決内容

`byte_builder_realloc_region_or_free` は grow に必要な `size` / `ptr` を旧 token から取り出し、`realloc_ptr` 失敗時は `dealloc_region<u8> region` へ token owner を丸ごと渡す形へ変更した。Err branch も typed `StdErrorKind::OutOfMemory` へ畳み、通常 stdlib 実装から `#intrinsic "unreachable"` を排除した。

`nodesrc/source_policy/stdlib_builder_owner.js` には、ByteBuilder grow cleanup が `dealloc_region<u8> region` で owner token を消費し、`dealloc_ptr<u8> old_ptr old_size` へ戻らないことを固定する regression を追加した。

## 検証結果

- `node nodesrc/test_stdlib_no_unsafe_helpers.js`: pass。
- `node nodesrc/test_stdlib_builder_owner_boundary.js`: pass。
- `node nodesrc/test_stdlib_memptr_owner_field_policy.js`: pass。
- `node nodesrc/tests.js -i stdlib\alloc\io\bytebuilder -i tests\stdlib\byte_builder.n.md -i tests\stdlib\bytebuf_result.n.md --no-tree -o tmp\agent1-bytebuilder-grow-cleanup-focused.json -j 1 --dist web/dist`: total=14, passed=14。
- `node nodesrc/issues.js check --dir issues`: pass。
