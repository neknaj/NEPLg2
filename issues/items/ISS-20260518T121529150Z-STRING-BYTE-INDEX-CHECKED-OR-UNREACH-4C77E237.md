---
id: ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237
title: "string byte index checked-or-unreachable helper keeps unsafe trap surface"
area: stdlib
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "stdlib/alloc/string/byte_index.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js, nodesrc/test_stdlib_string_access_boundary.js"
---

# ISS-20260518T121529150Z-STRING-BYTE-INDEX-CHECKED-OR-UNREACH-4C77E237: string byte index checked-or-unreachable helper keeps unsafe trap surface

## 概要

stdlib/alloc/string/byte_index.nepl still exposes string_byte_at_checked_or_unreachable with #intrinsic unreachable. It is witness-based before raw read, but the public infallible helper keeps a trap-based unsafe surface and conflicts with the global no-unsafe-helper policy.

## 対象

- `stdlib/alloc/string/byte_index.nepl, nodesrc/test_stdlib_no_unsafe_helpers.js, nodesrc/test_stdlib_string_access_boundary.js`

## 根拠

- 未記入

## 問題

stdlib/alloc/string/byte_index.nepl still exposes string_byte_at_checked_or_unreachable with #intrinsic unreachable. It is witness-based before raw read, but the public infallible helper keeps a trap-based unsafe surface and conflicts with the global no-unsafe-helper policy.

## 影響

Hot-path callers can continue relying on an infallible trap helper instead of threading a typed StringByteIndex or Option/Result proof. That weakens the Stage 6 goal that static-check helper mistakes are visible through typed APIs and source policy.

## 修正方針

Replace the transitional checked-or-unreachable API with a typed proof or checked result API, update hot-path callers to carry StringByteIndex/Option evidence, and remove the source-policy contradiction without adding a broad allowlist.

## 検証

Run string access/source policy tests, stdlib string focused doctests, selfhost lexer/module focused tests, and memory_safety compile_fail regressions.
