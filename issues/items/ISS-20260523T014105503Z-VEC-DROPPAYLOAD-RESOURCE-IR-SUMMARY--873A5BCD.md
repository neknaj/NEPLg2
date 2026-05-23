---
id: ISS-20260523T014105503Z-VEC-DROPPAYLOAD-RESOURCE-IR-SUMMARY--873A5BCD
title: "Vec DropPayload Resource IR summary stages exceed focused test budget"
area: core
status: fixed
resolved: true
priority: P2
type: performance
created: 2026-05-23
updated: 2026-05-23
target: "nepl-core/src/resource/**, nepl-core/tests/resource_ir.rs, stdlib/alloc/collections/vec/**"
---

# ISS-20260523T014105503Z-VEC-DROPPAYLOAD-RESOURCE-IR-SUMMARY--873A5BCD: Vec DropPayload Resource IR summary stages exceed focused test budget

## 概要

Focused monomorphized stdlib Vec<DropPayload> Resource IR regressions spend about 84 seconds in a single Rust test. NEPL_COMPILE_STAGE_TIMING shows resource_initialized_i32_scalar_summaries around 44s and resource_initialized_collection_slot_summaries around 28s for both the existing push/free regression and the new borrowed observer regression. This is a Resource IR summary stage cost problem, not a runtime problem and not specific to the new observer API.

## 対象

- `nepl-core/src/resource/**, nepl-core/tests/resource_ir.rs, stdlib/alloc/collections/vec/**`

## 根拠

- `NEPL_COMPILE_STAGE_TIMING=1 cargo test -q -p nepl-core resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle -- --nocapture`:
  - test wall time: 約 84.75s
  - `resource_initialized_i32_scalar_summaries=44677ms`
  - `resource_initialized_collection_slot_summaries=27707ms`
  - `resource_initialized_raw_init_summaries=7047ms`
- `NEPL_COMPILE_STAGE_TIMING=1 cargo test -q -p nepl-core resource_ir_vec_borrow_at_predicate_or -- --nocapture`:
  - test wall time: 約 83.80s
  - `resource_initialized_i32_scalar_summaries=43990ms`
  - `resource_initialized_collection_slot_summaries=27893ms`
  - `resource_initialized_raw_init_summaries=6641ms`
- 既存の `Vec<DropPayload>.push -> free` regression と新規 borrowed observer regression がほぼ同じ profile を示すため、`collection_slot_borrow_ref` 個別の設計ミスではなく、monomorphized stdlib Vec + Drop payload に対する Resource IR summary stage 全体の計算量問題として扱う。
- 関連設計: [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)

## 問題

Focused monomorphized stdlib Vec<DropPayload> Resource IR regressions spend about 84 seconds in a single Rust test. NEPL_COMPILE_STAGE_TIMING shows resource_initialized_i32_scalar_summaries around 44s and resource_initialized_collection_slot_summaries around 28s for both the existing push/free regression and the new borrowed observer regression. This is a Resource IR summary stage cost problem, not a runtime problem and not specific to the new observer API.

## 影響

A single focused regression can exceed practical local budgets and broad filters can time out. If left untracked, future stdlib observer or non-Copy transform work may hide real correctness failures behind slow Resource IR summary stages.

## 修正方針

Investigate the i32 scalar and collection slot summary dependency graph for monomorphized stdlib Vec<DropPayload> code. Reduce recomputation and summary propagation by source-derived dependencies, typed carrier pruning, and generic Resource IR proof boundaries; do not solve by deleting coverage, widening timeouts, or adding stdlib function allowlists.

## 解決内容

- i32 scalar return facts / condition query の収集で、同じ summary 内の `I32ConditionQueryContext` を共有し、条件照会ごとの leaf projection 展開と alias 条件評価を繰り返さないようにした。
- i32 scalar path state は exact equality で重複排除できるようにし、分岐・loop 由来の同一状態が次段 summary へ重複伝播しないようにした。
- collection slot lifecycle summary は、全関数を機械的に再計算するのではなく、source-derived dependency と typed carrier 判定から必要な関数だけを対象にした。stdlib 関数名 allowlist は使っていない。
- 修正中に、collection slot の storage carrier と owner carrier を同一視していた既存不備を発見したため、`type_can_carry_collection_slot_storage` を分離した。これにより、`RegionToken` owner token だけでなく、collection slot state がぶら下がる非 Copy storage 値の Move / return summary transfer も正しく扱う。
- collection slot の `StorageOffset(Known(0))` は raw cell の「0 offset」と違って第 0 slot identity なので、collection slot state transfer / StorageDealloc では raw cell 用の zero-offset canonicalization を使わないようにした。一方で `InitializeEmpty` など raw value proof が必要な lifecycle event は従来どおり raw cell alias canonicalization を使う。

## 検証

With NEPL_COMPILE_STAGE_TIMING=1, the existing Vec<DropPayload>.push -> free regression and borrowed observer regression should complete comfortably below the default focused-test budget while preserving Resource IR diagnostics and coverage.

実測:

- `resource_ir_vec_borrow_at_predicate_or_observes_drop_payload_without_move`
  - test wall time: 約 39.78s
  - `resource_initialized_i32_scalar_summaries=12919ms`
  - `resource_initialized_collection_slot_summaries=12787ms`
  - `resource_initialized_raw_init_summaries=7886ms`
  - `resource_initialized_function_checks=3101ms`
- `resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle`
  - test wall time: 約 37.39s
  - `resource_initialized_i32_scalar_summaries=12880ms`
  - `resource_initialized_collection_slot_summaries=11660ms`
  - `resource_initialized_raw_init_summaries=7276ms`
  - `resource_initialized_function_checks=2808ms`

Focused regression:

- `cargo fmt --check`
- `git diff --check`
- `cargo test -q -p nepl-core collection_slot_call_summary -- --nocapture`
- `cargo test -q -p nepl-core resource::collection_slot_storage_carrier::tests -- --nocapture`
- `cargo test -q -p nepl-core resource::collection_slot_summary_relevance::tests -- --nocapture`
- `cargo test -q -p nepl-core resource::collection_slot_summary_build::tests -- --nocapture`
- `cargo test -q -p nepl-core --test resource_ir resource_ir_collection_slot_move_transfers_slot_state_to_output -- --exact --nocapture`
- `NEPL_COMPILE_STAGE_TIMING=1 cargo test -p nepl-core --test resource_ir resource_ir_vec_borrow_at_predicate_or_observes_drop_payload_without_move -- --test-threads=1 --exact --nocapture`
- `NEPL_COMPILE_STAGE_TIMING=1 cargo test -p nepl-core --test resource_ir resource_ir_initialized_check_vec_drop_push_free_closes_stdlib_lifecycle -- --test-threads=1 --exact --nocapture`
