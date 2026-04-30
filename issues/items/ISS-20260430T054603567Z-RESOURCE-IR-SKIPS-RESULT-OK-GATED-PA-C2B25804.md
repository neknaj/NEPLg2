---
id: ISS-20260430T054603567Z-RESOURCE-IR-SKIPS-RESULT-OK-GATED-PA-C2B25804
title: "Resource IR skips Result::Ok-gated parameter raw cell initialization summaries"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/initialized_summary.rs, nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_apply.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/src/resource/initialized_variant.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/coverage_hir.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T054603567Z-RESOURCE-IR-SKIPS-RESULT-OK-GATED-PA-C2B25804: Resource IR skips Result::Ok-gated parameter raw cell initialization summaries

## 概要

MemPtr safe wrappers such as store_i32/fill_i32 return Result::Ok only after initializing the pointee raw cell, but Resource IR summaries keep only effects guaranteed on every return path. The Err path removes the param cell fact, so callers matching Result::Ok still see the pointee as uninitialized and memory_safety doctests fail with resource.cell.uninit.

## 対象

- `nepl-core/src/resource/initialized_summary.rs, nepl-core/src/resource/initialized_summary_build.rs, nepl-core/src/resource/initialized_summary_apply.rs, nepl-core/src/resource/initialized_control.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- 未記入

## 問題

MemPtr safe wrappers such as store_i32/fill_i32 return Result::Ok only after initializing the pointee raw cell, but Resource IR summaries keep only effects guaranteed on every return path. The Err path removes the param cell fact, so callers matching Result::Ok still see the pointee as uninitialized and memory_safety doctests fail with resource.cell.uninit.

## 影響

Valid checked-memory code is rejected under the strict initialized-cell gate. This pressures stdlib tests toward unsafe raw access or weakens confidence in MemPtr wrapper contracts, both of which conflict with the static-check correctness requirement.

## 修正方針

Represent raw cell initialization summary facts that are gated by a returned enum variant. Record parameter-cell effects for each branch returning Result::Ok/Option::Some and apply them only inside the matching match arm for the call result, while keeping unconditional summaries conservative.

## 検証

Add focused Resource IR regression coverage for Result::Ok-gated parameter raw cell initialization and keep the conditional non-Ok path conservative. Re-run tests/stdlib/memory_safety.n.md and resource_ir targeted tests.

## 2026-04-30 解決

Resource IR の raw cell initialization summary に、戻り値 enum variant で gated された parameter raw cell 初期化 fact と raw load 事前条件 fact を追加した。`store_i32(MemPtr<i32>, i32)` / `fill_i32(MemPtr<i32>, ...)` の `Result::Ok` arm では caller 側の `p.raw.*` cell を initialized として反映し、`Result::Err` arm には反映しない。

同時に、`load_i32(MemPtr<i32>) -> Option<i32>` の `Option::Some` arm では pointee cell が initialized であることを要求する summary を持たせた。これにより、MemPtr safe wrapper call を call-site の直接 `RawMemory` として誤って下げず、wrapper 内の分岐結果に沿って CellState を更新できる。

Resource lowering coverage も同じ分類へ更新し、MemPtr safe wrapper 呼び出しを HIR 側の direct raw memory count に含めないようにした。raw `i32` helper 呼び出しは従来通り `RawMemory` として coverage gate に残る。

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 141 passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-result-ok-param-init-summary.json -j 1`: 12 total / 7 passed / 5 failed
- main merge 後 `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-result-ok-param-init-summary-merge.json -j 1`: 12 total / 7 passed / 5 failed

残件:

- `ISS-20260430T060552075Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-5C2C877E`: checked MemPtr free/realloc の owner 消費は Result::Ok gated owner summary が必要。
- `ISS-20260430T060600668Z-CHECKED-MEMPTR-LOAD-VARIANT-REQUIREM-1A1ADF53`: `mem_ptr_wrap 0` のような静的 invalid pointer では `Option::Some` arm を unreachable として扱う refinement が必要。
