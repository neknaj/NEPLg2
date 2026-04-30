---
id: ISS-20260429T135959086Z-RESOURCE-OWNER-FLOW-EXCEEDS-CHECKER--EE03E20E
title: "Resource owner_flow exceeds checker responsibility source policy limit"
area: core
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-05-01
target: "nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/owner_raw_memory.rs, nepl-core/src/resource/owner_flow.rs, nepl-core/src/resource/owner_return.rs, nepl-core/src/resource/owner_return_summary.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_summary_cleanup.rs, nepl-core/src/resource/owner_summary_record.rs, nepl-core/src/resource/lower.rs, nepl-core/src/resource/lower_condition.rs, nepl-core/src/resource/lower_raw_address.rs, nepl-core/src/resource/lower_raw_address_place.rs, nepl-core/src/resource/mod.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260429T135959086Z-RESOURCE-OWNER-FLOW-EXCEEDS-CHECKER--EE03E20E: Resource owner_flow exceeds checker responsibility source policy limit

## 概要

GitHub Actions Source policy regressions fail because owner_flow.rs has 693 lines while nodesrc/test_resource_checker_responsibility.js enforces a 620-line responsibility split limit. This indicates Resource IR owner flow transfer/summary/raw-address responsibilities have re-concentrated in one module. After splitting owner_flow.rs, the same source policy exposed summary.rs as another oversized mixed responsibility module, so the fix must include summary responsibility separation as part of the same root cause.

## 対象

- `nepl-core/src/resource/owner_flow.rs`
- `nepl-core/src/resource/summary.rs`
- `nepl-core/src/resource/borrow_summary.rs`
- `nepl-core/src/resource/owner_summary.rs`
- `nepl-core/src/resource/owner_summary_leaf.rs`
- `nepl-core/src/resource/owner_alias.rs`
- `nepl-core/src/resource/owner_raw_address.rs`
- `nepl-core/src/resource/owner_transfer.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 関連計画

- `doc/neplg2/static_check_complexity_reduction_plan.md`

## 根拠

- `nodesrc/test_resource_checker_responsibility.js` が owner_flow.rs の責務分割上限を 620 行として検査していたが、owner_flow.rs は 693 行まで肥大化していた。
- owner_flow.rs には、owner alias 解決、raw address の所有権分類、owner state transfer が同居していた。
- summary.rs には、summary data model、borrow summary 固定点計算、owner summary 固定点計算、owner leaf projection 展開が同居していた。

## 問題

GitHub Actions Source policy regressions fail before the static check regression tests can run. More importantly, Resource IR の owner 検査で「flow 適用」「alias 解決」「raw address 分類」「state transfer」「summary 計算」「leaf projection 展開」が大きいモジュールへ再集約され、静的検査大規模修正の目的である責務分離と監査可能性が崩れていた。

## 影響

CI stops before rust/std/doctest jobs, hiding real static-check regressions. It also violates the static check complexity reduction plan by letting the Resource IR owner checker grow a new large responsibility cluster.

## 修正方針

Split owner_flow.rs and summary.rs by responsibility instead of raising the limits:

- owner alias resolution is isolated in owner_alias.rs.
- raw address ownership classification is isolated in owner_raw_address.rs and consumed by owner_flow.rs through enum-based match dispatch.
- low-level owner state transfer is isolated in owner_transfer.rs.
- borrow summary fixed-point computation is isolated in borrow_summary.rs.
- owner summary fixed-point computation is isolated in owner_summary.rs.
- owner leaf projection expansion is isolated in owner_summary_leaf.rs.
- summary.rs keeps only the shared summary data model and re-exports the computation entry points.

## 検証

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `cargo fmt --check -p nepl-core`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 31 passed
- `cargo check -p nepl-core --tests`: passed

## 解決

2026-04-29 に Resource IR owner 検査の責務分割を追加した。

- owner_flow.rs から alias/raw-address/transfer helper を分離し、raw address return 分類は enum を `match` する形で flow 適用本体へ責務を戻した。
- summary.rs から borrow summary / owner summary / owner leaf projection を分離し、summary.rs は data model と入口 re-export のみへ縮小した。
- source policy test に新モジュールの存在と行数上限を追加し、owner/summary 責務の再集約を検出できるようにした。

## 2026-04-30 再発対応

`fix(stdlib): make btree storage owner-safe` の取り込み後、main CI の Source policy regressions で `owner_flow.rs has 700 lines; responsibility split limit is 620` が再発した。owner return summary 適用が `owner_flow.rs` に再集約され、さらに分離後の確認で `owner_summary.rs` も 391 行となり 380 行上限を超えていた。

修正:

- call return / indirect call return / owner return summary 適用を `owner_return.rs` へ分離した。
- owner summary の projection/source 記録 helper と `OwnerParameterStorageSource` を `owner_summary_record.rs` へ分離した。
- `resource/mod.rs` と `nodesrc/test_resource_checker_responsibility.js` に新モジュール境界と行数上限を追加した。

検証:

- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `rustfmt --check nepl-core/src/resource/mod.rs nepl-core/src/resource/owner_flow.rs nepl-core/src/resource/owner_return.rs nepl-core/src/resource/owner_summary.rs nepl-core/src/resource/owner_summary_record.rs`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_ -- --nocapture`: 40 passed

## 2026-05-01 再発対応

remote main 同期後、strict resource checker responsibility policy が owner/lower 側の再集約を検出した。

- `owner_check.rs`: 905/800。raw memory operation の owner semantics が main checker dispatch に戻っていた。
- `owner_summary.rs`: 388/380。variant projection return cleanup が summary fixed-point 本体に残っていた。
- `owner_return.rs`: 518/400。call return entry point と owner return summary application が同居していた。
- `lower.rs`: 1386/1300。condition fact lowering と place skeleton 復元が lowering 本体に残っていた。
- `lower_raw_address.rs`: 733/700。raw address source 推論と MemPtr/RegionToken place/type helper が同居していた。

修正:

- raw memory owner semantics を `owner_raw_memory.rs` へ分離した。
- variant projection return cleanup を `owner_summary_cleanup.rs` へ分離した。
- owner return summary application を `owner_return_summary.rs` へ分離した。
- condition fact lowering / place skeleton 復元を `lower_condition.rs` へ分離した。
- raw address place/type helper を `lower_raw_address_place.rs` へ分離した。
- `nodesrc/test_resource_checker_responsibility.js` は新 module の存在と line limit を監視する。

検証:

- `cargo fmt --check`: passed
- `cargo check -p nepl-core`: passed
- owner/lower 側 line count は policy 内。strict policy は別件の initialized 系再肥大化で停止するため、そちらは再openした issue で継続する。
