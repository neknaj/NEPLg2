---
id: ISS-20260429T100747827Z-WASM-INDIRECT-SIGNATURE-MISSING-DIAG-DBB86ABB
title: "Wasm indirect signature missing diagnostic is unreachable from current precheck"
area: core
status: fixed
resolved: true
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

## 対応結果

`collect_wasm_signature_set` から call-site の `CallIndirect` 要求 signature を収集して type section に混ぜる処理を削除した。Wasm signature set は extern と到達可能 function の実在 signature だけを source of truth とし、`codegen_precheck` / wasm codegen の `CallIndirect` 側が要求する signature はその set に存在するかを検査する。

これにより、表現不能な signature は `WasmDiagnosticCode::IndirectSignatureUnsupported`、表現可能だが実在 function / extern signature に存在しない indirect call signature は `WasmDiagnosticCode::IndirectSignatureMissing` として分離される。

`nepl-core/tests/codegen_diagnostics.rs` に、precheck と wasm codegen の両方で `IndirectSignatureMissing` が到達する回帰テストを追加した。

## 2026-04-29 検証

- `cargo test -p nepl-core --test codegen_diagnostics`: pass, 10 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/codegen_diagnostics.n.md --no-tree -o tmp/wasm-indirect-codegen-diagnostics.json -j 1 --dist web/dist`: total=3, passed=3, failed=0
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass (CRLF warning only)

補足: `node nodesrc/tests.js -i tests/compiler/functions.n.md --no-tree -o tmp/wasm-indirect-signature-functions.json -j 1 --dist web/dist` は total=24, passed=23, failed=1。失敗は既知の `ISS-20260429T064519915Z-STDIO-PRINT-I32-TRIGGERS-RAWMEMORYLO-B90E5FA7` の `stdio print_i32` RawMemoryLoadCell ownership violation で、今回の indirect signature set 分離とは別問題。
