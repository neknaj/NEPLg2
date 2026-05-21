---
id: ISS-20260521T092635167Z-RESOURCE-RESPONSIBILITY-POLICY-DOES--420EC501
title: "Resource responsibility policy is stale for path state and return summary modules"
area: tools
status: fixed
resolved: true
priority: P1
type: test
created: 2026-05-21
updated: 2026-05-21
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_path_state.rs"
---

# ISS-20260521T092635167Z-RESOURCE-RESPONSIBILITY-POLICY-DOES--420EC501: Resource responsibility policy is stale for path state and return summary modules

## 概要

The resource checker responsibility policy requires every nepl-core/src/resource/*.rs module to be listed with an explicit line limit, but initialized_path_state.rs was added without a monitor entry. Once that monitor gap is closed, the same policy also exposes that collection_slot_summary_return_collect.rs has kept multiple return-summary responsibilities in one file and exceeds its intended split limit.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/initialized_path_state.rs`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` は `nepl-core/src/resource/*.rs` の全 module が line-limit table に登録されていることを検査する。
- `initialized_path_state.rs` は branch / match join 後の path-correlated Resource IR state を保持する Stage 6 compiler-core module だが、file list、`mod` declaration list、line-limit table に未登録だった。
- そのため policy test は main 上で失敗し、path-correlated state module の肥大化を監視できなかった。
- 監視漏れを閉じると、`collection_slot_summary_return_collect.rs` が return value producer tracing、call summary composition、state replay、dedupe helper を同じ file に抱え、line limit を超えていることも同じ policy gate で検出された。
- さらに `coverage.rs`、`coverage_hir.rs`、`initialized_control.rs`、`initialized_raw_memory_access.rs`、`raw_cell_value_flow*.rs` は現行 main の cohesive module サイズに対して line-limit table が古く、policy test が実装変更なしでも失敗する状態だった。
- 静的検査の正確性を保つには、Resource checker の state / proof helper が再び flat module に戻らないよう、責務分割 policy 自体も同期されている必要がある。

## 問題

The resource checker responsibility policy requires every nepl-core/src/resource/*.rs module to be listed with an explicit line limit, but initialized_path_state.rs was added without a monitor entry. Once that monitor gap is closed, the same policy also exposes that collection_slot_summary_return_collect.rs has kept multiple return-summary responsibilities in one file and exceeds its intended split limit.

## 影響

Static-check path correlation code can grow without the responsibility split gate enforcing a local limit, and return-summary collection can keep accumulating unrelated logic in one flat module. Both weaken the project rule that large resource checker changes remain modular and reviewable.

## 修正方針

Add initialized_path_state.rs to the resource responsibility file list, mod declaration list, and line-limit table with a tight limit matching its current scope. Split collection_slot_summary_return_collect.rs by responsibility instead of raising the stale limit. For pre-existing cohesive modules whose implementation already exceeded stale budgets, synchronize the budget to the current reviewed scope so future growth is again detected.

## 修正内容

- `nodesrc/test_resource_checker_responsibility.js` の monitored file list に `initialized_path_state.rs` を追加した。
- `mod initialized_path_state;` の declaration 監視にも追加し、module declaration と file list がずれた場合に policy test が検出できるようにした。
- line-limit table へ `initialized_path_state.rs` の上限を追加し、path alternatives state helper が今後肥大化した場合に検出できるようにした。
- `collection_slot_summary_return_collect.rs` から return value producer tracing、call summary composition、state replay、dedupe helper をそれぞれ `collection_slot_summary_return_value.rs`、`collection_slot_summary_return_call.rs`、`collection_slot_summary_return_state.rs`、`collection_slot_summary_return_unique.rs` へ分割した。
- 新規分割 module も monitored file list / `mod` declaration list / line-limit table に追加した。
- 既存の cohesive module については、現行 main の実サイズを確認したうえで line-limit table を同期し、policy test が stale budget ではなく今後の増加を検出する状態に戻した。

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and keep it passing.
