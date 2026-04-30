---
id: ISS-20260430T012746721Z-RESOURCE-IR-INITIALIZED-SUMMARIES-SK-727D49FD
title: "Resource IR initialized summaries skip unit-returning in-place helper effects"
area: core
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_summary*.rs, nepl-core/src/resource/initialized.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260430T012746721Z-RESOURCE-IR-INITIALIZED-SUMMARIES-SK-727D49FD: Resource IR initialized summaries skip unit-returning in-place helper effects

## 概要

Initialized-cell function summaries are currently discovered only for functions with a non-unit result, even though the summary type already has param_cells for caller-visible side effects on argument raw storage.

## 対象

- `nepl-core/src/resource/initialized_summary.rs`
- `nepl-core/src/resource/initialized_summary_build.rs`
- `nepl-core/src/resource/initialized_summary_apply.rs`
- `nepl-core/src/resource/initialized.rs`

## 根拠

- `ISS-20260429T234223901Z-KP-RAW-MEMORY-TESTS-FAIL-RESOURCE-IR-9B5964DA` の修正で `initialized_return` が追加され、戻り値にぶら下がる raw header / pointee の initialized cell を caller へ伝播できるようになった。
- ただし現在の collection は `Return { value: Some(...) }` を起点にしており、`Return { value: None }` の unit-returning helper では引数側の in-place effect を caller へ返せない。
- この問題は現在の `RawCellInitializationReturnSummary` が return value 配下の raw cells を対象にしているためで、function resource effect として引数 side effect を別に表す必要がある。
- このままだと `fn fill_buf <(i32)*>()> (p): ...` のような helper を通した initialized transition が caller で失われる。

## 問題

Initialized-cell function summaries are currently discovered only for functions with a non-unit result, even though the summary type already has param_cells for caller-visible side effects on argument raw storage.

## 影響

A user-defined or stdlib in-place helper that returns unit after initializing an argument buffer can still be rejected as RawMemoryLoadCell Uninit at the caller, pressuring tests or stdlib code toward direct raw operations instead of well-factored helpers.

## 修正方針

Redesign initialized summaries as function resource effects rather than return summaries: include unit-returning functions, keep return effects optional, and merge path facts conservatively so argument side effects are applied only when guaranteed.

## 修正内容

- `RawCellInitializationReturnSummary` を廃止し、`RawCellInitializationFunctionSummary` として戻り値配下の initialized raw cell と引数配下の caller-visible initialized raw cell を分けて表現した。
- summary 構築を `initialized_summary_build.rs`、summary data model を `initialized_summary.rs`、caller への適用を `initialized_summary_apply.rs` に分割し、古い return-only module 名を残さない構成にした。
- unit-returning helper でも引数 raw storage への store が全 return path で保証される場合は caller 側の該当 raw cell を initialized にするようにした。
- branch 等で片側 path だけが初期化する場合は `MaybeMoved` / 非 initialized state として扱い、無条件 summary へ昇格しないようにした。

## 検証

Add Resource IR regressions for a unit helper that initializes an argument raw cell and for a conditional unit helper that initializes only on one path; the first must pass and the second must not become an unsound unconditional initialization.

- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_summarizes_unit_helper_argument_raw_cell_initialization -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_keeps_conditional_unit_helper_argument_init_conservative -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: `139 passed`
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
