---
id: ISS-20260601T151600000Z-NEPLMETA-CALLABLE-ARITY-MISMATCH-BLOCK-5C8E2B91
title: ".neplmeta callable arity mismatch blocks materialized stdlib body skip"
area: core
status: open
resolved: false
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

これは次のいずれかである可能性が高い。

- public surface 生成側が、captured parameter、implicit receiver、type parameter、zero-arg/unit lambda、または curried-looking function type を callable arity として誤って数えている。
- materializer 側が NEPLg2.1 の function type canonicalization と callable application arity の境界を同一視しすぎている。
- field accessor arity 検査と通常 callable arity 検査の責務が混ざっている。

## 修正方針

まずどの entry が mismatch しているかを観測できるようにする。

- materializer reject diagnostic の message だけに依存せず、benchmark / debug report から entry name と surface arity / parameter count を読めるようにする。
- `PublicCallableSurface.arity` の生成箇所と `PublicTypeTerm::Function.params` の生成箇所を比較し、どちらが source callable application arity の authority かを固定する。
- field accessor は kind 固有 arity を維持し、通常 callable の補正と混ぜない。

## 受け入れ条件

- `type.public_surface.materializer.callable.arity_mismatch` の対象 entry と数値差分が観測できる。
- 生成側または materializer 側の root cause を直し、対象 fixture が arity mismatch では止まらない。
- `.neplobj` body missing へ到達した場合は、その候補を stable link symbol / selected callable body hash / generic instantiation hash の実装 issue へ戻す。
- memo_call / PrivateCache proof はこの issue では変更しない。

## 検証

- `cargo test -p nepl-core typed_public_surface --lib -- --nocapture`
- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`
- `cargo check -p nepl-web --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc\bench_materialized_compile_fallbacks.js --out tmp\materialized-arity-mismatch-20260601.json`

## 関連 issue

- `ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61`
- `ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B`
- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649`
