---
id: ISS-20260512T061533036Z-RESOURCE-OWNER-CHECK-EXCEEDS-RESPONS-687898EE
title: "Resource owner_check exceeds responsibility split limit after i32 fact changes"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-12
updated: 2026-05-12
target: "nepl-core/src/resource/owner_check.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260512T061533036Z-RESOURCE-OWNER-CHECK-EXCEEDS-RESPONS-687898EE: Resource owner_check exceeds responsibility split limit after i32 fact changes

## 概要

remote main commit `3487e386` と source policy rename 追従後、`nodesrc/test_resource_checker_responsibility.js` は次の blocker として `owner_check.rs has 813 lines; responsibility split limit is 800` を報告した。Resource IR owner summary と i32 fact 変更のあと、`owner_check.rs` に small helper / deferred merge が残り、traversal module が再び上限を超えていた。

## 対象

- `nepl-core/src/resource/owner_check.rs`
- `nepl-core/src/resource/owner_check_utils.rs`
- `nepl-core/src/resource/mod.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_check.rs has 813 lines; responsibility split limit is 800` で失敗した。
- `owner_check.rs` 末尾には `raw_owner_alias_moves_into_wrapper`、`merge_owner_deferred`、`direct_raw_memory_effect` が残っており、function/block/op traversal とは別の utility / deferred-state merge 責務だった。

## 問題

`owner_check.rs` が traversal / dispatch だけでなく、owner alias wrapper 判定、deferred count merge、raw memory effect 判定を抱えていた。小さな helper であっても owner checker entrypoint に戻すと、責務分割 policy の上限を再び超える。

## 影響

Resource owner checking starts accumulating helper predicates and deferred-state plumbing in the traversal module again. This weakens the responsibility split policy used to keep memory-safety checks maintainable.

## 修正方針

上限は緩めない。small owner-check utility predicates / deferred merge helper を dedicated module へ移し、`owner_check.rs` は traversal と dispatch に集中させる。source policy は新 module の存在、`mod` 宣言、line budget を検査する。

## 修正

- `nepl-core/src/resource/owner_check_utils.rs` を追加した。
- `raw_owner_alias_moves_into_wrapper`、`merge_owner_deferred`、`direct_raw_memory_effect` を `owner_check_utils.rs` へ移した。
- `nepl-core/src/resource/mod.rs` に `mod owner_check_utils;` を追加した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_check_utils.rs` の存在、`mod` 宣言、80 行 budget を追加した。
- line count は `owner_check.rs` 795、`owner_check_utils.rs` 22。

## 検証

- `cargo fmt -p nepl-core`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `owner_check.rs` blocker は解消。次の別件として `owner_summary_variant_conditions.rs has 295 lines; responsibility split limit is 260` に到達したため、`ISS-20260512T062230660Z-RESOURCE-OWNER-SUMMARY-VARIANT-CONDI-F79EFC3E` を追加した。
- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/issues.js check`: passed
- `git diff --check`: passed
