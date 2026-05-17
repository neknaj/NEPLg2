---
id: ISS-20260517T024243699Z-SOURCEMAP-EXPOSES-FILE-LEVEL-SOURCEC-AEFA0421
title: "SourceMap exposes file-level SourceCapabilities after exact use-site proof"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/source_map.rs; nepl-core/tests/effects.rs; nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260517T024243699Z-SOURCEMAP-EXPOSES-FILE-LEVEL-SOURCEC-AEFA0421: SourceMap exposes file-level SourceCapabilities after exact use-site proof

## 概要

SourceMap::capabilities(file_id) is still public, so external compiler users and integration tests can retrieve the whole capability set for a file after privileged evidence was narrowed to exact use sites.

## 対象

- `nepl-core/src/source_map.rs; nepl-core/tests/effects.rs; nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `SourceMap::capabilities(file_id)` が `pub` のまま残っており、crate 外の integration test から `SourceCapabilities` 全体を取得できた。
- exact use-site proof 導入後の compiler / typecheck gate は `SourceMap::*_allowed_at(span, ...)` へ寄せていたが、外部 API として file-level aggregate を取り出せると、後続実装がその集合を直接読んで exact query を迂回できる。
- `nepl-core/tests/effects.rs` もこの public accessor に依存していたため、テスト自身が file-level authority の利用例として残っていた。

## 問題

SourceMap::capabilities(file_id) is still public, so external compiler users and integration tests can retrieve the whole capability set for a file after privileged evidence was narrowed to exact use sites.

## 影響

This keeps a file-level authority escape hatch in the API and can let future static-check code bypass exact SourceMap queries, weakening the proof shape and hiding broad capability regressions.

## 修正方針

Make SourceMap capability access crate-internal, update external tests to query exact use-site predicates through SourceMap, and add source policy coverage so the public accessor cannot be reintroduced.

## 検証

- `cargo fmt -p nepl-core`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core source_map::tests -- --nocapture`: passed
- `cargo test -p nepl-core loader_does_not_mark_configured_stdlib_core_mem_facade_as_raw_memory_boundary --test effects -- --nocapture`: passed
- `cargo test -p nepl-core loader_does_not_mark_configured_stdlib_alloc_string_facade_as_raw_memory_boundary --test effects -- --nocapture`: passed
- `cargo test -p nepl-core loader_marks_configured_stdlib_implementation_boundaries_as_raw_memory_boundary --test effects -- --nocapture`: passed
