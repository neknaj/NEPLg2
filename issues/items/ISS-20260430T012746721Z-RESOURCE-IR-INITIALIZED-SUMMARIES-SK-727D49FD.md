---
id: ISS-20260430T012746721Z-RESOURCE-IR-INITIALIZED-SUMMARIES-SK-727D49FD
title: "Resource IR initialized summaries skip unit-returning in-place helper effects"
area: core
status: open
resolved: false
priority: P2
type: architecture
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_return.rs, nepl-core/src/resource/initialized.rs"
source: doc/neplg2/static_check_complexity_reduction_plan.md#stage-4-resource-check-への移行
---

# ISS-20260430T012746721Z-RESOURCE-IR-INITIALIZED-SUMMARIES-SK-727D49FD: Resource IR initialized summaries skip unit-returning in-place helper effects

## 概要

Initialized-cell function summaries are currently discovered only for functions with a non-unit result, even though the summary type already has param_cells for caller-visible side effects on argument raw storage.

## 対象

- `nepl-core/src/resource/initialized_summary.rs, nepl-core/src/resource/initialized.rs`

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

## 検証

Add Resource IR regressions for a unit helper that initializes an argument raw cell and for a conditional unit helper that initializes only on one path; the first must pass and the second must not become an unsound unconditional initialization.
