---
id: ISS-20260430T083411167Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-D35A0DAD
title: "Resource IR lacks Result::Ok-gated owner consumption for checked raw dealloc"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-30
updated: 2026-05-12
target: "nepl-core/src/resource/condition_fact.rs, nepl-core/src/resource/initialized_alias.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_return_apply_source.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_summary_variant_paths.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/src/resource/owner_variant_value_condition.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md"
---

# ISS-20260430T083411167Z-RESOURCE-IR-LACKS-RESULT-OK-GATED-OW-D35A0DAD: Resource IR lacks Result::Ok-gated owner consumption for checked raw dealloc

## 概要

The checked raw dealloc wrapper returns Result and consumes an alloc owner only on Result::Ok, but Resource IR owner summaries currently do not propagate that raw i32 owner consumption to callers. tests/stdlib/memory_safety.n.md doctest#4 fails with resource.owner.maybe_leak for the allocated pointer.

## 対象

- `nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_summary_variant_paths.rs, nepl-core/tests/resource_ir.rs, tests/stdlib/memory_safety.n.md`

## 根拠

- `tests/stdlib/memory_safety.n.md::doctest#4` が `alloc 8` の `Result::Ok p` 後に `dealloc p 8` を match すると、`dealloc` の `Result::Err` arm も到達可能と見なされ、`p` の owner が leak すると診断されていた。
- `stdlib/core/mem.nepl` の `alloc` は `lt 0 ptr` の then 側だけ `Result::Ok ptr` を返すため、Ok payload は `ptr > 0` である。
- `dealloc` は `or le ptr 0 lt size 0` の then 側だけ `Result::Err` を返すため、呼び出し側で `ptr > 0` かつ `size >= 0` が既知なら Err arm は到達不能である。

## 問題

The checked raw dealloc wrapper returns Result and consumes an alloc owner only on Result::Ok, but Resource IR owner summaries currently do not propagate that raw i32 owner consumption to callers. tests/stdlib/memory_safety.n.md doctest#4 fails with resource.owner.maybe_leak for the allocated pointer.

## 影響

Valid memory-safe alloc/dealloc code is rejected, while weakening owner checks would hide leaks and double frees in self-host storage cleanup.

## 修正方針

Summarize Result::Ok-gated owner consumption for checked raw dealloc without treating Err as consumed. Keep raw direct dealloc ownership checks strict and add Resource IR regression coverage.

## 検証

Add Resource IR regression for alloc -> checked dealloc where Ok consumes the raw owner and Err keeps it reserved/available as appropriate. Re-run focused resource_ir owner tests and memory_safety doctest#4.

## 2026-04-30 修正

- `ResourceConditionFact` を `Negative` / `NonNegative` / `Any` / `All` まで拡張し、`or le ptr 0 lt size 0` を文字列や ad hoc 判定ではなく enum tree として lowering するようにした。
- `RawCellAddressAliases` に i32 exact value と value condition を持たせ、literal `8` や `Result::Ok p` 由来の `p > 0` を branch / match / read / move / merge の中で保守的に伝播できるようにした。
- owner summary に variant 到達条件と variant payload 条件を追加し、`alloc` の Ok payload が positive であること、`dealloc` の Err 条件が `ptr <= 0 || size < 0` であることを caller 側の match 到達性へ反映した。
- owner checker の match 処理は、summary から false と証明できる variant arm を skip する。今回の `dealloc p 8` では Err arm が到達不能になり、Ok arm だけで raw owner を消費する。
- 回帰テスト `resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption` を追加し、Resource IR owner check と通常 compiler pipeline の両方で同じ checked dealloc pattern を検証した。

## 検証結果

- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 150 passed
- `cargo fmt --check -p nepl-core`: passed
- `git diff --check`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/memory-safety-owner-variant-value-conditions.json -j 1 --dist web/dist`: 12 total / 9 passed / 3 failed

`memory_safety.n.md::doctest#4` は本修正で pass した。残る doctest#6/#7/#8 は `RegionToken` / `MemPtr` の owner model 分離の残件として、`ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF` に追記した。

## 2026-05-12 回帰修正

- 現行 main で `resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption` と `tests/stdlib/memory_safety.n.md` が再び `resource.owner.maybe_leak` を出していた。
- 調査の結果、summary そのものは `alloc` の Ok payload に `Positive` / `NonNegative` を持ち、`dealloc` の Err/Ok variant condition も保持していた。
- 回帰の根本原因は、caller へ summary source を適用するときに、suffix が空の `OwnerProjectionSource` でも summary 側の抽象 `TypeId` へ置き換えていたことだった。
- `Place` と raw i32 condition の照合は型込みで行われるため、monomorphized caller の `p` / `tmp` に保持されている `Positive` fact を検索できず、`dealloc` の Err arm を到達不能と証明できなかった。
- `owner_projection_source_place_for_arg` / `summary_projection_place` を導入し、空 suffix では caller 側 `Place.ty` を保持し、非空 suffix では projection summary type を使う規則へ統一した。
- Result variant の owner consumption / owner return / payload condition / value condition truth / pending result resolution を同じ規則へ寄せ、Result::Ok-gated owner consumption が monomorphized type ID 差で欠落しないようにした。

## 2026-05-12 回帰検証

- `cargo fmt --check -p nepl-core`: passed
- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption -- --nocapture`: passed
- `trunk build`: passed
- `node nodesrc/tests.js -i tests/stdlib/memory_safety.n.md --no-tree -o tmp/agent1-memory-safety-result-ok-dealloc-owner.json -j 1 --dist web/dist`: total=23, passed=23
