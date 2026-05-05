---
id: ISS-20260505T105951378Z-MONOMORPHIZE-RESCANS-ALL-SPECIALIZED-20DC66B3
title: "Monomorphize rescans all specialized functions for trait calls"
area: core
status: fixed
resolved: true
priority: P1
type: performance
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/monomorphize.rs, nepl-core/tests"
---

# ISS-20260505T105951378Z-MONOMORPHIZE-RESCANS-ALL-SPECIALIZED-20DC66B3: Monomorphize rescans all specialized functions for trait calls

## 概要

monomorphize resolves remaining trait calls by repeatedly walking every specialized function after each worklist drain. Large selfhost graphs can add new specializations across many iterations, so previously resolved functions are rescanned even though trait impl resolution is static.

## 対象

- `nepl-core/src/monomorphize.rs, nepl-core/tests`

## 根拠

- `monomorphize_internal` は初期 worklist を drain した後、`resolve_remaining_trait_calls()` で `specialized` 全体を走査し、その走査で trait impl method が追加されるたびに worklist drain と全体再走査を繰り返していた。
- trait impl の候補表は monomorphize 開始時に `module.impls` から構築済みで、後から生成される specialized function によって trait impl 解決結果そのものは変わらない。
- したがって、各 specialized function の body を確定する時点で trait call を解決し、そこで要求された impl method を既存 worklist へ追加すれば十分である。既に解決済みの関数を再走査する必要はない。
- selfhost CLI driver doctest#2 の native wasm emit timeout 調査中、parser/pipeline import graph では specialized 関数数が大きく、全体再走査は post-check codegen 時間を悪化させる候補になっていた。

## 問題

monomorphize resolves remaining trait calls by repeatedly walking every specialized function after each worklist drain. Large selfhost graphs can add new specializations across many iterations, so previously resolved functions are rescanned even though trait impl resolution is static.

## 影響

Selfhost driver codegen spends backend time proportional to repeated full specialized-graph walks. This can make post-check monomorphize scale superlinearly with parser/pipeline import graphs.

## 修正方針

Resolve trait calls once while each specialized function body is being finalized, and let newly requested trait impl functions enter the existing worklist instead of rescanning all old specializations.

## 検証

Run compiler trait/function/generic regressions and the full codegen diagnostics suite; selfhost timeout parent remains open if native emit still exceeds the budget.

## 修正内容

- `resolve_remaining_trait_calls()` と、そのための「worklist drain 後に全 specialized function を再走査する」外側 loop を削除した。
- `process_instantiation()` の function body 確定直前に `resolve_trait_calls_in_block()` を実行し、非 generic function / generic specialization のどちらでも trait call をその function の確定時に 1 回だけ解決するようにした。
- trait call 解決で要求された impl method specialization は従来通り `request_instantiation()` 経由で worklist に積まれるため、静的検査や trait dispatch の意味を弱めずに再走査だけを除去した。
- `nepl-core/tests/check_pipeline.rs` に、non-generic function 内の trait call が確定時に解決され、codegen 可能な graph として残る regression を追加した。

## 完了条件

- `cargo test -p nepl-core --test check_pipeline monomorphize_resolves_trait_calls_when_finalizing_non_generic_function -- --nocapture`: passed
- `cargo test -p nepl-core --test check_pipeline monomorphize_accepts_deep_prefix_chain_without_stack_overflow -- --nocapture`: passed
- `cargo test -p nepl-core --test neplg2 generic_trait_impl_method_resolves_by_trait_args -- --nocapture`: passed
- `cargo test -p nepl-core --test generics -- --nocapture`: 24 passed
- `cargo test -p nepl-core --test functions function_first_class -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test codegen_diagnostics -- --nocapture`: 13 passed
- `cargo test -p nepl-core --test effects pure_indirect_impure_function_value_is_rejected -- --nocapture`: passed
- `target\debug\nepl-cli.exe -i tmp\selfhost_cli_driver_doctest2.nepl --target std --stdlib-root stdlib --emit wasm -o tmp\selfhost_cli_driver_doctest2_emit_after_trait_resolve`: 180 秒 timeout。したがってこの局所的な再走査問題は解消したが、親 issue の selfhost codegen timeout は open 継続。
