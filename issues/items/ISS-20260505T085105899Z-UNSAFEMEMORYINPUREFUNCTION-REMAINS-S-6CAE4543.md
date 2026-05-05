---
id: ISS-20260505T085105899Z-UNSAFEMEMORYINPUREFUNCTION-REMAINS-S-6CAE4543
title: "UnsafeMemoryInPureFunction remains shadow-only"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/compiler.rs,nepl-core/src/diagnostic_codes.rs,tests/compiler/move_effect.n.md"
---

# ISS-20260505T085105899Z-UNSAFEMEMORYINPUREFUNCTION-REMAINS-S-6CAE4543: UnsafeMemoryInPureFunction remains shadow-only

## 概要

Resource IR emits UnsafeMemoryInPureFunction for raw memory helper calls in pure functions, but compiler mapping returns None so the diagnostic is shadow-only outside raw-memory boundary.

## 対象

- `nepl-core/src/compiler.rs,nepl-core/src/diagnostic_codes.rs,tests/compiler/move_effect.n.md`

## 根拠

- 修正前は、pure user function から `core/mem` の raw memory helper を直接呼ぶ source が、別の raw cell violation に当たらない限り compile success になった。
- Resource IR 側は `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` を生成していたが、compiler pipeline の `resource_effect_boundary_diagnostic_to_error` がこの diagnostic だけ `None` にしていた。
- 既存の `resource_effect_gate_keeps_unsafe_memory_shadow_only` unit test は、この問題を仕様として固定しており、Stage 5 public escape diagnostics の方針と矛盾していた。
- `RawAddressEscapeFromInternalAlloc` は既に error 化済みだったため、raw identity return だけ拒否し、pure 関数内の raw write/load は shadow-only に残る非対称な境界になっていた。

## 問題

Resource IR emits UnsafeMemoryInPureFunction for raw memory helper calls in pure functions, but compiler mapping returns None so the diagnostic is shadow-only outside raw-memory boundary.

## 影響

User source can call pure-signature raw memory helpers and bypass the effect boundary even though Resource IR detects UnsafeMemory, weakening effect safety and raw memory discipline.

## 修正方針

Add a ResourceRaw diagnostic enum for unsafe memory boundary violations and map UnsafeMemoryInPureFunction to a compiler error unless SourceMap marks the file as a raw-memory boundary.

## 対応結果

- `ResourceRawDiagnosticCode::UnsafeMemoryBoundary` を追加し、stable code `resource.raw.unsafe_memory_boundary` と message を定義した。
- `ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction` を compiler diagnostic へ昇格し、pure user function から raw memory operation を呼ぶ経路を `resource.raw.unsafe_memory_boundary` として拒否するようにした。
- compiler-owned raw memory boundary は `resource_effect_boundary_diagnostic_is_raw_boundary_allowed` の SourceMap capability 判定を維持し、stdlib/compiler 内部の raw memory wrapper 実装だけを境界内に残した。
- `move_effect.n.md` の raw memory positive tests は、raw ownership/cell semantics の検証であり pure surface の検証ではないため、raw operation を行う関数を impure signature に更新した。

## 検証

- `cargo test -p nepl-core compiler::tests::resource_effect_gate -- --nocapture`: pass
- `cargo fmt --check -p nepl-core`: pass
- `cargo check -p nepl-core --tests`: pass
- `trunk build`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 1 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 2 --dist web/dist`: pass
- `node nodesrc/run_doctest.js -i tests/compiler/move_effect.n.md -n 3 --dist web/dist`: pass
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-unsafe-agent1.json -j 1 --dist web/dist`: total=111, passed=111

## 関連 issue

- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`: raw memory effect / ownership boundary の親 issue。

## 関連ドキュメント

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
