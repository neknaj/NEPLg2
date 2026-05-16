---
id: ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190
title: "LLVM codegen responsibility freeze regressed on main"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/codegen_llvm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md"
---

# ISS-20260516T065711051Z-LLVM-CODEGEN-RESPONSIBILITY-FREEZE-R-0530B190: LLVM codegen responsibility freeze regressed on main

## 概要

After syncing remote main during ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59, node nodesrc/test_parser_backend_responsibility_policy.js reports nepl-core/src/codegen_llvm.rs at 4217 lines while the responsibility freeze limit is 4189. This is independent from the WASM string-data/aggregate split.

## 対象

- `nepl-core/src/codegen_llvm.rs, nodesrc/test_parser_backend_responsibility_policy.js, doc/neplg2/parser_backend_responsibility_split_plan.md`

## 根拠

- `doc/neplg2/parser_backend_responsibility_split_plan.md` の B3 は、LLVM backend を module text assembly、function lowering、SSA value/local mapping、aggregate lowering、raw body bridge へ分ける方針である。
- `nepl-core/src/codegen_llvm.rs` は HIR type から LLVM type への写像と aggregate field selector 解決まで root に持っており、root file の責務境界を狭める余地があった。

## 問題

After syncing remote main during ISS-20260516T061424173Z-WASM-CODEGEN-RESPONSIBILITY-FREEZE-R-2705FB59, node nodesrc/test_parser_backend_responsibility_policy.js reports nepl-core/src/codegen_llvm.rs at 4217 lines while the responsibility freeze limit is 4189. This is independent from the WASM string-data/aggregate split.

## 影響

The parser/backend source-policy runner remains red after the WASM backend split, so unrelated compiler work can keep carrying an architecture warning and LLVM backend responsibilities can continue accumulating in the root file.

## 修正方針

Follow doc/neplg2/parser_backend_responsibility_split_plan.md B3. Move a coherent LLVM backend responsibility such as raw body handling, aggregate lowering, or value/local mapping into a dedicated module, then lower or keep the root line limit instead of raising it.

## 対応結果

- `nepl-core/src/codegen_llvm/type_map.rs` を追加し、HIR `TypeId` から LLVM scalar/value type への写像を分離した。
- `nepl-core/src/codegen_llvm/aggregate.rs` を追加し、tuple index / struct field name による aggregate field layout 解決を分離した。
- `nodesrc/test_parser_backend_responsibility_policy.js` は `codegen_llvm.rs` の line freeze を 4188 行へ下げ、新 module の存在と line budget を監視する。

## 検証

Run node nodesrc/test_parser_backend_responsibility_policy.js, focused LLVM codegen tests, node nodesrc/run_source_policy_regressions.js, node nodesrc/issues.js check --dir issues, and git diff --check.

確認結果:

- `cargo check -p nepl-core`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo test -p nepl-core codegen_llvm::tests -- --nocapture`: 9 passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 10 passed
- `node nodesrc/test_parser_backend_responsibility_policy.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
- LLVM responsibility line check: `codegen_llvm.rs` 4184/4188, `type_map.rs` 23/40, `aggregate.rs` 26/40
- `node nodesrc/run_source_policy_regressions.js`: `ISS-20260516T061152439Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-B1897B0F` の既存 doctest baseline 超過で失敗。
