---
id: ISS-20260429T215616519Z-STRINGBUILDER-SOURCE-POLICY-KEEPS-ST-256D72FD
title: "StringBuilder source policy keeps stale raw MemPtr owner contract"
area: test
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nodesrc/test_stdlib_string_no_unsafe_unwraps.js, nodesrc/test_stdlib_builder_owner_boundary.js"
---

# ISS-20260429T215616519Z-STRINGBUILDER-SOURCE-POLICY-KEEPS-ST-256D72FD: StringBuilder source policy keeps stale raw MemPtr owner contract

## 概要

Source policy regressions still require StringBuilder.data to be a bare MemPtr<u8>, even though the Resource IR owner-boundary fix moved StringBuilder.data to Option<MemPtr<u8>>. The duplicate policy drift breaks CI and can pressure the implementation back toward the old raw owner design.

## 対象

- `nodesrc/test_stdlib_string_no_unsafe_unwraps.js, nodesrc/test_stdlib_builder_owner_boundary.js`

## 根拠

- `main` の CI run `25135358817` 以降で `Source policy regressions` が `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` の `StringBuilder must use owned byte storage` assertion により失敗していた。
- `stdlib/alloc/string.nepl` の現行 `StringBuilder` は `data <Option<MemPtr<u8>>>` / `len <i32>` / `cap <i32>` で、空 storage と所有 storage を型で区別する設計に移行済みである。
- `nodesrc/test_stdlib_builder_owner_boundary.js` はこの `Option<MemPtr<u8>>` 契約を要求していたが、`nodesrc/test_stdlib_string_no_unsafe_unwraps.js` だけが旧 `data <MemPtr<u8>>` 直持ち契約を要求していた。

## 問題

Source policy regressions still require StringBuilder.data to be a bare MemPtr<u8>, even though the Resource IR owner-boundary fix moved StringBuilder.data to Option<MemPtr<u8>>. The duplicate policy drift breaks CI and can pressure the implementation back toward the old raw owner design.

## 影響

Main branch CI fails in Source policy regressions, and the test suite no longer protects the intended empty-vs-owning builder state model consistently.

## 修正方針

Centralize ByteBuilder/StringBuilder owner-boundary source policy checks and make the string unsafe-unwrap policy depend on the same Option<MemPtr<u8>> contract as the dedicated builder owner-boundary policy.

## 検証

Run the source policy tests for StringBuilder and builder owner boundaries, then run issue metadata checks.

## 2026-04-30 対応結果

- `nodesrc/source_policy/stdlib_builder_owner.js` を追加し、ByteBuilder / StringBuilder の owner-boundary source policy を共通化した。
- `StringBuilder` の policy は `data <Option<MemPtr<u8>>>`、`string_builder_from_owned_ptr`、`string_builder_with_len`、`get_ref` 経由の borrow-only 書き込みを一貫して要求するようにした。
- `nodesrc/test_stdlib_builder_owner_boundary.js` と `nodesrc/test_stdlib_string_no_unsafe_unwraps.js` は同じ `assertStringBuilderOwnerBoundary` を使うため、片方だけが旧 `MemPtr<u8>` 直持ち契約へ drift する余地をなくした。
- Source policy は現行の Resource IR / Stage 6 builder owner model を監視し、旧 raw owner sentinel 設計へ戻らないことを固定する。

## 2026-04-30 検証

- `node nodesrc/test_stdlib_string_no_unsafe_unwraps.js`: passed
- `node nodesrc/test_stdlib_builder_owner_boundary.js`: passed
