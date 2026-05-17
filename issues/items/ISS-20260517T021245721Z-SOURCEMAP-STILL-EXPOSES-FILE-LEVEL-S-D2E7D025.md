---
id: ISS-20260517T021245721Z-SOURCEMAP-STILL-EXPOSES-FILE-LEVEL-S-D2E7D025
title: "SourceMap still exposes file-level source capability aggregate queries"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_map.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T021245721Z-SOURCEMAP-STILL-EXPOSES-FILE-LEVEL-S-D2E7D025: SourceMap still exposes file-level source capability aggregate queries

## 概要

Exact use-site source capability proof is now the production authority, but SourceCapabilities and SourceMap still expose public file-level aggregate query methods such as raw_memory_operation_boundary_allowed(file, op). They derive from exact use-site artifacts but return file-wide authority and can be reused by future checker code to bypass exact span proof.

## 対象

- `nepl-core/src/source_map.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- 未記入

## 問題

Exact use-site source capability proof is now the production authority, but SourceCapabilities and SourceMap still expose public file-level aggregate query methods such as raw_memory_operation_boundary_allowed(file, op). They derive from exact use-site artifacts but return file-wide authority and can be reused by future checker code to bypass exact span proof.

## 影響

Static-check maintenance can regress from exact typed proof artifacts back to file-scoped capability checks. That would make raw memory, owner aggregate, and compiler memory gates easier to over-authorize and harder for Rust exhaustiveness/source policy to catch.

## 修正方針

Remove the file-level aggregate capability query methods from SourceCapabilities and SourceMap. Keep only exact _at queries and the narrow _within helper needed for diagnostic spans, and update source policy to reject reintroduction of file-id aggregate capability queries.

## 検証

Run cargo fmt/check for nepl-core, source_map focused tests, and nodesrc/test_static_check_boundary_responsibility.js.

## 2026-05-17 Agent 1 修正

`SourceCapabilities` / `SourceMap` から、file id だけで「その file に何らかの capability があるか」を返す aggregate query を削除した。production API は exact `*_allowed_at(span, ...)` と、diagnostic span が call-head より広い場合に限って使う `*_allowed_within(span, ...)` に限定した。

`loader.rs` の source capability regression は、production API へ broad query を戻さず、test-only の use-site 走査 helper で既存の「何らかの evidence があるか」検証を維持した。`nepl-core/tests/effects.rs` の facade safety 検査も exact span query へ移した。

`nodesrc/test_static_check_boundary_responsibility.js` に、`source_map.rs` が broad `pub fn ..._allowed(file_id, ...)` / `pub fn allows_...()` を再公開しない source policy を追加した。

検証:
- `cargo fmt -p nepl-core --check`
- `cargo check -p nepl-core`
- `node nodesrc/test_static_check_boundary_responsibility.js`
- `cargo test -p nepl-core source_map::tests -- --nocapture`
- `cargo test -p nepl-core raw_memory_boundary -- --nocapture`
