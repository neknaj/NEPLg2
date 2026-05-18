---
id: ISS-20260518T142248720Z-RESOURCE-IR-OWNER-CHECK-MISCLASSIFIE-8A3C73E5
title: "Resource IR owner check misclassifies selfhost diagnostic primary labels as leaking owner payload"
area: core
status: fixed
resolved: true
priority: P2
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "nepl-core/src/resource/**, stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/check/module.nepl"
---

# ISS-20260518T142248720Z-RESOURCE-IR-OWNER-CHECK-MISCLASSIFIE-8A3C73E5: Resource IR owner check misclassifies selfhost diagnostic primary labels as leaking owner payload

## 概要

While adding self-host checker diagnostics, returning a SelfhostDiagnostic with primary_label = Some(SelfhostDiagnosticLabel) caused resource.owner.maybe_leak on the label span fields in checker doctests. The reported places point inside SelfhostDiagnostic.primary_label rather than an actual owned allocation.

## 対象

- `nepl-core/src/resource/**, stdlib/neplg2/core/infra/diag.nepl, stdlib/neplg2/core/check/module.nepl`

## 根拠

- self-host checker diagnostic に `primary_label = Some(SelfhostDiagnosticLabel)` を付けた状態で focused doctest を実行すると、`resource.owner.maybe_leak` が `SelfhostDiagnostic.primary_label` 配下の span scalar field を指した。
- 同じ diagnostic code/message を label なしで返すと checker doctest は通過するため、問題は checker の raw block state machine ではなく、diagnostic label payload の Resource IR owner classification にある。
- diagnostic label は ownership payload ではなく source span metadata なので、ここを owner obligation として扱うと正しい diagnostic 設計を阻害する。
- 再現時の owner place は `SelfhostDiagnostic.primary_label.Some.label.span.{file_id,start,end}` のような nested `i32` metadata であり、base type は `Copy` aggregate だった。
- `MemPtr<T>` のような raw pointer view や `RegionToken<T>` を含む owner token aggregate まで除外すると外部 IO / owner proof を弱めるため、除外条件は「nested leaf」「base が Copy」「base が raw pointer ではない」「base が owner token を含まない」に限定する必要がある。

## 問題

While adding self-host checker diagnostics, returning a SelfhostDiagnostic with primary_label = Some(SelfhostDiagnosticLabel) caused resource.owner.maybe_leak on the label span fields in checker doctests. The reported places point inside SelfhostDiagnostic.primary_label rather than an actual owned allocation.

## 影響

Self-host checker diagnostics cannot safely use primary labels in focused doctests, reducing diagnostic precision. More importantly, Resource IR may still be treating ordinary diagnostic label payload fields as owner obligations.

## 修正方針

Resource IR owner summary の raw `i32` leaf seed で、Copy aggregate の nested scalar metadata を free obligation owner candidate から外す。これは diagnostic 名や stdlib module の allowlist ではなく、型構造と compiler memory type identity による汎用条件にする。`MemPtr` / owner token を含む aggregate は除外せず、external IO や owner transfer proof に必要な leaf は従来通り残す。owner proof 修正後に self-host checker diagnostic の primary label を復帰する。

## 検証

Add a regression where a self-host diagnostic with a primary label is returned and inspected without triggering resource.owner.maybe_leak.

## 対応内容

- `owner_seed_leaf_places` が `raw_i32_owner_leaf_places` の候補を取り込む前に、Copy aggregate の nested metadata scalar を判定して除外するようにした。
- 除外条件から `MemPtr<T>` と owner token を含む型を外し、raw pointer / free obligation owner の証明に必要な `i32` leaf を誤って落とさないようにした。
- self-host module checker の diagnostic helper に primary label を戻し、raw block 空、raw text block 不一致、invalid span、module item index 欠落の診断で原因箇所を label として出せるようにした。
- `tests/stdlib/neplg2_checker.n.md` の stdio doctest で別件の `ExternalIoPayloadExtent` proof gap が露出したため、`ISS-20260518T145905099Z-RESOURCE-IR-FD-WRITE-PAYLOAD-EXTENT--C471191E` として分離した。

## 検証結果

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_ignores_copy_diagnostic_label_i32_payloads -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_variant_reservation_ignores_copy_payload_sources -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reinitializes_self_update_aggregate_return -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_plain_i32_identity_as_owner_return -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_summary_does_not_treat_plain_i32_struct_fields_as_owners -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup -- --nocapture`: pass。
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_does_not_treat_raw_cell_payload_as_storage_owner -- --nocapture`: pass。
- `trunk build`: pass。
- `node nodesrc/tests.js -i stdlib/neplg2/core/check/module.nepl -i stdlib/neplg2/core/check/checker.nepl -i stdlib/neplg2/core/pipeline.nepl -i tests/stdlib/neplg2_checker.n.md --no-tree -o tmp/agent1-resource-diag-label-checker-after-rawptr.json -j 1 --dist web/dist --assert-io`: total=5, passed=3, failed=2。primary label の `resource.owner.maybe_leak` は消え、残り 2 件は stdio doctest の別件 `ExternalIoPayloadExtent` proof gap。
