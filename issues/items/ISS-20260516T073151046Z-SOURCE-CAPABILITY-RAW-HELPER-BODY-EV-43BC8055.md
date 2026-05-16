---
id: ISS-20260516T073151046Z-SOURCE-CAPABILITY-RAW-HELPER-BODY-EV-43BC8055
title: "source capability raw helper body evidence leaks across nested function scopes"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-16
updated: 2026-05-16
target: "nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/walk.rs, nepl-core/src/loader.rs"
---

# ISS-20260516T073151046Z-SOURCE-CAPABILITY-RAW-HELPER-BODY-EV-43BC8055: source capability raw helper body evidence leaks across nested function scopes

## 概要

`SourceCapabilityProofCollector` が raw memory evidence を active な全 function frame に記録していたため、nested function body の raw evidence が外側関数の body evidence として扱われていた。

## 対象

- `nepl-core/src/source_capability/proof.rs, nepl-core/src/source_capability/walk.rs, nepl-core/src/loader.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 source capability unified proof。
- `SourceCapabilityProofCollector::record_raw_memory_evidence` が修正前は `function_has_raw_memory_evidence` の全 frame を更新していた。
- `raw_memory_boundary_keeps_raw_helper_body_evidence_function_scoped` regression で、外側 `alloc_raw` の中に raw load を呼ぶ nested helper があるだけでは `RawMemoryOp::Alloc` capability を得ないことを固定した。

## 問題

raw helper definition evidence は「関数名が raw helper 名であり、その関数自身の body に raw evidence がある」場合だけ self-operation capability を付ける必要がある。修正前は nested body の evidence まで外側 function frame に伝播していたため、外側関数が `alloc_raw` / `load_i32` などの名前を持つだけで、その body が対象 operation を実行していなくても raw operation capability が過大付与され得た。

## 影響

compiler-owned source capability が、source AST から実際に証明できる性質より広くなる。Stage 6 の raw-memory proof model が弱まり、nested local function を持つ stdlib/internal helper で静的検査実装の誤りを隠す。

## 修正方針

raw-memory body evidence は現在 walk 中の function frame だけへ記録する。module-level operation evidence は global に保持するが、raw helper definition の self-operation evidence はその関数自身の body にだけ結び付け、nested function definition から外側関数へ伝播させない。

## 検証

- `cargo test -p nepl-core loader::tests::raw_memory_boundary_keeps_raw_helper_body_evidence_function_scoped -- --exact --nocapture`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo test -p nepl-core loader::tests::raw_memory_boundary -- --nocapture`: passed
- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core`: passed
- `node nodesrc/run_source_policy_regressions.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `git diff --check`: passed
