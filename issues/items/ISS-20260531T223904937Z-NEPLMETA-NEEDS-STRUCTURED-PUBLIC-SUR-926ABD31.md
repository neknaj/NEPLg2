---
id: ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31
title: ".neplmeta needs structured public surface payload"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-06-01
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

## 2026-06-01 structured payload checkpoint

`TypedPublicSurfaceTable` を追加し、`.neplmeta` payload が既存の `TypedPublicSignatureTable` と structured public surface の両方を持つようにした。

structured surface は public callable、struct、enum、trait、impl header を arena 非依存の enum/struct として保持する。`TypeId`、`Span`、`FileId`、`SourceMap`、`ImportResolution`、typed HIR、Resource IR、diagnostic span は保存しない。

`NeplMetaArtifactHeader` に `structured_public_surface_hash` と `structured_public_surface_entry_count` を追加し、payload consistency check でも structured surface hash / entry count の不一致を拒否する。artifact 形状を変更したため `.neplmeta` schema、artifact hash version、compiler identity は v2 に上げた。

Web `CompilerSession` stats JSON には structured public surface entry count と hash を追加した。payload 本体は出さない。

追加検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`

残件:

- `PublicTypeTerm::Named(String)` を materializer authority にしない。stable nominal identity は kind、module/source identity、name、arity、definition hash を持つ structured identity へ置き換える。
- `GenericParam(PublicTypeParam)` を materializer authority にしない。binder-indexed parameter reference を導入し、同名 nested generic parameter の衝突を防ぐ。
- callable surface に field accessor kind と stable public ABI/link symbol を追加する。ただし span-derived `mangle_function_symbol_for_def` は保存しない。
- generic impl parameter / bound を round-trip できるようにするか、materializer では fail-closed に拒否する。
- reexport / prelude edge と module canonical path を per-module `.neplmeta` surface に追加する。
