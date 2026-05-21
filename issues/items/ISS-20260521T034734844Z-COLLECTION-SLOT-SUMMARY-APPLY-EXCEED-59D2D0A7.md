---
id: ISS-20260521T034734844Z-COLLECTION-SLOT-SUMMARY-APPLY-EXCEED-59D2D0A7
title: "Collection slot proof modules exceed responsibility split limits"
area: core
status: fixed
resolved: true
priority: P1
type: maintenance
created: 2026-05-21
updated: 2026-05-21
target: "nepl-core/src/resource/collection_slot_summary_*.rs, nepl-core/src/resource/initialized_collection_slot*.rs, nepl-core/src/resource/raw_cell_value_flow*.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260521T034734844Z-COLLECTION-SLOT-SUMMARY-APPLY-EXCEED-59D2D0A7: Collection slot proof modules exceed responsibility split limits

## 概要

Collection slot non-Copy payload proof work introduced several concentrated modules: summary apply/replay, summary build, initialized collection slot proof application, and raw loaded-value flow tracking. `nodesrc/test_resource_checker_responsibility.js` exposed these as line-limit violations once the first blocker was split. These modules are part of the Resource IR memory-safety proof chain, so raising limits would hide static-check proof drift.

## 対象

- `nepl-core/src/resource/collection_slot_summary_*.rs`
- `nepl-core/src/resource/initialized_collection_slot*.rs`
- `nepl-core/src/resource/raw_cell_value_flow*.rs`
- `nepl-core/src/resource/initialized_call*.rs`
- `nodesrc/test_resource_checker_responsibility.js`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) は、Resource IR の proof boundary を enum / match で監査できる小さい単位へ分離することを Stage 6 の完了条件にしている。
- 親 issue [ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543](./ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543.md) は、non-Copy collection payload support を module allowlist ではなく generic Resource IR proof boundary に載せることを要求している。
- [ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC](./ISS-20260521T031943441Z-COLLECTION-SLOT-DROP-LIFECYCLE-NEEDS-DCBDB1EC.md) で追加した raw-loaded value drop proof は正しいが、実装が複数の既存 module に集中し、責務監視を通らなくなっていた。

## 問題

次の責務が混在していた。

- `collection_slot_summary_apply.rs`: call orchestration、summary op replay、path/loop merge replay、target instantiation、return transfer application。
- `collection_slot_summary_build.rs`: fixed-point summary computation、build state、op collection、callee summary translation、event proof construction。
- `collection_slot_summary_model.rs` / `collection_slot_summary_return_build.rs`: summary program model と return summary payload / merge helper。
- `initialized_collection_slot.rs`: lifecycle entry、summary proof conversion、storage relocate、proof rejection、unit tests。
- `raw_cell_value_flow.rs`: fact model、CellTable API wrapper、loaded-value origin transfer、path merge、tests。

## 影響

Resource IR collection-slot summaries and raw-loaded value flow are memory-safety proof infrastructure. Responsibility boundariesが曖昧なままだと、future non-Copy collection payload work が path merge、callee-certified proof replay、loaded-value drop proof、call argument ownership boundary を壊しても、小さな local change に見えてしまう。

## 修正方針

line limit を上げるのではなく、proof boundary ごとの module に分ける。

- summary replay: `collection_slot_summary_replay.rs`
- summary build state / op collection / translation / event proof / target projection: dedicated summary build modules
- return summary model / collection helper: dedicated return modules
- initialized collection slot: entry / apply / alias summary replay / relocate / proof rejection / tests
- raw value flow: fact engine / CellTable API wrapper / tests
- call argument loaded-origin discard and direct-call effect gate: dedicated initialized call helpers

## 検証

- `cargo check -p nepl-core`: passed
- `cargo test -p nepl-core raw_cell_value_flow -- --test-threads=1`: passed
- `cargo test -p nepl-core initialized_collection_slot -- --test-threads=1`: passed
- `cargo test -p nepl-core collection_slot -- --test-threads=1`: passed
- `node nodesrc/test_resource_checker_responsibility.js`: passed
- `node nodesrc/issues.js check --dir issues`: passed
- `cargo fmt --check`: passed
- `git diff --check`: passed
