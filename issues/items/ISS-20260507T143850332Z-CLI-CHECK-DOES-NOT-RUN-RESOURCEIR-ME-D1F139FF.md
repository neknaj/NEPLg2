---
id: ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF
title: "CLI --check does not run ResourceIR memory-safety gates"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/compiler.rs, nepl-core/tests/check_pipeline.rs, nepl-cli/src/main.rs"
---

# ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF: CLI --check does not run ResourceIR memory-safety gates

## 概要

nepl-cli --check calls check_module_with_source_map, and that API currently stops after target/profile precheck and run_typecheck. It does not run monomorphize, ResourceIR lowering coverage, initialized cell, drop plan, borrow, effect, owner, or drop bridge gates. A program can therefore pass check-only validation while the compile/codegen pipeline would reject it for memory-safety or resource-safety diagnostics.

## 対象

- `nepl-core/src/compiler.rs, nepl-core/tests/check_pipeline.rs, nepl-cli/src/main.rs`

## 根拠

- `nepl-cli/src/main.rs` の `--check` 分岐は `check_module_with_source_map(module, Some(&source_map), options)` を呼ぶ。
- `nepl-core/src/compiler.rs` の `check_module_with_source_map` は `precheck_module_before_codegen` と `run_typecheck` の後に `Ok(())` を返す。
- 同じ `compiler.rs` の `prepare_module_for_codegen_with_source_map` は `run_typecheck` 後に monomorphize、`run_resource_static_check`、drop elaboration HIR bridge、`passes::insert_resource_drops` まで進む。
- `nodesrc/test_resource_gate_order.js` は compile preparation 側の ResourceIR gate 順序を固定しているが、check-only API が同じ gate を通ることは固定していない。

## 問題

nepl-cli --check calls check_module_with_source_map, and that API currently stops after target/profile precheck and run_typecheck. It does not run monomorphize, ResourceIR lowering coverage, initialized cell, drop plan, borrow, effect, owner, or drop bridge gates. A program can therefore pass check-only validation while the compile/codegen pipeline would reject it for memory-safety or resource-safety diagnostics.

## 影響

The project policy requires type safety and memory safety to be enforced by static checks. If --check is used by users, CI, or future selfhost tooling as the verification command, it can return success without exercising the ResourceIR authority that now owns cell/owner/borrow/effect safety. This hides real regressions and can pressure later work toward weakening compile-time gates instead of fixing the check pipeline boundary.

## 修正方針

Redesign check_module_with_source_map so check-only validation runs the same static-safety authority as compile preparation without emitting artifacts. Reuse the non-recursive prepare pipeline or factor a shared prepare/check phase that performs target precheck, typecheck, resource monomorphize, ResourceIR static checks, drop elaboration HIR bridge validation, and diagnostic reporting, while still avoiding artifact codegen and the old stack-overflow path.

## 検証

Add a CLI --check regression where source compiles through parser/typecheck but fails ResourceIR owner/cell/effect validation, and assert --check returns diagnostics. Keep the deep-HIR stack-overflow regression to prove the shared check phase remains non-recursive and does not enter artifact emission.

## 対応結果

2026-05-07 に修正済み。

- `check_module_with_source_map` は typecheck で止まらず、`prepare_module_for_codegen_with_source_map` を共有して target/profile precheck、typecheck、monomorphize、Resource IR static gate、drop elaboration HIR bridge、resource drop insertion まで実行するようにした。
- 成果物 emission は引き続き行わないため、`--check` は codegen artifact を生成しない。一方で compile/codegen pipeline と同じ memory/resource safety authority を通る。
- `nepl-core/tests/check_pipeline.rs` に check-only API が Resource IR effect gate を通す regression を追加した。
- `nepl-cli/src/main.rs` に CLI `--check` が Resource IR diagnostic で失敗する regression を追加した。
- 既存の deep prefix regression を通し、check-only が artifact emission へ入らず stack overflow regression を再発させないことを確認した。

検証:

- `cargo test -p nepl-core --test check_pipeline check_module_ -- --nocapture`: passed
- `cargo test -p nepl-cli check_runs_resource_ir_static_safety_gates -- --nocapture`: passed
- `cargo test -p nepl-cli check_accepts_deep_prefix_chain_without_codegen_stack_overflow -- --nocapture`: passed
- `cargo test -p nepl-core --test effects raw_memory_boundary -- --nocapture`: passed
- `cargo test -p nepl-core --test effects loader_does_not_mark_user_core_mem_path_by_suffix -- --nocapture`: passed
