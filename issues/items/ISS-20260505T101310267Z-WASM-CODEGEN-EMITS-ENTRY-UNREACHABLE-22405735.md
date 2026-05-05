---
id: ISS-20260505T101310267Z-WASM-CODEGEN-EMITS-ENTRY-UNREACHABLE-22405735
title: "wasm codegen emits entry-unreachable functions after reachability precheck"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/codegen_wasm.rs, nepl-core/src/wasm_shared.rs, nepl-core/tests/codegen_diagnostics.rs"
---

# ISS-20260505T101310267Z-WASM-CODEGEN-EMITS-ENTRY-UNREACHABLE-22405735: wasm codegen emits entry-unreachable functions after reachability precheck

## 概要

precheck_wasm_codegen already computes the entry-reachable function set, but generate_wasm still iterates every monomorphized HIR function. Entry-unreachable imports or selfhost helper bodies can therefore produce wasm backend diagnostics or consume codegen time even though the public entry cannot reach them.

## 対象

- `nepl-core/src/codegen_wasm.rs, nepl-core/src/wasm_shared.rs, nepl-core/tests/codegen_diagnostics.rs`

## 根拠

- `nepl-core/src/passes/codegen_precheck.rs` は `collect_reachable_wasm_functions` によって entry 到達関数だけを precheck 対象にしている。
- 一方、修正前の `nepl-core/src/codegen_wasm.rs` は `module.functions` 全体を走査して `FuncLower::user` を作っていたため、precheck で到達不能として除外された関数も実 emit で unsupported signature / unsupported body diagnostic や生成時間の対象になっていた。
- selfhost CLI driver doctest の timeout 調査中、entry からの到達境界と実 codegen の境界が一致していないと、性能原因を「到達関数集合」と「backend lowering cost」に切り分けられないことを確認した。

## 問題

precheck_wasm_codegen already computes the entry-reachable function set, but generate_wasm still iterates every monomorphized HIR function. Entry-unreachable imports or selfhost helper bodies can therefore produce wasm backend diagnostics or consume codegen time even though the public entry cannot reach them.

## 影響

The wasm backend can fail or time out on functions that are outside the entry graph, masking real static-check/codegen failures and making selfhost doctest performance harder to reason about.

## 修正方針

Use the same typed reachability boundary for actual wasm function lowering. Keep compiler runtime helpers as explicit roots because aggregate lowering may call them without a source-level call expression.

## 検証

Add a regression where an unreachable function has an unsupported wasm signature and verify precheck plus generate_wasm ignore it, while reachable functions and runtime helper roots remain covered.

## 修正内容

- `collect_reachable_wasm_functions` の root に `__nepl_rt_alloc` / `__nepl_rt_dealloc` / `__nepl_rt_realloc` 系の runtime helper を明示的に加えた。aggregate lowering は source-level call を持たずに allocator helper を参照するため、entry root だけで helper を削ると逆に codegen が壊れる。
- `generate_wasm` の user function lowering を entry-reachable set に限定し、precheck と実 emit の関数境界を一致させた。
- 到達不能な bad function は precheck でも generate_wasm でも無視される regression を追加した。

## 完了条件

- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_ignores_entry_unreachable_bad_function -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics wasm_codegen_reports_unsupported_function_signature_without_panicking -- --nocapture`: passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 11 passed
- `cargo check -p nepl-core --tests`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test generics generics_struct_pair_construction -- --nocapture`: passed

補足: `cargo test -p nepl-core --test check_pipeline compile_wasm_accepts_deep_prefix_chain_without_codegen_stack_overflow -- --nocapture` はローカル 124 秒 timeout。今回の差分に直接関係する regression は上記の focused codegen test と aggregate compile test で確認した。
