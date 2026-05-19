---
id: ISS-20260519T054544598Z-RESOURCE-CHECKER-SOURCE-POLICY-DOES--BB3DC378
title: "Resource checker source policy misses new Resource IR modules and responsibility splits"
area: CORE
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-19
updated: 2026-05-19
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/*"
---

# ISS-20260519T054544598Z-RESOURCE-CHECKER-SOURCE-POLICY-DOES--BB3DC378: Resource checker source policy misses new Resource IR modules and responsibility splits

## 概要

Resource IR の module 追加後に `nodesrc/test_resource_checker_responsibility.js` の監視対象が追従しておらず、`initialized_alias_offset.rs` などの静的検査実装が line-limit / required-symbol policy の外で成長できる状態になっていた。

## 対象

- `nodesrc/test_resource_checker_responsibility.js`
- `nepl-core/src/resource/initialized_alias_offset.rs`
- `nepl-core/src/resource/initialized_scalar_flow.rs`
- `nepl-core/src/resource/initialized_str_layout.rs`
- `nepl-core/src/resource/cell_state_raw_range_cover.rs`
- `nepl-core/src/resource/coverage_hir.rs`
- `nepl-core/src/resource/lower_raw_address_source.rs`
- `nepl-core/src/resource/i32_call_facts.rs`
- `nepl-core/src/resource/initialized_alias_origin.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` が `initialized_alias_offset.rs must be monitored by resource responsibility line limits` を報告した。
- missing module を補うと、`initialized_scalar_flow.rs` と `initialized_str_layout.rs` も監視対象から漏れていた。
- 監視漏れが先に失敗していたため、既存の line budget 超過も隠れていた。特に HIR coverage の transparent raw-address 証明、raw range offset normalization、i32 call fact tests、i32 scalar flow propagation、raw-address offset model、raw value origin tests が単一 file に残っていた。

## 問題

Resource IR は静的検査の中核であり、module が responsibility policy の外に出ると、実装肥大化や個別証明の再集中を source policy で検出できなくなる。これにより静的検査実装自体の誤りを発見しやすくする仕組みが弱くなる。

## 影響

Resource IR initialized alias / scalar / raw-address proof logic can grow outside the responsibility budget, weakening the static-check implementation guardrail and making memory-safety proof regressions harder to spot during review.

## 修正方針

全 Resource IR module が responsibility line-limit monitor に入るようにし、漏れた module を単に登録するだけでなく、責務が再集中していた箇所を自然な単位へ分割する。

- raw range coverage から normalized raw offset model を分離する。
- HIR coverage から transparent raw-address return coverage proof を分離する。
- i32 call facts から unit tests を分離し、実装本体の budget を実装だけに効かせる。
- i32 scalar return summary と scalar propagation ops を分離する。
- raw-address source から offset model / arithmetic を分離する。
- raw value origin 実装から unit tests を分離する。
- `initialized_alias_offset.rs` / `initialized_scalar_flow.rs` / `initialized_str_layout.rs` を required-symbol と line-limit policy に登録する。

## 検証

- `cargo fmt -p nepl-core -- --check`
- `cargo check -p nepl-core`
- `cargo test -p nepl-core records_i32_offset_for_symbolic_add_even_when_source_value_is_known --lib -- --nocapture`
- `cargo test -p nepl-core copy_stable_origin_follows_temporary_source_origin --lib -- --nocapture`
- `cargo test -p nepl-core resource_ir_lowering_preserves_transparent_region_ptr_wrapper --test resource_ir -- --nocapture`
- `cargo test -p nepl-core fd_readdir --test resource_ir -- --nocapture`
- `node nodesrc/test_resource_checker_responsibility.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`

## 修正内容

- `nodesrc/test_resource_checker_responsibility.js` に漏れていた Resource IR module と required-symbol checks を追加した。
- `cell_state_raw_range_offset.rs` を追加し、raw byte range coverage の offset normalization を独立させた。
- `coverage_hir_transparent.rs` を追加し、transparent raw-address return coverage proof を HIR coverage traversal から分離した。
- `i32_call_facts_tests.rs` と `initialized_alias_origin_tests.rs` を追加し、unit test を実装本体から分離した。
- `initialized_scalar_flow_ops.rs` を追加し、i32 scalar summary collection と propagation operation walk を分離した。
- `lower_raw_address_offset.rs` を追加し、raw-address offset model / arithmetic を source construction から分離した。

## 検証結果

- `cargo fmt -p nepl-core -- --check`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core records_i32_offset_for_symbolic_add_even_when_source_value_is_known --lib -- --nocapture`: passed
- `cargo test -p nepl-core copy_stable_origin_follows_temporary_source_origin --lib -- --nocapture`: passed
- `cargo test -p nepl-core resource_ir_lowering_preserves_transparent_region_ptr_wrapper --test resource_ir -- --nocapture`: passed
- `cargo test -p nepl-core fd_readdir --test resource_ir -- --nocapture`: 3 tests passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
