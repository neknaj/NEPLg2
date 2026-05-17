---
id: ISS-20260517T124911215Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-49B83608
title: "WASM codegen responsibility freeze regressed after backend enum work"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-17
updated: 2026-05-17
target: "nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js"
---

# ISS-20260517T124911215Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-49B83608: WASM codegen responsibility freeze regressed after backend enum work

## 概要

nodesrc/run_source_policy_regressions.js now stops in nodesrc/test_parser_backend_responsibility_policy.js because nepl-core/src/codegen_wasm.rs has 2585 lines while the responsibility freeze limit is 2525. The old resolved WASM split issue is no longer sufficient after later backend enum and intrinsic changes grew the root module again.

## 対象

- `nepl-core/src/codegen_wasm.rs, nodesrc/test_parser_backend_responsibility_policy.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js` が `nodesrc/test_parser_backend_responsibility_policy.js` で停止し、`nepl-core/src/codegen_wasm.rs has 2585 lines; responsibility freeze limit is 2525` を報告した。
- `nepl-core/src/codegen_wasm.rs` には WASM local slot 割り当て、block scope stack、temporary local allocation、alloc helper index の状態管理が root backend lowering と同居していた。
- 既存の string data / aggregate split だけでは、後続の backend enum / intrinsic work による root 再肥大化を抑えられていなかった。

## 問題

nodesrc/run_source_policy_regressions.js now stops in nodesrc/test_parser_backend_responsibility_policy.js because nepl-core/src/codegen_wasm.rs has 2585 lines while the responsibility freeze limit is 2525. The old resolved WASM split issue is no longer sufficient after later backend enum and intrinsic changes grew the root module again.

## 影響

The source-policy runner no longer reaches later static-check policies, and WASM backend responsibilities can keep accumulating in the root module instead of staying in small auditable units.

## 修正方針

Do not raise the freeze. Identify a coherent WASM backend responsibility still embedded in codegen_wasm.rs, move it to a dedicated module, wire it through exhaustive typed interfaces, and keep the root under the existing 2525-line limit.

## 検証

Run node nodesrc/test_parser_backend_responsibility_policy.js, node nodesrc/run_source_policy_regressions.js or the affected subset, cargo/rustfmt checks for touched Rust files, issues check, and diff check.

## 対応内容

- `nepl-core/src/codegen_wasm/local_map.rs` を追加し、`LocalMap` の状態と操作を専用 module に分離した。
- root backend は `valtype(...)` で WASM representation を決め、その `Option<ValType>` を `LocalMap` に渡す形にした。これにより `LocalMap` は HIR / TypeCtx へ依存せず、WASM local management のみを担当する。
- `alloc_helper_idx` は public field access ではなく `set_alloc_helper_idx` / `alloc_helper_idx` の小さな API に閉じた。
- `nodesrc/test_parser_backend_responsibility_policy.js` に `local_map.rs` の存在と 120 行上限を追加し、root file への責務再混入を監視するようにした。
- `doc/neplg2/parser_backend_responsibility_split_plan.md` の B2 進捗と baseline を更新した。

## 検証結果

- `rustfmt --check nepl-core\src\codegen_wasm.rs nepl-core\src\codegen_wasm\local_map.rs`: pass
- `cargo check -p nepl-core`: pass
- `node nodesrc/test_parser_backend_responsibility_policy.js`: pass
- `node nodesrc/run_source_policy_regressions.js`: pass
- `git diff --check`: pass
