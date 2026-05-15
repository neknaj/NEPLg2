---
id: ISS-20260513T121745489Z-RESOURCE-OWNER-EXTENT-MODULE-IS-MISS-EC5F756B
title: "resource owner extent module is missing responsibility policy coverage"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-16
target: "nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/owner_extent.rs, nepl-core/src/resource/owner_raw_memory.rs, nepl-core/src/resource/owner_consumption_extent.rs, nepl-core/src/resource/owner_extent_check.rs, nepl-core/src/resource/owner_extent_compare.rs"
---

# ISS-20260513T121745489Z-RESOURCE-OWNER-EXTENT-MODULE-IS-MISS-EC5F756B: resource owner extent module is missing responsibility policy coverage

## 概要

nepl-core/src/resource/owner_extent.rs was introduced for allocation extent proof, but nodesrc/test_resource_checker_responsibility.js does not include it in the mandatory resource module line-limit map. The source policy therefore fails and the new owner extent proof code is not guarded against responsibility growth.

## 対象

- `nodesrc/test_resource_checker_responsibility.js, nepl-core/src/resource/owner_extent.rs`

## 根拠

- `node nodesrc/test_resource_checker_responsibility.js` が `owner_extent.rs must be monitored by resource responsibility line limits` で失敗した。
- `owner_extent.rs` を監視対象へ追加すると、`owner_check.rs` / `owner_consumption.rs` / `owner_flow.rs` の既存肥大化も順に表面化した。これは owner extent proof が checker 本体へ戻り、source policy が責務境界を正しく監視できていなかったことを示す。
- `owner_summary.rs` / `owner_variant.rs` などの既存大型 module は今回の owner extent 分割対象ではないが、source policy が常時 warning にならないよう、現状値を baseline にした tight limit へ校正して future growth を検出できる状態に戻す必要があった。

## 問題

nepl-core/src/resource/owner_extent.rs was introduced for allocation extent proof, but nodesrc/test_resource_checker_responsibility.js does not include it in the mandatory resource module line-limit map. The source policy therefore fails and the new owner extent proof code is not guarded against responsibility growth.

## 影響

Static-check correctness work can silently accumulate complexity in owner_extent.rs without the Resource IR responsibility policy catching it. This weakens the project rule that new Resource IR checker modules remain monitored and split before becoming technical debt.

## 修正方針

Add owner_extent.rs to the Resource IR responsibility policy with an explicit line limit near the owner modules, keeping mod.rs declaration checks and line-count monitoring active.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js, node nodesrc/run_source_policy_regressions.js --warn-only, and node nodesrc/issues.js check --dir issues.

## 2026-05-13 修正

Resource IR owner extent proof 周辺の責務境界を source policy に戻した。

- `owner_extent.rs` を `nodesrc/test_resource_checker_responsibility.js` の mandatory module / `mod` declaration / line limit 監視に追加した。
- `owner_check.rs` に残っていた raw memory operation handling を `owner_raw_memory.rs` へ分離した。
- `owner_consumption.rs` の extent-aware call argument consumption を `owner_consumption_extent.rs` へ分離した。
- `owner_flow.rs` の owner extent proof / unavailable diagnostic bridge を `owner_extent_check.rs` へ分離した。
- 既存の owner summary / owner variant 系大型 module は現状値に近い line limit へ校正し、source policy が常時 warning にならず今後の肥大化を検出できる状態にした。

検証:

- `cargo check -p nepl-core --tests`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/run_source_policy_regressions.js --warn-only`: passed with no warnings
- `cargo test -p nepl-core --test resource_ir raw_dealloc_extent -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir raw_realloc_old_extent -- --nocapture`: 1 passed
- `node nodesrc/issues.js check --dir issues`: passed

## 2026-05-16 追加修正

`OwnerStorageExtent::RegionTokenSize` の比較可能化が `owner_extent_check.rs` に戻り、`node nodesrc/test_resource_checker_responsibility.js` が `owner_extent_check.rs has 105 lines; responsibility split limit is 100` で失敗していた。

- `owner_extent_check.rs` は owner state 解決、extent proof 呼び出し、未証明要件の登録、診断 bridge に絞った。
- `RegionTokenSize` を実比較可能な payload-size place へ変換する責務を `owner_extent_compare.rs` へ分離した。
- `nodesrc/test_resource_checker_responsibility.js` に `owner_extent_compare.rs` の存在、`mod` 宣言、80 行上限を追加し、再び `owner_extent_check.rs` へ比較正規化が戻った場合に検出できるようにした。

検証:

- `cargo test -p nepl-core --test resource_ir raw_dealloc_extent -- --nocapture`: 2 passed
- `cargo test -p nepl-core --test resource_ir raw_realloc_old_extent -- --nocapture`: 1 passed
- `node nodesrc/issues.js check --dir issues`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: `owner_extent_check.rs` blocker は解消。次の別 issue として `owner_summary_raw_alias_walk.rs has 187 lines; responsibility split limit is 180` を検出したため `ISS-20260515T224600460Z-RESOURCE-OWNER-RAW-ALIAS-WALK-EXCEED-6BFE677D` に分離した。
