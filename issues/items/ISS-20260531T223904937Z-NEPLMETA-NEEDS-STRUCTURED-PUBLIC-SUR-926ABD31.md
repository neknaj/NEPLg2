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
target: "nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/loader.rs"
---

# ISS-20260531T223904937Z-NEPLMETA-NEEDS-STRUCTURED-PUBLIC-SUR-926ABD31: .neplmeta needs structured public surface payload

## 概要

.neplmeta currently stores TypedPublicSignatureTable stable text/hash only, which is not enough to rebuild TypeCtx/Env for imported modules without reparsing and retypechecking bodies.

## 対象

- `nepl-core/src/artifact.rs; nepl-core/src/typecheck/public_signature.rs; nepl-core/src/typecheck/public_surface.rs; nepl-core/src/loader.rs`

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

## 2026-06-01 public surface module split checkpoint

`TypedPublicSurfaceTable` の model / hash / builder / tests を `nepl-core/src/typecheck/public_surface.rs` へ分離した。

`public_signature.rs` は `TypedPublicSignatureTable` の stable text/hash 境界だけを担当する。structured surface は `.neplmeta` materializer の authority になるため、text signature の互換 hash と同じ module に置いたまま拡張しない。

`typecheck.rs` の public re-export は維持しており、`crate::typecheck::{PublicTypeTerm, TypedPublicSurfaceTable}` などの外部 API は変えない。`driver.rs` は signature builder と surface builder を別 module から呼ぶ。

この checkpoint はまだ `Named(String)` や generic parameter 参照を materializer authority として認めるものではない。次 checkpoint では stable nominal identity と binder-indexed generic reference を導入する。

検証:

- `cargo check -p nepl-core -p nepl-language`
- `cargo check --manifest-path nepl-web\Cargo.toml`
- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core typed_public_signature_hash --lib -- --nocapture`
- `cargo test -p nepl-core neplmeta --lib -- --nocapture`
- `trunk build --release`
- `node nodesrc/test_run_test_compiler_session.js`
- `node nodesrc/test_playground_compiler_session_policy.js`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-public-surface-split-20260601.json`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`

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
