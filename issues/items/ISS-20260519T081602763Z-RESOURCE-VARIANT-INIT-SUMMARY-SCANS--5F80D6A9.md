---
id: ISS-20260519T081602763Z-RESOURCE-VARIANT-INIT-SUMMARY-SCANS--5F80D6A9
title: "Resource variant init summary scans non-enum return functions"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-19
updated: 2026-05-19
target: nepl-core/src/resource/initialized_summary_variant_build.rs
---

# ISS-20260519T081602763Z-RESOURCE-VARIANT-INIT-SUMMARY-SCANS--5F80D6A9: Resource variant init summary scans non-enum return functions

## 概要

Resource IR raw initialization summary builds variant-param facts by replaying every returning function body even when the return type is provably not an enum or enum application. Large NM functions returning str or StringBuilder therefore pay an unnecessary full ResourceCheckEngine replay and branch scan while producing zero variant facts.

## 対象

- `nepl-core/src/resource/initialized_summary_variant_build.rs`

## 根拠

- `NEPL_RESOURCE_SUMMARY_TRACE=1` を使った probe で、`nm_inline_to_json_into` / `document_to_json` / `nm_inline_to_html` / `nm_render_source_html` が variant-param summary 構築に合計約 12 秒を費やしていた。
- これらの関数の戻り値は `str` または `StringBuilder` であり、enum variant payload ごとの param initialization facts は型上生成できない。
- 同じ probe で `variants=0` / `variant_reqs=0` だったため、走査は結果を生まず、release summary bottleneck を覆い隠すだけだった。

## 問題

Resource IR raw initialization summary builds variant-param facts by replaying every returning function body even when the return type is provably not an enum or enum application. Large NM functions returning str or StringBuilder therefore pay an unnecessary full ResourceCheckEngine replay and branch scan while producing zero variant facts.

## 影響

examples/nm.nepl spends seconds in the variant half of resource_initialized_raw_init_summaries for functions where variant summaries are impossible. This inflates the Stage 6 Resource IR performance issue and obscures the remaining release-summary bottleneck.

## 修正方針

Add a TypeCtx based return-type gate for variant-param summary collection. Only enum returns, enum applications, and unresolved/unknown nominal cases may run the variant replay; concrete non-enum returns must return immediately. Keep this generic and type-driven, not stdlib/module allowlisting.

2026-05-19 に対応済み。`TypeKind` の網羅的 `match` を `initialized_summary_variant_type.rs` に分離し、`Enum` / enum `Apply` / 未解決型だけが variant summary replay へ進むようにした。`str`、struct、struct application、tuple、function、reference などの concrete non-enum return は body replay 前に戻る。

## 検証

cargo test -p nepl-core resource::initialized_summary_variant_build_tests -- --nocapture; node nodesrc/issues.js check; node nodesrc/run_source_policy_regressions.js --warn-only

- `cargo test -p nepl-core resource::initialized_summary_variant_build_tests -- --nocapture`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed
- `NEPL_COMPILE_STAGE_TIMING=1 cargo run -p nepl-cli -- --target wasi --profile debug --input examples/nm.nepl --output tmp/agent1-nm-variant-gate`: 300 秒 probe で `resource_initialized_raw_init_summaries=76306ms`、`resource_initialized_moves=95503ms` まで確認。full compile はまだ `resource_effect_boundaries` 以降が残るため `ISS-20260519T074504799Z-NM-FULL-COMPILE-STILL-EXCEEDS-CI-BUD-5653B487` を継続する。
