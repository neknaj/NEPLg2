---
id: ISS-20260513T062850064Z-SOURCE-POLICY-STILL-FORBIDS-RAW-BOUN-664EA812
title: "source policy still forbids raw-boundary internal allocation identity returns"
area: nodesrc
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nodesrc/test_static_check_boundary_responsibility.js,nepl-core/src/compiler.rs"
---

# ISS-20260513T062850064Z-SOURCE-POLICY-STILL-FORBIDS-RAW-BOUN-664EA812: source policy still forbids raw-boundary internal allocation identity returns

## 概要

raw memory boundary 内部で internal allocation identity を返すことは、callee summary を caller 側へ伝搬して外部 escape を検査する現在の設計では必要な capability になった。しかし source-policy は古い設計のまま RawAddressEscapeFromInternalAlloc を常に raw-boundary で抑制不可と要求しており、現在の検査責務を誤って失敗扱いにする。

## 対象

- `nodesrc/test_static_check_boundary_responsibility.js,nepl-core/src/compiler.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` で `nodesrc/test_static_check_boundary_responsibility.js` が `RawAddressEscapeFromInternalAlloc` を常に raw-boundary で抑制不可とする古い正規表現により失敗していた。
- 現在の `nepl-core/src/compiler.rs` は raw-memory-boundary source capability を持つ実装 module 内でのみ `RawAddressEscapeFromInternalAlloc` を許可し、`nepl-core/tests/resource_ir.rs` の caller-side summary 伝搬 regression で外部 escape を検査している。

## 問題

raw memory boundary 内部で internal allocation identity を返すことは、callee summary を caller 側へ伝搬して外部 escape を検査する現在の設計では必要な capability になった。しかし source-policy は古い設計のまま RawAddressEscapeFromInternalAlloc を常に raw-boundary で抑制不可と要求しており、現在の検査責務を誤って失敗扱いにする。

## 影響

warn-only とはいえ静的検査の責務テストが実装と矛盾し、今後の修正で raw boundary 内部の正当な allocator 実装と caller 側 escape 検査のどちらを守るべきか判断しづらくなる。

## 修正方針

RawAddressEscapeFromInternalAlloc は UnsafeMemoryInPureFunction と同様に source_map の raw_memory_boundary capability でのみ抑制できること、ただし dedicated match branch と caller-side propagation regression が存在することを source-policy で検査する。

## 検証

- `node nodesrc/test_static_check_boundary_responsibility.js`

## 2026-05-13 修正

`nodesrc/test_static_check_boundary_responsibility.js` を現在の静的検査設計へ合わせた。`RawAddressEscapeFromInternalAlloc` は専用 match branch で `source_map.raw_memory_boundary_allowed(span.file_id)` による明示 capability がある場合のみ抑制できることを検査し、さらに `resource_effect_gate_allows_raw_identity_escape_inside_raw_boundary` と `resource_ir_effect_check_propagates_internal_alloc_return_summary` の regression が存在することを source-policy で確認するようにした。
