---
id: ISS-20260428T232624608Z-SELFHOSTOUTCOME-STORES-RESULT-IN-RAW-219628DB
title: "SelfhostOutcome stores Result in raw pointer cell rejected by RawMemoryLoadCell"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md, nodesrc/test_selfhost_outcome_no_raw_result_cell.js"
---

# ISS-20260428T232624608Z-SELFHOSTOUTCOME-STORES-RESULT-IN-RAW-219628DB: SelfhostOutcome stores Result in raw pointer cell rejected by RawMemoryLoadCell

## 概要

SelfhostOutcome keeps Result<T,E> in a one-cell MemPtr and later loads it after the pointer has travelled through an aggregate field. Under the RawMemoryLoadCell gate, selfhost_outcome_result on i32,str reports the result_ptr cell as Uninit even though selfhost_outcome_new stored the Result.

## 対象

- `stdlib/neplg2/core/infra/outcome.nepl, tests/stdlib/neplg2_diag_outcome.n.md`

## 根拠

- `node nodesrc/tests.js -i stdlib\neplg2\core\infra\outcome.nepl --no-tree -o tmp\outcome-current-after-cli-merge.json -j 1` が `total=1, passed=0, failed=1` になった。
- `selfhost_outcome_result__SelfhostOutcome_T_E_T_E__Result_T_E_T_E__imp_i32_str` の `load<Result<.T, .E>> mem_ptr_addr result_ptr` が D3100 になり、`result_ptr.*` は `Uninit` と報告された。
- `selfhost_outcome_new` は同じ cell に `store<Result<T,E>>` しているため、表面上の missing store ではなく、`MemPtr` を aggregate field へ入れて戻した時に initialized cell provenance が stage 境界から失われている。

## 問題

SelfhostOutcome keeps Result<T,E> in a one-cell MemPtr and later loads it after the pointer has travelled through an aggregate field. Under the RawMemoryLoadCell gate, selfhost_outcome_result on i32,str reports the result_ptr cell as Uninit even though selfhost_outcome_new stored the Result.

## 影響

The self-host diagnostic outcome smoke doctest fails, and the stage boundary encourages preserving a raw-memory workaround instead of using a normal owned Result field.

## 修正方針

Replace the one-cell MemPtr result storage with direct Result<T,E> ownership inside SelfhostOutcome, then update result extraction, diagnostic push, free, and payload cleanup paths so raw storage is not used for the stage result.

## 検証

Run outcome focused doctests, tests/stdlib/neplg2_diag_outcome.n.md, stdlib/neplg2 focused tests, node nodesrc/issues.js check, and git diff --check.

## 修正内容

- `SelfhostOutcome<T,E>` の `result` field を `MemPtr<Result<T,E>>` から直接 owned `Result<T,E>` へ変更した。
- `selfhost_outcome_new` は result cell allocation / raw store を行わず、diagnostics と result をそのまま outcome へ格納するようにした。
- `selfhost_outcome_result` / `selfhost_outcome_free` / diagnostics push failure cleanup から result cell の raw load/dealloc を削除した。
- `nodesrc/test_selfhost_outcome_no_raw_result_cell.js` を追加し、`MemPtr<Result<T,E>>`、raw `load/store<Result<T,E>>`、result pointer deallocator の再導入を禁止した。

## 検証結果

- `node nodesrc/test_selfhost_outcome_no_raw_result_cell.js`: pass
- `node nodesrc/tests.js -i stdlib\neplg2\core\infra\outcome.nepl --no-tree -o tmp\outcome-direct-result-focused-2.json -j 1`: total=1, passed=1
- `node nodesrc/tests.js -i stdlib\neplg2 --no-tree -o tmp\outcome-direct-result-neplg2.json -j 1`: total=32, passed=23, failed=9。`core/infra/outcome.nepl` は pass。残件は既知の Vec element provenance 系。
- `node nodesrc/tests.js -i tests\stdlib\neplg2_diag_outcome.n.md --no-tree -o tmp\outcome-direct-result-fixture-4.json -j 1`: total=3, passed=0, failed=3。doctest#1/#2 は既知の Vec element provenance、doctest#3 は新規 `ISS-20260428T233410073Z-GENERIC-OWNED-AGGREGATE-FIELD-MOVES--0A6FA87B` に分離した generic aggregate partial-move D3053。
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
