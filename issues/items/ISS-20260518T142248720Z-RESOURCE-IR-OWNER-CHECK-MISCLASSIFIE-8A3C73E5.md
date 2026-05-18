---
id: ISS-20260518T142248720Z-RESOURCE-IR-OWNER-CHECK-MISCLASSIFIE-8A3C73E5
title: "Resource IR owner check misclassifies selfhost diagnostic primary labels as leaking owner payload"
area: core
status: open
resolved: false
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

## 問題

While adding self-host checker diagnostics, returning a SelfhostDiagnostic with primary_label = Some(SelfhostDiagnosticLabel) caused resource.owner.maybe_leak on the label span fields in checker doctests. The reported places point inside SelfhostDiagnostic.primary_label rather than an actual owned allocation.

## 影響

Self-host checker diagnostics cannot safely use primary labels in focused doctests, reducing diagnostic precision. More importantly, Resource IR may still be treating ordinary diagnostic label payload fields as owner obligations.

## 修正方針

Review Resource IR owner classification for SelfhostDiagnosticLabel / Option payloads and ensure Copy scalar label fields are not treated as free obligations. Re-enable primary labels in self-host checker diagnostics after the owner proof is fixed.

## 検証

Add a regression where a self-host diagnostic with a primary label is returned and inspected without triggering resource.owner.maybe_leak.
