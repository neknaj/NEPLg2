---
id: ISS-20260531T050630951Z-I32-SCALAR-SUMMARY-NEEDS-STABLE-MIRR-E70E2D93
title: "i32 scalar summary needs stable mirror replay"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/initialized_scalar_flow.rs; nepl-core/src/resource/resource_summary_value_cache"
---

# ISS-20260531T050630951Z-I32-SCALAR-SUMMARY-NEEDS-STABLE-MIRR-E70E2D93: i32 scalar summary needs stable mirror replay

## 概要

RPN same-session code edit では raw-init complete leaf replay が成功しても、
`i32_scalar_summary` 固定点計算が 209 recomputations 残っている。`I32ScalarReturnFacts`
を stable mirror value として保存・再投影し、unchanged stdlib function の i32 scalar
summary を worklist 前に preseed できるようにする。

## 対象

- `nepl-core/src/resource/initialized_scalar_flow.rs`
- `nepl-core/src/resource/i32_scalar_return_facts.rs`
- `nepl-core/src/resource/resource_summary_value_cache`
- `nepl-web/src/lib.rs`

## 根拠

- `tmp/rpn_stage_breakdown_code_edit_20260531.json` の code edit delta では、`resource_i32_scalar_summary_recomputations=209`、`resource_i32_scalar_summary_count=87` が残っている。
- native release timing では `resource_initialized_i32_scalar_summaries=1558ms` で、`sb_append_result`、`byte_builder_reserve`、`byte_builder_push_bytes_ref` が上位だった。
- raw-init 側は同じ測定で `resource_summary_value_replayed_ops=253`、`raw_init_param_facts_bypasses=0` まで進んでいるため、次の支配項は raw-init replay miss ではない。

## 問題

i32 scalar summary は、call 境界を越えて i32 leaf alias / offset / relation /
constant / condition を伝播するために必要である。一方で、現状は session cache に
stable value を持たず、compile ごとに reachable function の fixed-point worklist を
再実行している。RPN のように stdlib-heavy だが微小編集だけの workload では、
unchanged stdlib function の summary を再計算することが秒単位 compile time の一部に
なっている。

## 影響

- Web playground の code edit compile が 0.5 秒未満に入らない。
- raw-init replay の改善効果が i32 scalar fixed-point の全関数再実行に隠れる。
- `MemoKey` / `MemoValue` や private cache purity の設計でも i32 条件・hash・size などの pure query を使うため、summary cache 境界を明確にする必要がある。

## 修正方針

- `I32ScalarReturnFacts` の complete stable mirror entry を作る。
- key は Resource summary value cache と同じく namespace、function identity、function body hash、type parameter boundary、generic type argument、source capability policy、summary kind/version を含める。
- stable entry には `TypeId` / `Span` / `SourceMap` / raw alias graph を保存しない。
- return projection、parameter projection、scalar type、relation op、condition、constant value を現在 compile の `TypeCtx` / function signature へ fail-closed に再投影する。
- replay 成功時は `compute_i32_scalar_return_summaries` の worklist relevant function から外し、dependency closure の body/source policy/type boundary が変わった場合は通常 recompute に戻す。

## 検証

- focused unit test で alias / offset / relation / constant / return condition / parameter condition を含む i32 scalar summary が replay されることを確認する。
- signature type、function body、dependency body、source capability policy の変更で stale replay しないことを確認する。
- RPN same-session code edit JSON で `resource_i32_scalar_summary_recomputations` が大きく減ることを確認する。
