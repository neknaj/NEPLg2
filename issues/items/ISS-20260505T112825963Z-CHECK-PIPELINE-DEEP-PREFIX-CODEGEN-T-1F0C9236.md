---
id: ISS-20260505T112825963Z-CHECK-PIPELINE-DEEP-PREFIX-CODEGEN-T-1F0C9236
title: "check_pipeline deep prefix codegen tests exceed local test budget"
area: core
status: open
resolved: false
priority: P2
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/tests/check_pipeline.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/compiler.rs"
---

# ISS-20260505T112825963Z-CHECK-PIPELINE-DEEP-PREFIX-CODEGEN-T-1F0C9236: check_pipeline deep prefix codegen tests exceed local test budget

## 概要

check_pipeline の deep prefix 系 regression は check / drop / monomorphize / move_check が数秒で通る一方、prepare_codegen と compile_wasm が 180 秒以上完了しない。深い prefix AST に対する wasm codegen 側の計算量、出力 wasm 構築、またはテスト fixture 粒度のどれが支配的か未切り分けのまま全体テストのシグナルを壊している。

## 対象

- `nepl-core/tests/check_pipeline.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/compiler.rs`

## 根拠

- `cargo test -p nepl-core --test check_pipeline -- --nocapture` は 360 秒 timeout で完了しなかった。
- 同じ deep prefix source でも `check_module_accepts_deep_prefix_chain_without_codegen_stack_overflow` / `drop_insertion_accepts_deep_prefix_chain_without_stack_overflow` / `monomorphize_accepts_deep_prefix_chain_without_stack_overflow` / `move_check_accepts_deep_prefix_chain_without_stack_overflow` はそれぞれ数秒で pass した。
- 一方、`prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow` は 180 秒 timeout、`compile_wasm_accepts_deep_prefix_chain_without_codegen_stack_overflow` も 180 秒 timeout した。
- この差分から、少なくとも同 fixture では parse/typecheck/drop/monomorphize/move_check より後の prepare/codegen 経路が支配的である可能性が高い。ただし、compiler 実装の計算量問題か fixture の目的に対して過大な入力かは未確定である。

## 問題

check_pipeline の deep prefix 系 regression は check / drop / monomorphize / move_check が数秒で通る一方、prepare_codegen と compile_wasm が 180 秒以上完了しない。深い prefix AST に対する wasm codegen 側の計算量、出力 wasm 構築、またはテスト fixture 粒度のどれが支配的か未切り分けのまま全体テストのシグナルを壊している。

## 影響

compiler の root-cause 性能問題と単に重すぎる regression fixture が区別できず、今回のような monomorphize 性能修正の検証でも check_pipeline 全体を使えない。

## 修正方針

prepare_codegen/compile_wasm を phase profiling し、HIR lowering・Resource IR・wasm emit のどこで深い prefix chain が線形以上になっているかを特定する。本質的な compiler 計算量なら実装修正し、fixture が過大なら目的別に分割して通常 CI 予算内へ戻す。

## 検証

各 phase の所要時間を記録し、check_pipeline 全体が通常のローカル検証時間内に完了することを確認する。
