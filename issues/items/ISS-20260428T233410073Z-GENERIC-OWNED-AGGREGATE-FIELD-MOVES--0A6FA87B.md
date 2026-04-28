---
id: ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B
title: "generic owned aggregate field moves still reject SelfhostOutcome direct Result cleanup"
area: core
status: open
resolved: false
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "nepl-core/src/passes/move_check.rs, stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md"
---

# ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B: generic owned aggregate field moves still reject SelfhostOutcome direct Result cleanup

## 概要

After SelfhostOutcome stores Result<T,E> directly, selfhost_outcome_result/selfhost_outcome_free must move both result and diagnostics fields from SelfhostOutcome<T,E>. For Result<DropCounter,str>, move_check reports D3053 use of moved value on the second field extraction even though the fields are disjoint.

## 対象

- `nepl-core/src/passes/move_check.rs, stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md`

## 根拠

- `SelfhostOutcome<T,E>` を `result <Result<T,E>>` / `diagnostics <SelfhostDiagnostics>` の直接 owned field へ変更した後、`node nodesrc/tests.js -i tests\stdlib\neplg2_diag_outcome.n.md --no-tree -o tmp\outcome-direct-result-fixture-4.json -j 1` を実行した。
- `tests\stdlib\neplg2_diag_outcome.n.md::doctest#3` の `selfhost_outcome_free<DropCounter,str>` 経路で、`selfhost_outcome_result` 内の 2 つ目の field extraction が `error[D3053]: use of moved value: outcome` になった。
- 同じ issue family の `ISS-20260426T175008731Z-OWNED-AGGREGATE-DECOMPOSITION-LACKS--48C352EE` は verified だが、今回の再現は `SelfhostOutcome<.T,.E>` の generic field `Result<.T,.E>` が実体化後に non-Copy になるケースで、既存修正の対象から漏れている。

## 問題

After SelfhostOutcome stores Result<T,E> directly, selfhost_outcome_result/selfhost_outcome_free must move both result and diagnostics fields from SelfhostOutcome<T,E>. For Result<DropCounter,str>, move_check reports D3053 use of moved value on the second field extraction even though the fields are disjoint.

## 影響

SelfhostOutcome can remove the raw result cell for Copy payload smoke tests, but non-Copy payload cleanup remains blocked by a compiler partial-move regression. Keeping the raw pointer workaround would hide this compiler issue and retain unsafe stage storage.

## 修正方針

Extend owned aggregate decomposition so generic struct fields with instantiated non-Copy payloads can be moved exactly once by disjoint field path. Preserve rejection of repeated field moves, owner use after partial move, and borrow-live field moves.

## 検証

Add compiler move_check regression for a generic struct with two non-Copy fields, run tests/compiler/move_check.n.md, run tests/stdlib/neplg2_diag_outcome.n.md after Vec element provenance is fixed, node nodesrc/issues.js check, and git diff --check.
