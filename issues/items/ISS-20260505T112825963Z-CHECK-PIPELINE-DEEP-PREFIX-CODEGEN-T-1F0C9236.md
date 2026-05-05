---
id: ISS-20260505T112825963Z-CHECK-PIPELINE-DEEP-PREFIX-CODEGEN-T-1F0C9236
title: "check_pipeline deep prefix codegen tests exceed local test budget"
area: core
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/compiler.rs, nepl-core/src/resource/gate_demand.rs, nepl-core/src/resource/mod.rs, nepl-core/tests/check_pipeline.rs"
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

## 対応

- deep prefix source を小さい入力で phase profiling し、`prepare_module_for_codegen` の支配時間が wasm emit ではなく Resource IR initialized / effect / owner gate にあることを確認した。
- Resource IR lowering coverage は必ず実行した上で、後続の高コスト安全 gate を走らせる必要があるかを `ResourceSafetyGateDemand` enum で分類する設計にした。
- 分類ロジックを `nepl-core/src/resource/gate_demand.rs` に分離し、compiler は enum を `match` して `ResourceNeutral` の場合だけ initialized / borrow / effect / owner gate を省略する。
- `i32` だけの pure identity call chain、`Block` / `Call` marker、primitive-only `raw_address_alias` は resource safety obligation を持たないため中立と判定する。一方で raw memory op、borrow/move/drop、projection、reference、`MemPtr` / `RegionToken`、impure effect、user aggregate は必ず `RequiresResourceSafetyGates` へ倒す。
- `gate_demand_keeps_primitive_identity_calls_neutral` と `gate_demand_requires_raw_memory_gates` を追加し、primitive-only の高速化と raw memory gate 維持を両方固定した。
- `doc/neplg2/static_check_complexity_reduction_plan.md` では Stage 4 resource check / Stage 5 effect model に該当する。Resource IR lowering coverage を残し、検査対象が存在する場合は従来通り各 gate を実行するため、静的検査の正確性を弱めていない。

## 検証

- `cargo test -p nepl-core --lib gate_demand -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test check_pipeline -- --nocapture`: 8 passed / 約9秒
- `cargo test -p nepl-core --test check_pipeline prepare_codegen_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: pass / 約6.7秒
- `cargo test -p nepl-core --test check_pipeline compile_wasm_accepts_deep_prefix_chain_without_codegen_stack_overflow -- --nocapture`: pass / 約6.9秒
- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check_reports_raw_alloc_escape_through_identity_call -- --nocapture`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_raw_fill -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_region_ptr_rewrap_view_dealloc -- --nocapture`: pass
- `cargo test -p nepl-core --test effects pure_indirect_impure_function_value_is_rejected -- --nocapture`: pass
- `cargo fmt --check -p nepl-core`: pass
- `cargo check -p nepl-core --tests`: pass
