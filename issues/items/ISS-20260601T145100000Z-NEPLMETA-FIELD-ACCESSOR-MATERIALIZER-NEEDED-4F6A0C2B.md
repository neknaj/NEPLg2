---
id: ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B
title: ".neplmeta field accessor materializer needed for stdlib body skip"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-06-01
updated: 2026-06-01
target: "nepl-core/src/typecheck/materializer.rs; nepl-core/src/typecheck/field_apply.rs; nepl-core/src/typecheck/public_surface.rs; nepl-web/src/lib.rs; nodesrc/bench_materialized_compile_fallbacks.js"
---

# ISS-20260601T145100000Z-NEPLMETA-FIELD-ACCESSOR-MATERIALIZER-NEEDED-4F6A0C2B: .neplmeta field accessor materializer needed for stdlib body skip

## 概要

`.neplmeta` materialized compile の warm fallback reason を typed diagnostic code で分解した結果、`core/char` fixture は `.neplobj` body missing ではなく `type.public_surface.materializer.field_accessor_unsupported` で source fallback していることが分かった。

## 対象

- `nepl-core/src/typecheck/materializer.rs`
- `nepl-core/src/typecheck/field_apply.rs`
- `nepl-core/src/typecheck/public_surface.rs`
- `nepl-web/src/lib.rs`
- `nodesrc/bench_materialized_compile_fallbacks.js`

## 根拠

- `nodesrc/bench_materialized_compile_fallbacks.js --out tmp/materialized-fallback-detail-20260601.json` で、warm compile 3 回の `materialized_fallback_diagnostic_code_counts` が `type.public_surface.materializer.field_accessor_unsupported: 3` になった。
- `materialized_body_missing_fallbacks_delta_sum=0` であり、今回の fixture は `.neplobj` body fragment 不足では止まっていない。
- `PublicCallableSurface.field_accessor` は既に stable public surface に含まれるが、`typecheck/materializer` は `FieldAccessorUnsupported` として fail-closed に拒否している。
- field accessor helper は通常 callable と異なり、`BindingKind::Func.field_accessor` を通じて HIR の field projection / store lowering と Resource / SourceCapability 境界へ接続する。

## 問題

field accessor callable を `.neplmeta` から復元できないため、stdlib dependency artifact が projection と semantic materializer を通過しても、`get` / `get_ref` / `put` を含む module は source fallback へ戻る。

この fallback は `.neplobj` を実装しても解消しない。selected callable body 以前に、typecheck materializer が current session の callable environment を再構築できていないためである。

## 影響

`core/char` のような小さい stdlib module でも warm compile が source fallback し、base / warm compile time が 0.5 秒未満、0.1 秒未満の目標へ近づきにくい。

## 修正方針

field accessor callable surface を通常 callable と同一視せず、`PublicFieldAccessorKind` を保持したまま `BindingKind::Func.field_accessor` へ復元する。

- `PublicCallableSurface.field_accessor` がある場合でも、stable link symbol、name、arity、effect、signature hash を検証する。
- `get` / `get_ref` / `put` の accessor kind を `BindingKind::Func.field_accessor` へ戻す。
- `def_id=None` の `.neplmeta` callable のまま direct call 専用にし、`@func` / `memo_call @func` / indirect call の function value identity へは広げない。
- field selector と aggregate type の具体的な HIR lowering は既存の `field_apply` / selected call path に委ね、materializer 側で型を推測しない。
- SourceCapability や Resource proof を bypass しない。field accessor が owner/raw aggregate boundary を要求する場合は、既存の source capability proof と同じ authority を必要とする。

## 受け入れ条件

- `FieldAccessorUnsupported` が原因の materialized fallback が、対象 fixture で別の root blocker または body missing へ進む。
- `materialized_fallback_diagnostic_code_counts` が `type.public_surface.materializer.field_accessor_unsupported` だけに偏らなくなる。
- `.neplmeta` 由来 field accessor は direct call では使えるが、function value identity には使えない。
- field accessor kind の mismatch、signature hash mismatch、arity/effect mismatch は fail-closed に拒否される。
- `memo_call` / private cache proof / `.neplobj` code fragment reuse はこの issue では変更しない。

## 検証

- `cargo test -p nepl-core materializer --lib -- --nocapture`
- `cargo check -p nepl-core -p nepl-language`
- `cargo check -p nepl-web --manifest-path nepl-web\Cargo.toml`
- `trunk build --release`
- `node nodesrc\bench_materialized_compile_fallbacks.js --out tmp\materialized-fallback-detail-20260601.json`
- `node tests\compiler\tree\run.js`

## 関連 issue

- `ISS-20260531T111205690Z-BINARY-INTERMEDIATE-ARTIFACTS-NEEDED-1C570649`
- `ISS-20260601T193116311Z-NEPLMETA-TRAIT-IMPL-MATERIALIZER-NEEDED-D3A0C2F1`
- `ISS-20260531T223904937Z-NEPLMETA-NEEDS-TYPECHECK-SURFACE-MAT-E7FF61B7`
