---
id: ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D
title: "Rust compiler diagnostics are not aligned with Resource IR and self-host model"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_ids.rs, nepl-core/src/compiler.rs, nepl-cli/src/main.rs, nodesrc/tests.js, stdlib/neplg2/core/infra/diag.nepl, doc/neplg2/compiler_diagnostics_redesign_plan.md"
---

# ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D: Rust compiler diagnostics are not aligned with Resource IR and self-host model

## 概要

Rust core diagnostics still rely on hand-maintained numeric IDs and free-form Diagnostic construction. Resource IR errors are forced into legacy IDs such as D3025/D3100/D3101, while the self-host compiler already uses stable string codes, labels, and notes. The two models are diverging.

## 対象

- `nepl-core/src/diagnostic.rs, nepl-core/src/diagnostic_ids.rs, nepl-core/src/compiler.rs, nepl-cli/src/main.rs, nodesrc/tests.js, stdlib/neplg2/core/infra/diag.nepl, doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 根拠

- `nepl-core/src/diagnostic_ids.rs` は数値 `DiagnosticId`、`from_u32`、`message()` を手で保守しており、Resource IR 追加後の cell / owner / borrow / raw effect の意味分類を表す stable string code を持たない。
- `nepl-core/src/diagnostic.rs` には `code` field があるが、多くの Rust core call site は `Diagnostic::error(...).with_id(...)` を直接使い、`code` を主識別子として扱っていない。
- `nepl-core/src/compiler.rs` の Resource IR gate は `ResourceCheckDiagnostic` / `ResourceOwnerDiagnostic` / `ResourceBorrowDiagnostic` / `ResourceEffectBoundaryDiagnostic` を `D3025` / `D3100` / `D3101` / 旧 move-check ID へ写像しており、Resource IR 側の意味分類が compiler diagnostic で粗くなる。
- `stdlib/neplg2/core/infra/diag.nepl` の self-host diagnostic は string `code`、message、primary label、note を中心にしており、Rust core の数値 ID 中心モデルと既に分岐している。
- `nodesrc/parser.js` / `nodesrc/tests.js` は `diag_id` / `diag_ids` を検査できるが、stable string diagnostic code を regression として固定する仕組みがない。

## 問題

Rust core diagnostics still rely on hand-maintained numeric IDs and free-form Diagnostic construction. Resource IR errors are forced into legacy IDs such as D3025/D3100/D3101, while the self-host compiler already uses stable string codes, labels, and notes. The two models are diverging.

## 影響

Static check gates can only be connected through ad-hoc ID mapping, regression tests pin accidental legacy buckets, and self-host parity cannot compare Rust and NEPL diagnostics without another translation layer.

## 修正方針

Introduce a diagnostics redesign plan: stable string diagnostic codes with legacy numeric compatibility, typed diagnostic kinds/builders per compiler stage, Resource IR diagnostic mapping through semantic categories, richer notes/help/related labels, and generated registry consistency checks.

詳細設計と実装段階は [NEPLg2 compiler diagnostic redesign plan](../../doc/neplg2/compiler_diagnostics_redesign_plan.md) に定義する。

この issue は `D3102` のような局所的な ID 追加を避けるための親 issue とする。既存 `diag_id` は互換層として残すが、新しい Resource IR / effect / borrow / owner diagnostic は stable `diag_code` を持つ方向へ移行する。

## 検証

Add registry consistency tests, CLI rendering compatibility tests, doctest support for diagnostic codes alongside legacy diag_id, and focused Resource IR diagnostic mapping regressions.
