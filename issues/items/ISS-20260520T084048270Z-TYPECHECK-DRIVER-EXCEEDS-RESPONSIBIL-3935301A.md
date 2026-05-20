---
id: ISS-20260520T084048270Z-TYPECHECK-DRIVER-EXCEEDS-RESPONSIBIL-3935301A
title: "typecheck driver and overload selection exceed responsibility split limits"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-20
updated: 2026-05-20
target: "nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/driver_span.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/overload_candidate.rs, nepl-core/src/typecheck/overload_narrowing.rs, nepl-core/src/typecheck.rs, nodesrc/test_static_check_boundary_responsibility.js"
---

# ISS-20260520T084048270Z-TYPECHECK-DRIVER-EXCEEDS-RESPONSIBIL-3935301A: typecheck driver and overload selection exceed responsibility split limits

## 概要

typecheck/driver.rs has grown to 1701 lines and fails the static check responsibility policy limit of 1700 lines. After separating that helper responsibility, the same policy exposed overload_selection.rs at 659 lines against a 460-line limit. Both files still owned helper/model responsibilities that should be separated from orchestration.

## 対象

- `nepl-core/src/typecheck/driver.rs, nepl-core/src/typecheck/driver_span.rs, nepl-core/src/typecheck/overload_selection.rs, nepl-core/src/typecheck/overload_candidate.rs, nepl-core/src/typecheck/overload_narrowing.rs, nepl-core/src/typecheck.rs, nodesrc/test_static_check_boundary_responsibility.js`

## 根拠

- `node nodesrc/test_static_check_boundary_responsibility.js` が `typecheck/driver.rs has 1701 lines; responsibility split limit is 1700` で失敗していた。
- `typecheck/driver.rs` には typecheck orchestration とは独立した top-level declaration span/key helper が残っていた。
- driver helper を分離した後、同じ policy が `typecheck/overload_selection.rs has 659 lines; responsibility split limit is 460` を検出した。
- `overload_selection.rs` は overload candidate model、rejection/materialization stats、ambiguity reason、narrowing algorithm、candidate construction を同じ file に抱えていた。
- line limit policy は巨大 driver への責務回帰を検出するための policy なので、上限緩和ではなく helper responsibility の分離で解く必要がある。

## 問題

typecheck/driver.rs and overload_selection.rs exceed responsibility split limits. The line-limit policy itself is correctly exposing oversized orchestration files, so the fix must split responsibilities instead of changing policy thresholds.

## 影響

Source policy regressions fail, and future typecheck changes are pushed toward oversized files instead of explicit responsibility modules. In overload selection specifically, candidate data model changes and narrowing algorithm changes are harder to review independently.

## 修正方針

Move top-level declaration span/key helper responsibility out of driver.rs into a dedicated typecheck driver support module. Split overload candidate model/stats and narrowing into dedicated modules so overload_selection.rs keeps candidate construction and diagnostic connection. Keep the line-limit policy unchanged, and run the responsibility policy plus cargo check.

## 検証

Run node nodesrc/test_static_check_boundary_responsibility.js, cargo check -p nepl-core, node nodesrc/issues.js check, and git diff --check.

## 2026-05-20 Agent 1 修正

`typecheck/driver_span.rs` を追加し、`span_key` と `top_level_definition_span` を `driver.rs` から分離した。`driver.rs` は top-level orchestration と declaration pass の制御に集中し、top-level item span/key extraction は dedicated support module 側に置く。

さらに `typecheck/overload_candidate.rs` と `typecheck/overload_narrowing.rs` を追加し、candidate/rejection/stats/ambiguity payload と、pure preference / signature dedup / specificity などの narrowing algorithm を `overload_selection.rs` から分けた。`overload_selection.rs` は候補の構築、generic constraint、診断への接続に集中する。

line limit は変更していない。`driver.rs` と `overload_selection.rs` は policy 上限以下に戻り、`node nodesrc/test_static_check_boundary_responsibility.js` が通る状態になった。

検証:

- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test overload -- --nocapture`: 10 passed
- `cargo test -p nepl-core --test generics -- --nocapture`: 25 passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
