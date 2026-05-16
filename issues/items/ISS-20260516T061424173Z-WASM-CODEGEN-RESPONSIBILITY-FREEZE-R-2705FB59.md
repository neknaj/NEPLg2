---
id: ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59
title: "WASM codegen responsibility freeze regressed on main"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md"
---

# ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59: WASM codegen responsibility freeze regressed on main

## 概要

origin/main currently has nepl-core/src/codegen_wasm.rs at 2582 lines while nodesrc/test_parser_backend_responsibility_policy.js freezes the limit at 2574. This is not caused by the BTreeMap ResourceIR branch; current, HEAD, and origin/main all report the same 2582-line count.

## 対象

- `nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- `doc/neplg2/parser_backend_responsibility_split_plan.md` の B2 は、WASM backend を section assembly、function lowering、instruction emission、aggregate lowering、call loweringへ分ける方針である。
- `nepl-core/src/codegen_wasm.rs` は string literal の static data layout と aggregate field selector 解決まで root に持っていたため、小さな増分でも responsibility freeze を超えた。

## 問題

origin/main currently has nepl-core/src/codegen_wasm.rs at 2582 lines while nodesrc/test_parser_backend_responsibility_policy.js freezes the limit at 2574. This is not caused by the BTreeMap ResourceIR branch; current, HEAD, and origin/main all report the same 2582-line count.

## 影響

The parser/backend source-policy runner warns even in unrelated compiler work, and the WASM backend continues to accumulate responsibilities in a large root file instead of moving instruction emission, aggregate lowering, match lowering, raw body handling, or helper lowering into planned submodules.

## 修正方針

Follow doc/neplg2/parser_backend_responsibility_split_plan.md B2 rather than raising the line limit. Move a coherent WASM backend responsibility into a dedicated module and lower or keep the root limit.

## 対応結果

- `nepl-core/src/codegen_wasm/string_data.rs` を追加し、string literal の offset、data segment、heap base、minimum memory page 算出を分離した。
- `nepl-core/src/codegen_wasm/aggregate.rs` を追加し、tuple index / struct field name による aggregate field layout 解決を分離した。
- `nodesrc/test_parser_backend_responsibility_policy.js` は `codegen_wasm.rs` の line freeze を 2525 行へ下げ、新 module の存在と line budget を監視する。
- 同じ policy 実行中に `codegen_llvm.rs` の既存 freeze 超過が露出したため、別 issue `ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190` として分離した。

## 検証

Run node nodesrc/test_parser_backend_responsibility_policy.js, relevant WASM codegen tests, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check --dir issues, and git diff --check.

確認結果:

- `cargo check -p nepl-core`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 10 passed
- `cargo test -p nepl-core --test layout -- --nocapture`: 4 passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
- WASM responsibility line check: `codegen_wasm.rs` 2519/2525, `string_data.rs` 66/80, `aggregate.rs` 26/40
- `node nodesrc/test_parser_backend_responsibility_policy.js`: `codegen_llvm.rs` の既存 freeze 超過で失敗。別 issue `ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190` に分離済み。
- `node nodesrc/run_source_policy_regressions.js`: `ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F` の既存 doctest baseline 超過で失敗。
