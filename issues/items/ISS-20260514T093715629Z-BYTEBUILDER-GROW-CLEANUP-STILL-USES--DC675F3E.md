---
id: ISS-20260514T093715629Z-BYTEBUILDER-GROW-CLEANUP-STILL-USES--DC675F3E
title: "ByteBuilder grow cleanup still uses unreachable on dealloc failure"
area: stdlib
status: open
resolved: false
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

- 未記入

## 問題

byte_builder_realloc_region_or_free still traps with #intrinsic unreachable when old buffer cleanup fails after realloc_ptr failure. This keeps unsafe-helper debt in a Stage 6 owner boundary implementation and makes nodesrc/test_stdlib_no_unsafe_helpers.js fail.

## 影響

Static-check Stage 6 cannot claim stdlib owner-boundary code avoids unsafe helpers while ByteBuilder grow cleanup can turn an allocator invariant drift into an unreachable trap instead of a typed error path.

## 修正方針

Remove the unreachable branch from ByteBuilder grow cleanup, keep RegionToken owner transfer explicit, and return a typed StdErrorKind result after the old token has been passed to cleanup. Re-run bytebuilder focused tests and no-unsafe-helper source policy.

## 検証

Run node nodesrc/test_stdlib_no_unsafe_helpers.js, node nodesrc/test_stdlib_builder_owner_boundary.js, node nodesrc/test_stdlib_memptr_owner_field_policy.js, and focused ByteBuilder/ByteBuf doctests.
