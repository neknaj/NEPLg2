---
id: ISS-20260520T131842206Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-7DA20949
title: "Stdlib documentation contract declaration doctest baseline regressed to 1036"
area: stdlib
status: fixed
resolved: true
priority: P1
type: doc
created: 2026-05-20
updated: 2026-05-20
target: "stdlib/core/**, stdlib/alloc/**, stdlib/std/**, nodesrc/test_stdlib_documentation_contract.js"
---

# ISS-20260520T131842206Z-STDLIB-DOCUMENTATION-CONTRACT-DECLAR-7DA20949: Stdlib documentation contract declaration doctest baseline regressed to 1036

## 概要

The global stdlib documentation contract currently reports declarationNoDoctest=1036 while the frozen baseline is 1032. This failure is outside the selfhost checker files changed for ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D, but it blocks the aggregate source-policy runner from reporting cleanly.

## 対象

- `stdlib/core/**, stdlib/alloc/**, stdlib/std/**, nodesrc/test_stdlib_documentation_contract.js`

## 根拠

- `node nodesrc/test_stdlib_documentation_contract.js` fails with `stdlib declaration doctest gaps increased: 1036 > 1032`.
- `node nodesrc/run_source_policy_regressions.js` reaches `nodesrc/test_stdlib_documentation_contract.js` and stops on the same failure.
- The selfhost checker timeout fix touches `stdlib/neplg2/core/check/**`, while the documentation contract scans only `stdlib/core`, `stdlib/alloc`, and `stdlib/std`; this regression is therefore separated from `ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D`.
- Baseline comparison identified the four new declaration doctest gaps as `VecDataView`, `VecCopyInvariantInvalid`, `VecCopyInvariant`, and `vec_current_copy_invariant`.

## 問題

The global stdlib documentation contract currently reports declarationNoDoctest=1036 while the frozen baseline is 1032. This failure is outside the selfhost checker files changed for ISS-20260520T125846092Z-SELFHOST-MODULE-CHECKER-DOC-COMMENT--EA72F33D, but it blocks the aggregate source-policy runner from reporting cleanly.

## 影響

Global source-policy verification remains noisy and executable documentation coverage for public stdlib declarations has regressed below the enforced baseline.

## 修正方針

Audit the four new declaration doctest gaps, add meaningful n.md-style doctests instead of relaxing the baseline, and keep the documentation contract baseline at 1032 or lower.

## 修正

- `VecDataView` の enum doc に、`with_capacity 0` が `Empty` view、`new` が allocated `Data` view になることを明示する doctest を追加した。
- 既存の `data_mem_view` doctest は `new` を empty storage と誤解していたため、`with_capacity 0` を使う実行例へ修正し、`exit_code: 0` を明示した。
- `VecCopyInvariantInvalid` の enum doc に、failure reason を exhaustive match で扱う doctest を追加した。
- `VecCopyInvariant` の enum doc に、`Valid` proof と `Invalid(reason)` payload の両方を match する doctest を追加した。
- `vec_current_copy_invariant` の function doc に、public `Vec` facade から invariant proof を得て `Valid` を確認する doctest を追加した。

## 検証

- `node nodesrc/test_stdlib_documentation_contract.js`: passed (`declarationNoDoctest=1032`)
- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`: passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec/access/data.nepl -i stdlib/alloc/collections/vec/invariant.nepl --no-tree --dist web/dist -o tmp/agent1-vec-doc-contract-1036.json -j 1 --assert-io`: 7/7 passed
- `node nodesrc/run_source_policy_regressions.js`: passed
