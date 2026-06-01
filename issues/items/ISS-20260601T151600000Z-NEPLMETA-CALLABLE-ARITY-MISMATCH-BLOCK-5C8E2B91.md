---
id: ISS-20260601T151600000Z-NEPLMETA-CALLABLE-ARITY-MISMATCH-BLOCK-5C8E2B91
title: ".neplmeta callable arity mismatch blocks materialized stdlib body skip"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/typecheck/public_surface.rs; nepl-core/src/typecheck/materializer.rs; nepl-core/src/typecheck/driver.rs; nodesrc/bench_materialized_compile_fallbacks.js"
---

# ISS-20260601T151600000Z-NEPLMETA-CALLABLE-ARITY-MISMATCH-BLOCK-5C8E2B91: .neplmeta callable arity mismatch blocks materialized stdlib body skip

## 概要

`.neplmeta` callable reject を細分化した結果、`core/char` fixture の warm materialized compile fallback は `type.public_surface.materializer.callable.arity_mismatch` で止まっていることが分かった。

## 根拠

- `tmp/materialized-callable-reject-detail-20260601.json` で、warm compile 3 回の `materialized_fallback_diagnostic_code_counts` が `type.public_surface.materializer.callable.arity_mismatch: 3` になった。
- `type.public_surface.materializer.field_accessor_unsupported` と `type.public_surface.materializer.callable_rejected` は消えた。
- `materialized_body_missing_fallbacks_delta_sum=0` と `neplobj_candidate_body_missing_surfaces_delta_sum=0` のままであり、`.neplobj` body missing にはまだ到達していない。

## 問題

`PublicCallableSurface.arity` と `PublicTypeTerm::Function.params.len()` が一致していない public surface がある。

根本原因は、`detect_field_accessor_fn` が `#intrinsic "get_field_ref"` を使う specialized wrapper まで field accessor facade として誤分類していたことである。

`core/field.get_ref` は selector を引数として受け取る 2 引数 field accessor facade だが、`core/mem/types.region_token_size_ref` や `core/mem/internal.region_token_raw_ref` は selector を literal として固定した 1 引数 wrapper である。これらを `PublicCallableSurface.field_accessor=GetRef` として保存すると、materializer は `get_ref` の 2 引数 ABI として復元しようとして `CallableArityMismatch` になる。

## 修正方針

`detect_field_accessor_fn` は次の条件を満たす場合だけ field accessor metadata を付ける。

- intrinsic 名が `get_field` / `get_field_ref` / `set_field` のいずれかである。
- function parameter 数と intrinsic argument 数が `FieldAccessorKind::argument_count()` と一致する。
- intrinsic argument が function parameter を同じ順序でそのまま渡している。

selector literal を固定する specialized wrapper は通常 callable として扱い、`.neplobj` が入るまでは selected callable body missing による source fallback へ進ませる。

## 受け入れ条件

- `type.public_surface.materializer.callable.arity_mismatch` が `core/char` fixture の warm materialized compile fallback から消える。
- `core/field.get` のような generic field accessor facade は field accessor metadata を維持する。
- `core/mem/types.region_token_size_ref` のような selector 固定 wrapper は field accessor metadata を持たない。
- `.neplobj` body missing へ到達した候補を stable link symbol / selected callable body hash / generic instantiation hash の実装 issue へ戻す。
- memo_call / PrivateCache proof はこの issue では変更しない。

## 検証

- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`
- `cargo check -p nepl-web --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc\bench_materialized_compile_fallbacks.js --out tmp\materialized-arity-mismatch-20260601.json`

## 解決

2026-06-01 に `detect_field_accessor_fn` を厳密化した。

- pass: `cargo test -p nepl-core typed_public_surface_does_not_mark_specialized_field_ref_wrapper_as_accessor --lib -- --nocapture`
- pass: `cargo test -p nepl-core typed_public_surface_keeps_field_accessor_kind_for_callable --lib -- --nocapture`
- pass: `cargo test -p nepl-core materializer_mvp_rejects_malformed_callable_surface_metadata --lib -- --nocapture`
- pass: `cargo check -p nepl-core -p nepl-language`
- pass: `cargo check -p nepl-web --manifest-path nepl-web\Cargo.toml`
- pass: `trunk build --release`
- pass: `node nodesrc\bench_materialized_compile_fallbacks.js --out tmp\materialized-field-wrapper-arity-20260601.json`

`tmp/materialized-field-wrapper-arity-20260601.json` では warm 3 回の fallback diagnostic が `backend.codegen.materialized_function_body_missing` になり、`type.public_surface.materializer.callable.arity_mismatch` は消えた。`materialized_body_missing_fallbacks_delta_sum=3`、`neplobj_candidate_body_missing_surfaces_delta_sum=15` なので、次は `.neplobj` / `.nepllink` 側の selected callable body artifact で扱う。

## 関連 issue

- `ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61`
- `ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B`
- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649`
