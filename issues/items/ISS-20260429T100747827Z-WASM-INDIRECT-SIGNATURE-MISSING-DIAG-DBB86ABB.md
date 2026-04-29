---
id: ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB
title: "Wasm indirect signature missing diagnostic is unreachable from current precheck"
area: core
status: open
resolved: false
priority: P2
type: bug
created: 2026-04-29
updated: 2026-04-29
target: "nepl-core/src/wasm_shared.rs, nepl-core/src/passes/codegen_precheck.rs"
---

# ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB: Wasm indirect signature missing diagnostic is unreachable from current precheck

## 概要

collect_wasm_signature_set inserts every supported indirect call signature before codegen_precheck checks whether the same signature exists, so WasmDiagnosticCode::IndirectSignatureMissing appears unreachable through the public precheck path.

## 対象

- `nepl-core/src/wasm_shared.rs, nepl-core/src/passes/codegen_precheck.rs`

## 根拠

- `nepl-core/src/wasm_shared.rs` の `collect_wasm_signature_set` は `collect_indirect_sigs` で HIR 内の supported indirect call signature を収集し、そのまま signature set に追加している。
- `nepl-core/src/passes/codegen_precheck.rs` の `check_indirect_sig_expr` は同じ set に call-site signature が含まれるかを見て `WasmDiagnosticCode::IndirectSignatureMissing` を出すが、supported signature は前段で set に入るため、公開 `precheck_wasm_codegen` 経路からは missing として観測できない。
- 診断の enum contract は [NEPLg2 compiler diagnostic redesign plan](../../doc/neplg2/compiler_diagnostics_redesign_plan.md) で管理するため、dead branch のまま残すかどうかも診断設計として明示的に判断する必要がある。

## 問題

collect_wasm_signature_set inserts every supported indirect call signature before codegen_precheck checks whether the same signature exists, so WasmDiagnosticCode::IndirectSignatureMissing appears unreachable through the public precheck path.

## 影響

A diagnostic variant exists for missing indirect table signatures, but the current signature-set construction cannot detect that condition separately from supported signature encodability. This weakens backend precheck coverage and can leave dead diagnostic branches.

## 修正方針

Redesign wasm indirect signature precheck so the expected table/function signature source and the call-site signature request are separate. Keep IndirectSignatureUnsupported for non-encodable signatures and make IndirectSignatureMissing cover a reachable missing-table-signature condition, or remove the variant if the invariant is intentionally impossible.

## 検証

Add a focused precheck regression that reaches IndirectSignatureMissing after the signature source is separated, plus keep unsupported-signature regression.
