---
id: ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31
title: ".neplmeta needs structured public surface payload"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/loader.rs"
---

# ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31: .neplmeta needs structured public surface payload

## 概要

.neplmeta currently stores TypedPublicSignatureTable stable text/hash only, which is not enough to rebuild TypeCtx/Env for imported modules without reparsing and retypechecking bodies.

## 対象

- `nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/loader.rs`

## 根拠

- 未記入

## 問題

.neplmeta currently stores TypedPublicSignatureTable stable text/hash only, which is not enough to rebuild TypeCtx/Env for imported modules without reparsing and retypechecking bodies.

## 影響

Base compile time cannot drop toward the stdlib prechecked artifact target while dependency modules still need full body parse/typecheck for public callable/type/trait/impl surfaces.

## 修正方針

Add arena-independent structured public surface entries to .neplmeta: module canonical path, public callable/type/trait/impl headers, trait bounds, noshadow and symbol policy. Do not store TypeId, Span, SourceMap, typed HIR, Resource IR, or diagnostics.

## 検証

Unit tests should materialize stable surface payloads for public functions, types, traits, impl headers, reexports, prelude edges, and body-only edits without changing payload hashes.
