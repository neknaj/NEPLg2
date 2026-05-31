---
id: ISS-20260531T035335856Z-MEMO-CALL-NEEDS-TYPED-FUNCTION-IDENT-3B612E6C
title: "memo_call needs typed function identity for higher-order values"
area: core
status: open
resolved: false
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

- 未記入

## 問題

memo_call cannot rely on ResourceOp::FunctionValue name strings because overloads, module provenance, generic instantiation, function effect, and definition identity all affect the private cache namespace and purity proof.

## 影響

String-based function values can create stale cache identity, confuse overloaded functions, or make memoized function purity depend on an unstable backend/table name instead of typed compiler identity.

## 修正方針

Introduce a typed function value identity that includes definition identity, module/source provenance, resolved signature/effect, and generic type arguments, then use it in typecheck, Resource IR lowering, function alias tracking, Resource summary body hash, and memo_call primitive checks.

## 検証

Focused typecheck and Resource IR tests should reject ambiguous or unresolved function values and should distinguish same-name functions, generic instantiations, and pure/impure overloads in memo_call.
