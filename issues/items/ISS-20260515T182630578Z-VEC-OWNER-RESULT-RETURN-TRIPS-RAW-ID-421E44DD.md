---
id: ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD
title: "Vec owner Result return trips raw identity escape in fs normalize range push"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/effect_summary_identity.rs; nepl-core/src/resource/effect_return_escape.rs; stdlib/std/fs/path/normalize/range_stack.nepl"
---

# ISS-20260515T182630578Z-VEC-OWNER-RESULT-RETURN-TRIPS-RAW-ID-421E44DD: Vec owner Result return trips raw identity escape in fs normalize range push

## 概要

std/fs/stat.nepl doctest stops in fs_normalize_range_push with resource.raw.identity_escape because Result<Vec<i32>, i32> owner returns from Vec push are still treated as raw internal allocation identity escapes.

## 対象

- `nepl-core/src/resource/effect_summary_identity.rs; nepl-core/src/resource/effect_return_escape.rs; stdlib/std/fs/path/normalize/range_stack.nepl`

## 根拠

- `node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree -o tmp/agent1-fs-stat-region.json -j 1 --dist web/dist --assert-io` が `stdlib/std/fs/stat.nepl::doctest#1` の compile phase で失敗した。
- diagnostic は `error[resource.raw.identity_escape]: pure function 'fs_normalize_range_push__Vec_T_i32_i32_i32__Result_T_E_Vec_T_i32_i32__pure' returns raw address identity from internal Alloc` だった。
- `fs_normalize_range_push` は `Vec<i32>` owner を `v::push` で更新し、`Result<Vec<i32>, i32>` として返す safe typed owner boundary であり、`i32` raw address や `MemPtr<T>` raw pointer を public surface へ返しているわけではない。
- 既存の `ISS-20260515T065425800Z-RESOURCE-EFFECT-IDENTITY-ESCAPE-TREA-9460C7FB` は `str` / `ByteBuf` / `StringBuilder` owner return を修正済みだが、`Result<Vec<T>, E>` payload の owner-protected raw identity までは覆えていない可能性がある。

## 問題

std/fs/stat.nepl doctest stops in fs_normalize_range_push with resource.raw.identity_escape because Result<Vec<i32>, i32> owner returns from Vec push are still treated as raw internal allocation identity escapes.

## 影響

Safe filesystem path normalization and any pure API returning owner-protected Vec results can be rejected before the caller reaches the intended memory-safety checks. Weakening identity_escape globally would hide real MemPtr/i32 raw pointer leaks, so the compiler must distinguish Vec/Result owner carriers precisely.

## 修正方針

Extend Resource IR raw identity escape analysis with projection-aware owner protection for Vec and Result payloads, preserving diagnostics for i32/MemPtr raw address leaves while allowing typed owner carriers to leave pure functions.

## 検証

Focused Resource IR regression for Result<Vec<i32>, i32> owner return; node nodesrc/tests.js -i stdlib/std/fs/stat.nepl --no-tree --assert-io
