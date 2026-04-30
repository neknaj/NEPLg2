---
id: ISS-20260430T070449791Z-FALLIBLE-OWNER-EFFECTS-DO-NOT-RESERV-32CC9198
title: "Fallible owner effects do not reserve owners before Result refinement"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-04-30
target: "nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_control.rs, nepl-core/src/resource/model.rs, nepl-core/src/compiler.rs, nepl-core/src/diagnostic_codes.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260430T070449791Z-FALLIBLE-OWNER-EFFECTS-DO-NOT-RESERV-32CC9198: Fallible owner effects do not reserve owners before Result refinement

## 概要

PendingVariantOwnerEffects delays owner consumption/return until a Result match arm is selected, but the source owner remains usable before that refinement. A caller can ignore or delay matching a fallible owner effect and may reuse an owner that is consumed on the success variant.

## 対象

- `nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- 未記入

## 問題

PendingVariantOwnerEffects delays owner consumption/return until a Result match arm is selected, but the source owner remains usable before that refinement. A caller can ignore or delay matching a fallible owner effect and may reuse an owner that is consumed on the success variant.

## 影響

This can become a false negative for memory safety: a fallible dealloc/realloc wrapper may consume an owner at runtime, while the Resource IR checker still permits using the original owner until the Result is matched.

## 修正方針

Represent fallible owner effects as a reserved/path-dependent owner state until the result is refined. Before a matching Result arm or equivalent refinement, direct use/dealloc/return of the reserved source must be rejected or require explicit handling of all variants.

## 検証

Add Resource IR regressions where dealloc_ptr/realloc_ptr result is ignored or matched after reusing the original owner, and assert resource.owner diagnostics are emitted.

## 2026-04-30 修正

- `OwnerState::Reserved` と `resource.owner.reserved` を追加し、fallible owner effect の Result が refine される前の元 owner 利用を専用状態として診断するようにした。
- `PendingVariantOwnerEffects` に予約 source 検査を追加し、pending consumption / pending return の source と overlap する read / move / assign / return / call argument / raw dealloc / raw realloc / raw memory access を拒否するようにした。
- `match` arm で Result variant が確定した時点で pending effect を解決し、同一 source に紐づく Result copy/read temporary の予約も残らないようにした。
- branch / match の戻り値 transfer でも予約 source を確認し、未精査 Result をまたいで owner を返す経路を拒否するようにした。
- `dealloc_ptr` と `realloc_ptr` の Result を match する前に元 `MemPtr` を `dealloc_raw` する回帰テストを追加し、`OwnerState::Reserved` で落ちることを確認した。

## 検証結果

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core resource_owner_gate_maps_reserved_owner_to_reserved_code -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 145 passed
- `node nodesrc/issues.js check`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/compiler/move_effect.n.md --no-tree -o tmp/move-effect-owner-reservation.json -j 1 --dist web/dist`: 110 total / 110 passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-owner-reservation.json -j 1 --dist web/dist`: 12 total / 7 passed / 5 failed

`memory_safety.n.md` の残り 5 件は、本 issue の予約診断追加では増減していない既存残件として継続する。
