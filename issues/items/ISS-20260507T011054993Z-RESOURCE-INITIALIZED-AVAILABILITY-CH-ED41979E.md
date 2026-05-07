---
id: ISS-20260507T011054993Z-RESOURCE-INITIALIZED-AVAILABILITY-CH-ED41979E
title: "Resource initialized availability checks exceed responsibility split policy"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-05-07
updated: 2026-05-07
target: "nepl-core/src/resource/initialized.rs; nepl-core/src/resource/initialized_availability.rs; nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260507T011054993Z-RESOURCE-INITIALIZED-AVAILABILITY-CH-ED41979E: Resource initialized availability checks exceed responsibility split policy

## 概要

Resource initialized checker has grown past the source-policy responsibility limit because availability, consume, and unavailable-diagnostic helpers remain embedded in the operation dispatcher.

## 対象

- `nepl-core/src/resource/initialized.rs; nepl-core/src/resource/initialized_availability.rs; nodesrc/test_resource_checker_responsibility.js`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` で `initialized.rs` の責務分割上限超過が露出していた。
- `initialized.rs` には ResourceOp dispatch と、availability / consume / unavailable diagnostic の共通 helper が同居していた。
- availability / consume helper は `initialized_raw_memory.rs`、`initialized_summary_release.rs`、`initialized_variant.rs` などからも使われるため、operation dispatch とは別責務として分けるのが自然である。

## 問題

Resource initialized checker has grown past the source-policy responsibility limit because availability, consume, and unavailable-diagnostic helpers remain embedded in the operation dispatcher.

## 影響

Resource IR static-check logic starts re-centralizing around initialized.rs, making initialized/moved/drop-state auditability weaker and allowing future memory-safety fixes to accumulate without an explicit responsibility boundary.

## 修正方針

Extract availability and consume validation into an initialized_availability module, register that module in the source policy, and keep the operation dispatcher focused on ResourceOp control flow.

## 検証

Run node nodesrc/test_resource_checker_responsibility.js and focused Resource IR/drop checks.

## 2026-05-07 Agent 1 fixed

- `ResourceCheckEngine` の availability / consume / unavailable diagnostic helper を `initialized_availability.rs` に分離した。
- `initialized.rs` は function/block/op traversal と operation dispatch に責務を戻した。
- `nodesrc/test_resource_checker_responsibility.js` に `initialized_availability.rs` を登録し、`initialized.rs` 750 行 / `initialized_availability.rs` 120 行の上限で再集中を検出できるようにした。
- `initialized.rs` は 658/750 行、`initialized_availability.rs` は 108/120 行。
- 直接 policy は次の別問題 `owner_summary_leaf.rs has 387 lines; responsibility split limit is 260` を露出したため、`ISS-20260507T011238860Z-RESOURCE-OWNER-SUMMARY-LEAF-EXCEEDS--EE0957DE` を追加して継続追跡する。

確認:

- `cargo fmt --check -p nepl-core`
- `cargo check -p nepl-core --tests`
- `cargo test -p nepl-core resource:: --lib`
- `trunk build`
- `node -e ... initialized.rs / initialized_availability.rs line count`
- `node nodesrc/run_source_policy_regressions.js --warn-only`: initialized 側は解消。`owner_summary_leaf.rs` 超過のみ警告。
- `node nodesrc/issues.js check`
- `node nodesrc/tests.js -i tests/compiler/drop.n.md -i tests/compiler/drop_overwrite.n.md --no-tree --dist web/dist -o tmp/drop_agent1_after_initialized_availability_split.json -j 1 --assert-io`
