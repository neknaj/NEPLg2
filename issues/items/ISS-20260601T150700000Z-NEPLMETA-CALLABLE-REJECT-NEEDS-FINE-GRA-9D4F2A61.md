---
id: ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61
title: ".neplmeta callable reject needs fine-grained diagnostic and root fix"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/typecheck/materializer.rs; nepl-core/src/diagnostic_codes.rs; nepl-web/src/lib.rs; nodesrc/bench_materialized_compile_fallbacks.js"
---

# ISS-20260601T150700000Z-NEPLMETA-CALLABLE-REJECT-NEEDS-FINE-GRA-9D4F2A61: .neplmeta callable reject needs fine-grained diagnostic and root fix

## 概要

`.neplmeta` field accessor materializer を通した後、`core/char` fixture の warm materialized compile fallback は `type.public_surface.materializer.field_accessor_unsupported` ではなく `type.public_surface.materializer.callable_rejected` で止まるようになった。

この diagnostic は `MissingCallableLinkSymbol`、`CallableLinkNameMismatch`、`CallableTypeExpected`、`CallableArityMismatch`、`CallableEffectMismatch`、`CallableSignatureHashMismatch` をまとめているため、次に直すべき root metadata 不整合を benchmark JSON だけから判断できない。

## 対象

- `nepl-core/src/typecheck/materializer.rs`
- `nepl-core/src/diagnostic_codes.rs`
- `nepl-web/src/lib.rs`
- `nodesrc/bench_materialized_compile_fallbacks.js`

## 根拠

- `tmp/materialized-field-accessor-20260601.json` では warm compile 3 回の `materialized_fallback_diagnostic_code_counts` が `type.public_surface.materializer.callable_rejected: 3` になった。
- `materialized_body_missing_fallbacks_delta_sum=0` と `neplobj_candidate_body_missing_surfaces_delta_sum=0` のままであり、`.neplobj` body fragment 不足へはまだ到達していない。
- field accessor kind の復元後も source fallback するため、残る blocker は callable surface の metadata authority か diagnostic 分解不足である。

## 問題

`callable_rejected` は materializer の複数の fail-closed 条件を 1 つの code に畳み込んでいる。

この状態では、次のどれが起きているかを CI / benchmark report から特定できない。

- link symbol が欠落している。
- link symbol の name が public entry name と一致していない。
- public surface の type term が function ではない。
- callable arity と function parameter count が一致していない。
- surface effect と function type effect が一致していない。
- link symbol の signature hash が public type term の stable hash と一致していない。

## 修正方針

まず diagnostic を細分化し、benchmark summary で root blocker を typed code として観測できるようにする。

- `PublicSurfaceMaterializeRejectReason` の callable metadata reject を個別の `TypeDiagnosticCode` へ写す。
- `OtherCoreError` の numeric fallback reason は互換性のため残し、既存の `last_fallback_diagnostic_code` を使って詳細を見る。
- message 文字列の解析ではなく、compiler-owned diagnostic registry を authority にする。
- 詳細 code が出た後、その code に対応する metadata 生成側または materializer 側を別 checkpoint で修正する。

## 受け入れ条件

- `type.public_surface.materializer.callable_rejected` が benchmark summary の主要 blocker として残らない。
- callable metadata reject が missing link symbol / name mismatch / type expected / arity mismatch / effect mismatch / signature hash mismatch のいずれかの stable diagnostic code として出る。
- `field_accessor_unsupported` は再発しない。
- `.neplobj` body missing、memo_call、PrivateCache proof はこの issue では変更しない。

## 解決

`PublicSurfaceMaterializeRejectReason` の callable metadata reject を個別の `TypeDiagnosticCode` に写すようにした。

既存の `type.public_surface.materializer.callable_rejected` は互換性のため残したが、既知の callable metadata reject 6 種は次の stable code へ分かれる。

- `type.public_surface.materializer.callable.missing_link_symbol`
- `type.public_surface.materializer.callable.link_name_mismatch`
- `type.public_surface.materializer.callable.type_expected`
- `type.public_surface.materializer.callable.arity_mismatch`
- `type.public_surface.materializer.callable.effect_mismatch`
- `type.public_surface.materializer.callable.signature_hash_mismatch`

実測では `tmp/materialized-callable-reject-detail-20260601.json` の warm compile 3 回すべてが `type.public_surface.materializer.callable.arity_mismatch` になった。したがって次の root gap は arity metadata の生成または materializer 側検査であり、`.neplobj` body missing にはまだ到達していない。

## 検証

- `cargo test -p nepl-core diagnostic_codes_have_unique_serialized_names --lib -- --nocapture`
- `cargo test -p nepl-core materializer_reject_reason_maps_to_stable_diagnostic_code --lib -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`
- `cargo check -p nepl-web --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc\bench_materialized_compile_fallbacks.js --out tmp\materialized-callable-reject-detail-20260601.json`
- `node nodesrc\test_bench_materialized_compile_fallbacks.js`

## 関連 issue

- `ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B`
- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649`
- `ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1`
