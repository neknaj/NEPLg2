---
id: ISS-20260531T035335856Z-MEMO-CALL-NEEDS-TYPED-FUNCTION-IDENT-3B612E6C
title: "memo_call needs typed function identity for higher-order values"
area: core
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/typecheck; nepl-core/src/resource/model.rs; nepl-core/src/resource/lower_call.rs"
---

# ISS-20260531T035335856Z-MEMO-CALL-NEEDS-TYPED-FUNCTION-IDENT-3B612E6C: memo_call needs typed function identity for higher-order values

## 概要

memo_call cannot rely on ResourceOp::FunctionValue name strings because overloads, module provenance, generic instantiation, function effect, and definition identity all affect the private cache namespace and purity proof.

## 対象

- `nepl-core/src/typecheck; nepl-core/src/resource/model.rs; nepl-core/src/resource/lower_call.rs`

## 根拠

- `HirExprKind::FnValue` が backend symbol 文字列だけを保持すると、同名 overload、generic instantiation、effect、definition identity を Resource IR へ伝えられない。
- `FunctionAliasTable` が `Vec<String>` を保持すると、indirect call summary 適用と memoization の purity proof が表示名 / backend symbol に依存する。
- Resource summary body hash が function value の `name` だけを見ると、同じ symbol でも型引数や関数型が異なる関数値を同一 body として扱う危険がある。

## 問題

memo_call cannot rely on ResourceOp::FunctionValue name strings because overloads, module provenance, generic instantiation, function effect, and definition identity all affect the private cache namespace and purity proof.

## 影響

String-based function values can create stale cache identity, confuse overloaded functions, or make memoized function purity depend on an unstable backend/table name instead of typed compiler identity.

## 修正方針

Introduce a typed function value identity that includes definition identity, module/source provenance, resolved signature/effect, and generic type arguments, then use it in typecheck, Resource IR lowering, function alias tracking, Resource summary body hash, and memo_call primitive checks.

## 検証

Focused typecheck and Resource IR tests should reject ambiguous or unresolved function values and should distinguish same-name functions, generic instantiations, and pure/impure overloads in memo_call.

## 解決

2026-05-31 に、`memo_call` Phase 1 の前提になる typed function identity checkpoint として解決した。

- `FunctionValueIdentity` を追加し、backend symbol、compile-time definition id、function type、surface effect、resolved type args を関数値の payload として保持するようにした。
- `HirExprKind::FnValue`、`ResourceOp::FunctionValue`、Resource IR lowering、monomorphize、function alias tracking を `FunctionValueIdentity` に接続した。
- indirect call の borrow / owner / effect / initialized / collection slot summary 適用は、alias table に残った typed identity から既存 summary index 用の `symbol()` を取り出す形にした。
- Resource summary body hash は namespace を `neplg2-resource-function-body-v3` に上げ、function value の symbol だけでなく function type、effect、type args を hash するようにした。`DefId` は compile session 内補助 identity なので、長寿命 cache key へは直接入れない。
- function value を依存として集める `summary_dependency` と collection slot relevance は legacy `name` ではなく typed identity を authority にした。
- `memo_call` primitive 本体、`MemoKey` / `MemoValue`、`PrivateCache` effect、backend sealed cache representation は、この issue では実装しない。次段階は `ISS-20260531T060756264Z-MEMO-CALL-PHASE1-NEEDS-COMPILER-KNOW-2DB7C53C` 以降で扱う。

検証:

- `cargo check -p nepl-core`
- `cargo test -p nepl-core --tests --no-run`
- `cargo test -p nepl-core resource_ir_lowering_preserves_call_targets_and_callback_places --test resource_ir -- --nocapture`
- `cargo test -p nepl-core resource_summary_value_cache::body_hash --lib -- --nocapture`
- `cargo test -p nepl-core summary_dependents_cover_nested_calls_function_values_and_self_recursion --lib -- --nocapture`
- `cargo test -p nepl-core wasm_precheck_reports_indirect_signature_unsupported_code --test codegen_diagnostics -- --nocapture`
